use crate::{
    CellIndex, DirectedEdgeIndex, Resolution,
    error::DissolutionError,
    math::{Coord2d, linear_ring_area},
};
use ahash::{HashMap, HashMapExt as _, HashSet};
use core::f64::consts::{FRAC_PI_2, PI};
use geo::{Coord, LineString, MultiPolygon, Polygon, ToDegrees as _, polygon};
use ordered_float::OrderedFloat;
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct ArcSet {
    arcs: Option<Vec<Arc>>,
    partitions: UnionFind,
}

impl ArcSet {
    /// Initializes a new `ArcSet` from a set of homogeneous cells.
    pub fn new(
        cells: impl IntoIterator<Item = CellIndex>,
    ) -> Result<Self, DissolutionError> {
        let cells = validate_homogeneous(cells)?;
        if cells.is_empty() {
            return Ok(Self::default());
        }

        let mut solvent = ArcSolvent::new(cells.len(), 6);
        for cell in cells {
            let count = if cell.is_pentagon() { 5 } else { 6 };
            solvent.add_edges(cell.edges(), count);
        }

        Ok(solvent.into())
    }

    /// Initializes a new `ArcSet` from a set of heterogeneous cells.
    pub fn new_heterogeneous(
        cells: impl IntoIterator<Item = CellIndex>,
        resolution: Resolution,
    ) -> Result<Self, DissolutionError> {
        let (cells, batch_size) = validate_heterogeneous(cells, resolution)?;
        if cells.is_empty() {
            return Ok(Self::default());
        }

        let mut solvent = ArcSolvent::new(cells.len(), batch_size);
        for cell in cells {
            let gosper = cell.outline(resolution);
            let count = gosper.len();

            solvent.add_edges(gosper, count);
        }

        Ok(solvent.into())
    }

    // Extract all linear ring and sort them by:
    // 1. their connected component, and then by
    // 2. the loop area.
    //
    // This makes rings for each polygon contiguous in memory.
    //
    // Within each polygon, the sorting makes the loop with the smallest enclosed
    // area come first (accounting for loop winding direction), which is what we
    // take to be the outer loop for that polygon.
    fn build_rings(&mut self) -> Option<Vec<LinearRing>> {
        let arcs = self.arcs.as_ref()?;

        let mut visited: HashSet<DirectedEdgeIndex> = HashSet::default();
        let mut rings = Vec::new();

        for arc in arcs {
            if arc.is_removed() || visited.contains(&arc.id) {
                continue;
            }

            let polygon_id = self.partitions.find(arc.polygon_id);
            rings.push(LinearRing::new(arcs, arc, &mut visited, polygon_id));
        }

        rings.sort_unstable_by_key(|ring| (ring.polygon_id, ring.area));
        Some(rings)
    }
}

impl From<ArcSolvent> for ArcSet {
    fn from(mut value: ArcSolvent) -> Self {
        value.finalize();

        Self {
            arcs: Some(value.arcs),
            partitions: value.partitions,
        }
    }
}

impl From<ArcSet> for MultiPolygon {
    fn from(mut value: ArcSet) -> Self {
        let Some(rings) = value.build_rings() else {
            return Self::new(Vec::new());
        };
        build_multipolygon(rings)
    }
}

// Check for duplicate and heterogeneous cell resolution.
fn validate_homogeneous(
    cells: impl IntoIterator<Item = CellIndex>,
) -> Result<Vec<CellIndex>, DissolutionError> {
    let mut cells = cells.into_iter().collect::<Vec<_>>();

    let old_len = cells.len();
    cells.sort_unstable();
    cells.dedup();
    if cells.len() < old_len {
        return Err(DissolutionError::DuplicateInput);
    }

    // Infer the resolution from the first cell (since its homogeneous).
    let Some(resolution) = cells.first().copied().map(CellIndex::resolution)
    else {
        return Ok(Vec::new());
    };

    if cells.iter().any(|cell| cell.resolution() != resolution) {
        return Err(DissolutionError::UnsupportedResolution);
    }

    Ok(cells)
}

// Check for duplicate (including ancestors) and unsupported cell resolution.
fn validate_heterogeneous(
    cells: impl IntoIterator<Item = CellIndex>,
    resolution: Resolution,
) -> Result<(Vec<CellIndex>, usize), DissolutionError> {
    let cells = cells.into_iter().collect::<Vec<_>>();
    let all = cells.iter().copied().collect::<HashSet<_>>();
    if all.len() != cells.len() {
        return Err(DissolutionError::DuplicateInput);
    }

    let mut max_batch_size = 6;
    for &cell in &all {
        if cell.resolution() > resolution {
            return Err(DissolutionError::UnsupportedResolution);
        }

        for res in Resolution::range(Resolution::Zero, cell.resolution())
            .rev()
            .skip(1)
        {
            let ancestor =
                cell.parent(res).expect("loop range makes it infaillible");
            if all.contains(&ancestor) {
                return Err(DissolutionError::DuplicateInput);
            }
        }

        let nb_sides = 6 - usize::from(cell.is_pentagon());
        let delta = u8::from(resolution) - u8::from(cell.resolution());
        let batch_size = nb_sides * 3_usize.pow(delta.into());
        if batch_size > max_batch_size {
            max_batch_size = batch_size;
        }
    }

    Ok((cells, max_batch_size))
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
struct Arc {
    id: DirectedEdgeIndex,
    polygon_id: usize,

    // Link this arc to its neighbors in the linear ring.
    prev_idx: Cell<usize>,
    next_idx: Cell<usize>,
}

impl Arc {
    const fn new(
        id: DirectedEdgeIndex,
        polygon_id: usize,
        prev_idx: usize,
        next_idx: usize,
    ) -> Self {
        Self {
            id,
            polygon_id,
            prev_idx: Cell::new(prev_idx),
            next_idx: Cell::new(next_idx),
        }
    }

    fn is_removed(&self) -> bool {
        self.prev_idx == self.next_idx
    }
}

// -----------------------------------------------------------------------------

struct ArcSolvent {
    arcs: Vec<Arc>,
    index: HashMap<DirectedEdgeIndex, usize>,
    freelist: Vec<usize>,
    slots: Vec<usize>,
    partitions: UnionFind,
}

impl ArcSolvent {
    fn new(cell_count: usize, max_batch_size: usize) -> Self {
        // If max_batch_size is 6 it means we're processing homogeneous input.
        if max_batch_size == 6 {
            let factor = cell_count.checked_next_power_of_two().unwrap_or(4096);
            let arcs = (factor * 2).clamp(32, 4096);

            Self {
                arcs: Vec::with_capacity(arcs),
                index: HashMap::with_capacity(arcs),
                freelist: Vec::with_capacity(128),
                slots: Vec::with_capacity(6),
                partitions: UnionFind::new(cell_count),
            }
        } else {
            // Empirical factor derived from experimentations.
            // Not set in stone.
            Self {
                arcs: Vec::with_capacity(cell_count * 3),
                index: HashMap::with_capacity(cell_count * 6),
                freelist: Vec::with_capacity(max_batch_size * 2),
                slots: Vec::with_capacity(max_batch_size),
                partitions: UnionFind::new(cell_count),
            }
        }
    }

    fn add_edges(
        &mut self,
        edges: impl IntoIterator<Item = DirectedEdgeIndex>,
        nb_slots: usize,
    ) {
        // Pre-compute slots so that we can set {prev,next}_idx ahead of time.
        let to_reuse = std::cmp::min(nb_slots, self.freelist.len());
        let leftover = self.freelist.len() - to_reuse;
        let to_alloc = nb_slots - to_reuse;
        self.slots.clear();
        self.slots.extend(self.freelist.drain(leftover..));
        self.slots
            .extend((0..to_alloc).map(|offset| self.arcs.len() + offset));

        // Create and insert the arcs.
        let polygon_id = self.partitions.add();
        let len = self.slots.len();
        for ((idx, slot), edge) in
            self.slots.iter().copied().enumerate().zip(edges)
        {
            let prev = if idx == 0 { len - 1 } else { idx - 1 };
            let next = if idx == len - 1 { 0 } else { idx + 1 };
            let prev_idx = self.slots[prev];
            let next_idx = self.slots[next];

            let arc = Arc::new(edge, polygon_id, prev_idx, next_idx);
            if slot < self.arcs.len() {
                self.arcs[slot] = arc;
            } else {
                self.arcs.push(arc);
            }
            self.index.insert(edge, slot);
        }

        // Dissolve the duplicated ones.
        for &slot in &self.slots {
            let arc = &self.arcs[slot];
            // Already dissolved, nothing to do.
            if arc.is_removed() {
                continue;
            }
            let edge = self.arcs[slot].id;
            let Some(&rev_idx) = self.index.get(&edge.reverse()) else {
                // This edge is unique, nothing to do.
                continue;
            };
            let cur_idx = slot;
            self.merge_arcs(cur_idx, rev_idx);

            // Merge them into a single linear ring.
            self.partitions
                .union(arc.polygon_id, self.arcs[rev_idx].polygon_id);
            self.index.remove(&self.arcs[cur_idx].id);
            self.index.remove(&self.arcs[rev_idx].id);

            self.freelist.push(cur_idx);
            self.freelist.push(rev_idx);
        }
    }

    // Extract and index edges from a list of cells.
    // Merge arcs together.
    //
    // Remove the shared edge and stitch both side together into a linear ring.
    fn merge_arcs(&self, cur_idx: usize, rev_idx: usize) {
        // Stitch both side.
        let cur_prev_idx = self.arcs[cur_idx].prev_idx.get();
        let cur_next_idx = self.arcs[cur_idx].next_idx.get();
        let rev_prev_idx = self.arcs[rev_idx].prev_idx.get();
        let rev_next_idx = self.arcs[rev_idx].next_idx.get();
        self.arcs[cur_next_idx].prev_idx.set(rev_prev_idx);
        self.arcs[cur_prev_idx].next_idx.set(rev_next_idx);
        self.arcs[rev_next_idx].prev_idx.set(cur_prev_idx);
        self.arcs[rev_prev_idx].next_idx.set(cur_next_idx);

        // Remove both edges.
        self.arcs[cur_idx]
            .next_idx
            .set(self.arcs[cur_idx].prev_idx.get());
        self.arcs[rev_idx]
            .next_idx
            .set(self.arcs[rev_idx].prev_idx.get());
    }

    fn finalize(&mut self) {
        for &slot in &self.freelist {
            self.arcs[slot].prev_idx.set(self.arcs[slot].next_idx.get());
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(capacity: usize) -> Self {
        Self {
            parent: Vec::with_capacity(capacity),
            rank: Vec::with_capacity(capacity),
        }
    }

    fn add(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        id
    }

    /// Return the parent of the node `a`.
    fn find(&mut self, mut a: usize) -> usize {
        // Use path halving, seems to be marginally faster (up to 5%) than path
        // compression and path splitting.
        while self.parent[a] != a {
            self.parent[a] = self.parent[self.parent[a]];
            a = self.parent[a];
        }

        a
    }

    /// Merge the sets `a` and `b` together.
    ///
    /// Returns false if both set were already connected.
    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return;
        }

        // Union by rank
        // Cf. https://en.wikipedia.org/wiki/Disjoint-set_data_structure#Union_by_rank
        if self.rank[root_a] < self.rank[root_b] {
            self.parent[root_a] = root_b;
            self.rank[root_b] += 1;
        } else {
            self.parent[root_b] = root_a;
            self.rank[root_a] += 1;
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
struct LinearRing {
    polygon_id: usize,
    area: OrderedFloat<f64>,
    coords: Vec<Coord>,
}

impl LinearRing {
    fn new(
        arcs: &[Arc],
        start: &Arc,
        visited: &mut HashSet<DirectedEdgeIndex>,
        polygon_id: usize,
    ) -> Self {
        let mut count = 0;
        let mut arc = start;
        loop {
            // We overestimate since most edge will only have a single vertex.
            // Only cells spanning more than one icosahedron face, for class III
            // resolution will have an extra distortion vertex.
            //
            // But at least with this we have an upper bound and avoir
            // reallocation and copy.
            count += 2;
            arc = &arcs[arc.next_idx.get()];
            if arc.id == start.id {
                break;
            }
        }

        let mut coords = Vec::with_capacity(count);
        loop {
            let vertices = arc.id.boundary();
            coords.extend(
                vertices
                    .iter()
                    // XXX: not using the From impl (it converts to degrees).
                    .map(|ll| Coord {
                        x: ll.lng_radians(),
                        y: ll.lat_radians(),
                    })
                    .take(vertices.len().saturating_sub(1)),
            );
            visited.insert(arc.id);
            arc = &arcs[arc.next_idx.get()];
            if arc.id == start.id {
                break;
            }
        }

        Self {
            polygon_id,
            area: OrderedFloat(linear_ring_area(&coords)),
            coords,
        }
    }
}

impl From<LinearRing> for LineString {
    fn from(value: LinearRing) -> Self {
        Self::new(value.coords)
    }
}

// -----------------------------------------------------------------------------

fn build_multipolygon(rings: Vec<LinearRing>) -> MultiPolygon {
    if rings.is_empty() {
        return world_polygon();
    }

    let mut rings = rings.into_iter();
    let mut outer = rings.next().expect("rings is not empty");
    let mut holes = Vec::new();
    let mut polygons = Vec::new();
    for mut ring in rings {
        if ring.polygon_id == outer.polygon_id {
            holes.push(ring);
        } else {
            #[expect(clippy::iter_with_drain, reason = "false positive")]
            let interiors = holes.drain(..).map(Into::into).collect::<Vec<_>>();
            std::mem::swap(&mut ring, &mut outer);
            polygons.push(Polygon::new(ring.into(), interiors));
        }
    }
    polygons.push(Polygon::new(
        outer.into(),
        holes.into_iter().map(Into::into).collect(),
    ));

    polygons.sort_by_cached_key(|polygon| {
        core::cmp::Reverse(OrderedFloat(linear_ring_area(
            &polygon.exterior().0,
        )))
    });

    let mut mpoly = MultiPolygon::new(polygons);
    mpoly.to_degrees_in_place();
    mpoly
}

// Returns a `MultiPolygon` representing the entire world.
//
// The world is represented using 8 triangular polygons, with
// all edge arcs of exactly 90 degrees (i.e., π/2 radians).
fn world_polygon() -> MultiPolygon {
    let mut polygons = vec![
        polygon![(x: 0., y: FRAC_PI_2),  (x: 0.,         y: 0.), (x: FRAC_PI_2,  y: 0.)],
        polygon![(x: 0., y: FRAC_PI_2),  (x: FRAC_PI_2,  y: 0.), (x: PI,         y: 0.)],
        polygon![(x: 0., y: FRAC_PI_2),  (x: PI,         y: 0.), (x: -FRAC_PI_2, y: 0.)],
        polygon![(x: 0., y: FRAC_PI_2),  (x: -FRAC_PI_2, y: 0.), (x: 0.,         y: 0.)],
        polygon![(x: 0., y: -FRAC_PI_2), (x: 0.,         y: 0.), (x: -FRAC_PI_2, y: 0.)],
        polygon![(x: 0., y: -FRAC_PI_2), (x: -FRAC_PI_2, y: 0.), (x: -PI,        y: 0.)],
        polygon![(x: 0., y: -FRAC_PI_2), (x: -PI,        y: 0.), (x: FRAC_PI_2,  y: 0.)],
        polygon![(x: 0., y: -FRAC_PI_2), (x: FRAC_PI_2,  y: 0.), (x: -0.,        y: 0.)],
    ];

    polygons.sort_by_cached_key(|polygon| {
        core::cmp::Reverse(OrderedFloat(linear_ring_area(
            &polygon.exterior().0,
        )))
    });

    let mut world = MultiPolygon::new(polygons);
    world.to_degrees_in_place();
    world
}

impl Coord2d for Coord {
    fn xy(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

#[cfg(test)]
#[path = "./arc_set_tests.rs"]
mod tests;
