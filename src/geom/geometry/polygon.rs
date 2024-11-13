use super::{bbox, CellBoundary, Ring};
use crate::{
    error::InvalidGeometry, geom::ContainmentMode, CellIndex, LatLng,
    Resolution,
};
use alloc::{borrow::Cow, vec::Vec};
use core::cmp;
use either::Either;
use geo::{coord, CoordsIter};

#[cfg(feature = "std")]
use ahash::{HashSet, HashSetExt};
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet;

type ContainmentPredicate = fn(polygon: &Polygon, cell: CellIndex) -> bool;

#[cfg(not(feature = "std"))]
type Set<K> = BTreeSet<K>;
#[cfg(feature = "std")]
type Set<K> = HashSet<K>;

/// A bounded two-dimensional area.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    exterior: Ring,
    interiors: Vec<Ring>,
}

impl Polygon {
    // Initialize a new polygon from a `geo::Polygon` whose coordinates are in
    // radians.
    //
    // # Errors
    //
    // [`InvalidGeometry`] if the polygon is invalid (e.g. contains non-finite
    // coordinates).
    pub fn from_radians(
        polygon: geo::Polygon,
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

    // Initialize a new polygon from a `geo::Polygon` whose coordinates are in
    // degrees.
    //
    // # Errors
    //
    // [`InvalidGeometry`] if the polygon is invalid (e.g. contains non-finite
    // coordinates).
    pub fn from_degrees(
        polygon: geo::Polygon,
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

    pub fn max_cells_count(&self, resolution: Resolution) -> usize {
        const POLYGON_TO_CELLS_BUFFER: usize = 12;

        let estimated_count = bbox::hex_estimate(&self.bbox(), resolution);

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

    pub(super) const fn bbox(&self) -> geo::Rect {
        self.exterior.bbox()
    }

    pub(super) fn exterior(&self) -> &geo::LineString {
        self.exterior.geom()
    }

    fn interiors(&self) -> impl Iterator<Item = &geo::LineString> {
        self.interiors.iter().map(Ring::geom)
    }

    // Return the cell indexes that traces the ring outline.
    fn hex_outline(
        &self,
        resolution: Resolution,
        already_seen: &mut Set<CellIndex>,
        scratchpad: &mut [u64],
        contains: ContainmentPredicate,
    ) -> Vec<CellIndex> {
        // IIUC, the collect is necessary to consume the iterator and release
        // the mutable borrow on `already_seen`.
        #[allow(
            clippy::needless_collect,
            reason = "needed because mutable borrow"
        )]
        // Compute the set of cells making the outlines of the polygon.
        let outlines = self
            .interiors()
            .chain(core::iter::once(self.exterior()))
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
        already_seen: &mut Set<CellIndex>,
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

    /// This implementation traces the outlines of the polygon's rings, fill one
    /// layer of internal cells and then propagate inwards until the whole area
    /// is covered.
    ///
    /// Only the outlines and the first inner layer of cells requires
    /// Point-in-Polygon checks, inward propagation doesn't (since we're bounded
    /// by the outlines) which make this approach relatively efficient.
    pub fn into_cells(
        self,
        resolution: Resolution,
        containment: ContainmentMode,
    ) -> impl Iterator<Item = CellIndex> {
        let contains = match containment {
            ContainmentMode::ContainsCentroid => contains_centroid,
            ContainmentMode::ContainsBoundary
            | ContainmentMode::IntersectsBoundary
            | ContainmentMode::Covers => intersects_or_contains,
        };

        // Set used for dedup.
        let mut seen = Set::new();
        // Scratchpad memory to store a cell and its immediate neighbors.
        // Cell itself + at most 6 neighbors = 7.
        let mut scratchpad = [0; 7];

        // First, compute the outline.
        let mut outlines =
            self.hex_outline(resolution, &mut seen, &mut scratchpad, contains);

        if outlines.is_empty() && containment == ContainmentMode::Covers {
            return Either::Left(core::iter::once(
                self.exterior.centroid().to_cell(resolution),
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
        #[cfg(not(feature = "std"))]
        let mut new_seen = Set::new();
        #[cfg(feature = "std")]
        let mut new_seen = Set::with_capacity(seen.len());

        if containment == ContainmentMode::ContainsBoundary {
            outlines.retain(|&cell| {
                !intersects_boundary(&self, &CellBoundary::from(cell))
            });
            candidates.retain(|&cell| {
                !intersects_boundary(&self, &CellBoundary::from(cell))
            });
        }

        // Last step: inward propagation from the outermost layers.
        let inward_propagation = core::iter::from_fn(move || {
            if candidates.is_empty() {
                return None;
            }

            for &cell in &candidates {
                debug_assert!(
                    contains_boundary(&self, &CellBoundary::from(cell)),
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

            core::mem::swap(&mut next_gen, &mut candidates);
            next_gen.clear();

            core::mem::swap(&mut new_seen, &mut seen);
            new_seen.clear();

            Some(curr_gen.into_iter())
        });

        Either::Right(outlines.into_iter().chain(inward_propagation.flatten()))
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
    let boundary = CellBoundary::from(cell);

    intersects_boundary(polygon, &boundary)
        || contains_boundary(polygon, &boundary)
}

fn intersects_boundary(polygon: &Polygon, boundary: &CellBoundary) -> bool {
    fn inner(polygon: &Polygon, boundary: &Ring) -> bool {
        let intersects_envelope = polygon
            .exterior
            .intersects_boundary(Cow::Borrowed(boundary));

        intersects_envelope || {
            polygon
                .interiors
                .iter()
                .any(|ring| ring.intersects_boundary(Cow::Borrowed(boundary)))
        }
    }

    match *boundary {
        CellBoundary::Regular(ref boundary) => inner(polygon, boundary),
        CellBoundary::Transmeridian(ref b1, ref b2) => {
            inner(polygon, b1) || inner(polygon, b2)
        }
    }
}

fn contains_boundary(polygon: &Polygon, boundary: &CellBoundary) -> bool {
    fn inner(polygon: &Polygon, boundary: &Ring) -> bool {
        let within_envelope =
            polygon.exterior.contains_boundary(Cow::Borrowed(boundary));

        within_envelope
            && !polygon.interiors.iter().any(|ring| {
                ring.intersects_boundary(Cow::Borrowed(boundary))
                    || ring.contains_boundary(Cow::Borrowed(boundary))
            })
    }

    match *boundary {
        CellBoundary::Regular(ref boundary) => inner(polygon, boundary),
        CellBoundary::Transmeridian(ref b1, ref b2) => {
            inner(polygon, b1) || inner(polygon, b2)
        }
    }
}

// ----------------------------------------------------------------------------

// Return the cell indexes that traces the ring outline (rough approximation)
fn get_edge_cells(
    ring: &geo::LineString,
    resolution: Resolution,
) -> impl Iterator<Item = CellIndex> + '_ {
    ring.lines()
        .flat_map(move |line @ geo::Line { start, end }| {
            let count = line_hex_estimate(&line, resolution);

            assert!(count <= 1 << f64::MANTISSA_DIGITS);
            #[allow(
                clippy::cast_precision_loss,
                reason = "cannot happen thanks to assert above"
            )]
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
fn line_hex_estimate(line: &geo::Line, resolution: Resolution) -> u64 {
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

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "truncate on purpose"
    )]
    let estimate = dist_ceil as u64;

    cmp::max(estimate, 1)
}
