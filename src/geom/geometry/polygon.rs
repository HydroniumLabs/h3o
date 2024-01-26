use super::{bbox, Geometry, Ring};
use crate::{
    error::InvalidGeometry,
    geom::{ContainmentMode, PolyfillConfig, ToCells},
    CellIndex, LatLng, Resolution,
};
use ahash::{HashSet, HashSetExt};
use geo::{coord, CoordsIter};
use std::{borrow::Cow, boxed::Box, cmp};

type ContainmentPredicate = fn(polygon: &Polygon, cell: CellIndex) -> bool;

/// A bounded two-dimensional area.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    exterior: Ring,
    interiors: Vec<Ring>,
}

impl Polygon {
    /// Initialize a new polygon from a `geo::Polygon` whose coordinates are in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the polygon is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::polygon;
    /// use h3o::geom::Polygon;
    ///
    /// let p: geo::Polygon<f64> = polygon![
    ///     (x: 0.6559997912129759, y: 0.9726707149994819),
    ///     (x: 0.6573835290630796, y: 0.9726707149994819),
    ///     (x: 0.6573835290630796, y: 0.9735034901250053),
    ///     (x: 0.6559997912129759, y: 0.9735034901250053),
    ///     (x: 0.6559997912129759, y: 0.9726707149994819),
    /// ];
    /// let polygon = Polygon::from_radians(p)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        polygon: geo::Polygon<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let (exterior, interiors) = polygon.into_inner();
        Ok(Self {
            exterior: Ring::from_radians(exterior)?,
            interiors: interiors
                .into_iter()
                .map(Ring::from_radians)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Initialize a new polygon from a `geo::Polygon` whose coordinates are in
    /// degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the polygon is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::polygon;
    /// use h3o::geom::Polygon;
    ///
    /// let p: geo::Polygon<f64> = polygon![
    ///     (x: 37.58601939796671, y: 55.72992682544245),
    ///     (x: 37.66530173673016, y: 55.72992682544245),
    ///     (x: 37.66530173673016, y: 55.777641325418415),
    ///     (x: 37.58601939796671, y: 55.777641325418415),
    ///     (x: 37.58601939796671, y: 55.72992682544245),
    /// ];
    /// let polygon = Polygon::from_degrees(p)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        polygon: geo::Polygon<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let (exterior, interiors) = polygon.into_inner();
        Ok(Self {
            exterior: Ring::from_degrees(exterior)?,
            interiors: interiors
                .into_iter()
                .map(Ring::from_degrees)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Initialize a new polygon from a [`geo::Rect`] whose coordinates are in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the rectangle is invalid (e.g. contains
    /// non-finite coordinates).
    pub(super) fn from_rect(
        rect: geo::Rect<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let (exterior, interiors) = rect.to_polygon().into_inner();
        debug_assert!(interiors.is_empty());

        Ok(Self {
            exterior: Ring::from_radians(exterior)?,
            interiors: Vec::new(),
        })
    }

    /// Initialize a new polygon from a [`geo::Triangle`] whose coordinates are
    /// in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the triangle is invalid (e.g. contains on-finite
    /// coordinates).
    pub(super) fn from_triangle(
        triangle: geo::Triangle<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let (exterior, interiors) = triangle.to_polygon().into_inner();
        debug_assert!(interiors.is_empty());

        Ok(Self {
            exterior: Ring::from_radians(exterior)?,
            interiors: Vec::new(),
        })
    }

    pub(super) const fn bbox(&self) -> geo::Rect<f64> {
        self.exterior.bbox()
    }

    pub(super) fn exterior(&self) -> &geo::LineString<f64> {
        self.exterior.geom()
    }

    fn interiors(&self) -> impl Iterator<Item = &geo::LineString<f64>> {
        self.interiors.iter().map(Ring::geom)
    }

    // Return the cell indexes that traces the ring outline.
    fn hex_outline(
        &self,
        resolution: Resolution,
        already_seen: &mut HashSet<CellIndex>,
        scratchpad: &mut [u64],
        contains: ContainmentPredicate,
    ) -> Vec<CellIndex> {
        // IIUC, the collect is necessary to consume the iterator and release
        // the mutable borrow on `already_seen`.
        #[allow(clippy::needless_collect)]
        // Compute the set of cells making the outlines of the polygon.
        let outlines = self
            .interiors()
            .chain(std::iter::once(self.exterior()))
            .flat_map(|ring| get_edge_cells(ring, resolution))
            .filter(|cell| already_seen.insert(*cell))
            .collect::<Vec<_>>();
        // Reset the `already_seen` set: content can't be trusted because we
        // `get_edge_cells` is based on a rough approximation.
        already_seen.clear();

        // Buffer the initial outlines with immediate neighbors, (since we used
        // a rough approximation, some cells from the initial set may be just
        // out of the polygon).
        outlines.into_iter().fold(Vec::new(), |mut acc, cell| {
            let count = neighbors(cell, scratchpad);

            acc.extend(scratchpad[0..count].iter().filter_map(|candidate| {
                // SAFETY: candidate comes from `ring_disk_*`.
                let index = CellIndex::new_unchecked(*candidate);

                already_seen
                    .insert(index)
                    .then_some(index)
                    .and_then(|index| contains(self, index).then_some(index))
            }));

            acc
        })
    }

    // Compute the outermost layer of inner cells.
    //
    // Those are the last ones that requires a PiP check, due to their
    // proximity with the outline.
    fn outermost_inner_cells(
        &self,
        outlines: &[CellIndex],
        already_seen: &mut HashSet<CellIndex>,
        scratchpad: &mut [u64],
        contains: ContainmentPredicate,
    ) -> Vec<CellIndex> {
        outlines.iter().fold(Vec::new(), |mut acc, cell| {
            let count = neighbors(*cell, scratchpad);

            acc.extend(scratchpad[0..count].iter().filter_map(|candidate| {
                // SAFETY: candidate comes from `ring_disk_*`.
                let index = CellIndex::new_unchecked(*candidate);

                already_seen
                    .insert(index)
                    .then_some(index)
                    .and_then(|index| contains(self, index).then_some(index))
            }));
            acc
        })
    }
}

impl From<Polygon> for geo::Polygon<f64> {
    fn from(value: Polygon) -> Self {
        Self::new(
            value.exterior.into(),
            value.interiors.into_iter().map(Into::into).collect(),
        )
    }
}

impl TryFrom<Geometry> for Polygon {
    type Error = InvalidGeometry;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::Polygon(polygon) => Ok(polygon),
            _ => Err(Self::Error::new("invalid type (polygon expected)")),
        }
    }
}

impl ToCells for Polygon {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        const POLYGON_TO_CELLS_BUFFER: usize = 12;

        let estimated_count =
            bbox::hex_estimate(&self.bbox(), config.resolution);

        // This algorithm assumes that the number of vertices is usually less
        // than the number of hexagons, but when it's wrong, this will keep it
        // from failing.
        let vertex_count = self
            .interiors()
            // -1 because the last coord is duplicated to close the ring.
            .fold(self.exterior().coords_count() - 1, |acc, line| {
                acc + line.coords_count() - 1
            });

        // When the polygon is very small, near an icosahedron edge and is an
        // odd resolution, the line tracing needs an extra buffer than the
        // estimator function provides (but beefing that up to cover causes most
        // situations to overallocate memory)
        cmp::max(estimated_count, vertex_count) + POLYGON_TO_CELLS_BUFFER
    }

    /// This implementation traces the outlines of the polygon's rings, fill one
    /// layer of internal cells and then propagate inwards until the whole area
    /// is covered.
    ///
    /// Only the outlines and the first inner layer of cells requires
    /// Point-in-Polygon checks, inward propagation doesn't (since we're bounded
    /// by the outlines) which make this approach relatively efficient.
    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        let contains = match config.containment {
            ContainmentMode::ContainsCentroid => contains_centroid,
            ContainmentMode::ContainsBoundary
            | ContainmentMode::IntersectsBoundary
            | ContainmentMode::Covers => intersects_or_contains,
        };

        // Set used for dedup.
        let mut seen = HashSet::new();
        // Scratchpad memory to store a cell and its immediate neighbors.
        // Cell itself + at most 6 neighbors = 7.
        let mut scratchpad = [0; 7];

        // First, compute the outline.
        let mut outlines = self.hex_outline(
            config.resolution,
            &mut seen,
            &mut scratchpad,
            contains,
        );

        if outlines.is_empty() && config.containment == ContainmentMode::Covers
        {
            return Box::new(std::iter::once(
                self.exterior.centroid().to_cell(config.resolution),
            ));
        }

        // Next, compute the outermost layer of inner cells to seed the
        // propagation step.
        let mut candidates = self.outermost_inner_cells(
            &outlines,
            &mut seen,
            &mut scratchpad,
            contains,
        );
        let mut next_gen = Vec::with_capacity(candidates.len() * 7);
        let mut new_seen = HashSet::with_capacity(seen.len());

        if config.containment == ContainmentMode::ContainsBoundary {
            outlines.retain(|&cell| !intersects_boundary(self, cell));
            candidates.retain(|&cell| !intersects_boundary(self, cell));
        }

        // Last step: inward propagation from the outermost layers.
        let inward_propagation = std::iter::from_fn(move || {
            if candidates.is_empty() {
                return None;
            }

            for &cell in &candidates {
                debug_assert!(
                    contains_boundary(self, cell),
                    "cell index {cell} in polygon"
                );

                let count = neighbors(cell, &mut scratchpad);
                next_gen.extend(scratchpad[0..count].iter().filter_map(
                    |candidate| {
                        // SAFETY: candidate comes from `ring_disk_*`.
                        let index = CellIndex::new_unchecked(*candidate);
                        new_seen.insert(index);
                        seen.insert(index).then_some(index)
                    },
                ));
            }

            let curr_gen = candidates.clone();

            std::mem::swap(&mut next_gen, &mut candidates);
            next_gen.clear();

            std::mem::swap(&mut new_seen, &mut seen);
            new_seen.clear();

            Some(curr_gen.into_iter())
        });

        Box::new(outlines.into_iter().chain(inward_propagation.flatten()))
    }
}

// ----------------------------------------------------------------------------

fn contains_centroid(polygon: &Polygon, cell: CellIndex) -> bool {
    let ll = LatLng::from(cell);
    let coord = coord! { x: ll.lng_radians(), y: ll.lat_radians() };

    polygon.exterior.contains_centroid(coord)
        && !polygon
            .interiors
            .iter()
            .any(|ring| ring.contains_centroid(coord))
}

fn intersects_or_contains(polygon: &Polygon, cell: CellIndex) -> bool {
    intersects_boundary(polygon, cell) || contains_boundary(polygon, cell)
}

fn intersects_boundary(polygon: &Polygon, cell: CellIndex) -> bool {
    let boundary = compute_cell_ring(cell);

    let intersects_enveloppe = polygon
        .exterior
        .intersects_boundary(Cow::Borrowed(&boundary));

    intersects_enveloppe || {
        polygon
            .interiors
            .iter()
            .any(|ring| ring.intersects_boundary(Cow::Borrowed(&boundary)))
    }
}

fn contains_boundary(polygon: &Polygon, cell: CellIndex) -> bool {
    let boundary = compute_cell_ring(cell);

    let within_enveloppe =
        polygon.exterior.contains_boundary(Cow::Borrowed(&boundary));

    within_enveloppe
        && !polygon.interiors.iter().any(|ring| {
            ring.intersects_boundary(Cow::Borrowed(&boundary))
                || ring.contains_boundary(Cow::Borrowed(&boundary))
        })
}

// ----------------------------------------------------------------------------

// Return the cell indexes that traces the ring outline (rough approximation)
fn get_edge_cells(
    ring: &geo::LineString<f64>,
    resolution: Resolution,
) -> impl Iterator<Item = CellIndex> + '_ {
    ring.lines()
        .flat_map(move |line @ geo::Line { start, end }| {
            let count = line_hex_estimate(&line, resolution);

            assert!(count <= 1 << f64::MANTISSA_DIGITS);
            #[allow(clippy::cast_precision_loss)]
            // Nope thanks to assert above.
            (0..count).map(move |i| {
                let i = i as f64;
                let count = count as f64;

                let lat = (start.y * (count - i) / count) + (end.y * i / count);
                let lng = (start.x * (count - i) / count) + (end.x * i / count);

                LatLng::from_radians(lat, lng)
                    .expect("finite line coordinate")
                    .to_cell(resolution)
            })
        })
}

// Return the immediate neighbors.
fn neighbors(cell: CellIndex, scratchpad: &mut [u64]) -> usize {
    let mut count = 0;

    // Don't use `grid_disk` to avoid the allocation,
    // use the pre-allocated scratchpad memory instead.
    for candidate in cell.grid_disk_fast(1) {
        if let Some(neighbor) = candidate {
            scratchpad[count] = neighbor.into();
            count += 1;
        } else {
            count = 0;
            break;
        }
    }

    // Unsafe version failed, fallback on the safe version.
    if count == 0 {
        for candidate in cell.grid_disk_safe(1) {
            scratchpad[count] = candidate.into();
            count += 1;
        }
    }

    count
}

/// Returns an estimated number of hexagons that trace the cartesian-projected
/// line.
fn line_hex_estimate(line: &geo::Line<f64>, resolution: Resolution) -> u64 {
    // Get the area of the pentagon as the maximally-distorted area possible
    const PENT_DIAMETER_RADS: [f64; 16] = [
        0.32549355508382627,
        0.11062000431697926,
        0.0431531246375496,
        0.015280278825461551,
        0.006095981694441515,
        0.00217237586248339,
        0.0008694532999397082,
        0.0003101251537809772,
        0.00012417902430910614,
        0.00004429922220615181,
        0.00001773927716796858,
        0.000006328371112691009,
        0.0000025341705472716865,
        0.0000009040511973807097,
        0.00000036202412300873475,
        0.00000012915013523209886,
    ];
    let pentagon_diameter = PENT_DIAMETER_RADS[usize::from(resolution)];

    let origin = LatLng::from_radians(line.start.y, line.start.x)
        .expect("finite line-start coordinate");
    let destination = LatLng::from_radians(line.end.y, line.end.x)
        .expect("finite line-end coordinate");
    let distance = origin.distance_rads(destination);

    let dist_ceil = (distance / pentagon_diameter).ceil();
    assert!(dist_ceil.is_finite());

    // Truncate on purpose.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let estimate = dist_ceil as u64;

    cmp::max(estimate, 1)
}

/// Returns the ring corresponding to the cell's boundary.
fn compute_cell_ring(cell: CellIndex) -> Ring {
    let mut boundary = geo::LineString(
        cell.boundary()
            .iter()
            .copied()
            .map(|ll| coord! { x: ll.lng_radians(), y: ll.lat_radians() })
            .collect(),
    );
    boundary.close();
    Ring::from_radians(boundary).expect("cell boundary is a valiud geometry")
}
