use crate::{error::InvalidGeometry, CellIndex, LatLng, Resolution, TWO_PI};
use ahash::{HashSet, HashSetExt};
use either::Either;
use float_eq::float_eq;
use geo::{
    algorithm::{
        coordinate_position::{coord_pos_relative_to_ring, CoordPos},
        relate::PreparedGeometry,
    },
    coord, BooleanOps as _, BoundingRect as _, Centroid as _, Coord,
    CoordsIter as _, Intersects, Line, LineString, MultiPolygon, Polygon, Rect,
    Relate as _, ToRadians as _,
};
use std::{
    cmp,
    f64::consts::{FRAC_PI_2, PI},
};

/// A tiler that produces an H3 coverage of the given shapes.
#[derive(Debug, Clone)]
pub struct Tiler {
    resolution: Resolution,
    containment_mode: ContainmentMode,
    convert_to_rads: bool,
    transmeridian_heuristic_enabled: bool,
    geom: MultiPolygon,
}

impl Tiler {
    /// Adds a `Polygon` to tile.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the polygon is invalid.
    pub fn add(&mut self, mut polygon: Polygon) -> Result<(), InvalidGeometry> {
        // Convert to radians if necessary.
        if self.convert_to_rads {
            polygon.to_radians_in_place();
        }

        // Check coordinates validity.
        ring_is_valid(polygon.exterior())?;
        for interior in polygon.interiors() {
            ring_is_valid(interior)?;
        }

        // Identify and fix transmeridian polygon if necessary.
        if self.transmeridian_heuristic_enabled && is_transmeridian(&polygon) {
            for fixed_polygon in fix_transmeridian(polygon).0 {
                self.geom.0.push(fixed_polygon);
            }
        } else {
            self.geom.0.push(polygon);
        }

        Ok(())
    }

    /// Adds a batch of `Polygon` to tile.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the polygon is invalid.
    pub fn add_batch(
        &mut self,
        geoms: impl IntoIterator<Item = Polygon>,
    ) -> Result<(), InvalidGeometry> {
        for polygon in geoms {
            self.add(polygon)?;
        }
        Ok(())
    }

    /// Returns an upper bound to the number of cells returned by `into_coverage`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use geo::{LineString, Polygon};
    /// use h3o::{geom::{ContainmentMode, TilerBuilder}, Resolution};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    /// let mut tiler = TilerBuilder::new(Resolution::Ten)
    ///     .containment_mode(ContainmentMode::Covers)
    ///     .build();
    /// tiler.add(polygon)?;
    ///
    /// let size_hint = tiler.coverage_size_hint();
    ///
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    #[must_use]
    pub fn coverage_size_hint(&self) -> usize {
        const POLYGON_TO_CELLS_BUFFER: usize = 12;

        self.geom
            .iter()
            .map(|polygon| {
                let estimated_count = bbox_hex_estimate(
                    &polygon.bounding_rect().expect("valid polygon"),
                    self.resolution,
                );

                // This algorithm assumes that the number of vertices is usually less
                // than the number of hexagons, but when it's wrong, this will keep it
                // from failing.
                let vertex_count = polygon
                    .interiors()
                    .iter()
                    .chain(std::iter::once(polygon.exterior()))
                    // -1 because the last coord is duplicated to close the ring.
                    .map(|line| line.coords_count() - 1)
                    .sum();

                // When the polygon is very small, near an icosahedron edge and is an
                // odd resolution, the line tracing needs an extra buffer than the
                // estimator function provides (but beefing that up to cover causes most
                // situations to overallocate memory)
                cmp::max(estimated_count, vertex_count)
                    + POLYGON_TO_CELLS_BUFFER
            })
            .sum()
    }

    /// Computes the cell coverage of the geometries.
    ///
    /// The output may contain duplicate indexes in case of overlapping input
    /// geometries/depending on the selected containment mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use geo::{LineString, Polygon};
    /// use h3o::{geom::{ContainmentMode, TilerBuilder}, Resolution};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    /// let mut tiler = TilerBuilder::new(Resolution::Ten)
    ///     .containment_mode(ContainmentMode::Covers)
    ///     .build();
    /// tiler.add(polygon)?;
    ///
    /// let cells = tiler.into_coverage().collect::<Vec<_>>();
    ///
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn into_coverage(self) -> impl Iterator<Item = CellIndex> {
        // This implementation traces the outlines of the polygon's rings, fill one
        // layer of internal cells and then propagate inwards until the whole area
        // is covered.
        //
        // Only the outlines and the first inner layer of cells requires
        // Point-in-Polygon checks, inward propagation doesn't (since we're bounded
        // by the outlines) which make this approach relatively efficient.

        let predicate =
            ContainmentPredicate::new(&self.geom, self.containment_mode);
        // Set used for dedup.
        let mut seen = HashSet::new();
        // Scratchpad memory to store a cell and its immediate neighbors.
        // Cell itself + at most 6 neighbors = 7.
        let mut scratchpad = [0; 7];

        // First, compute the outline.
        let mut outlines = self.hex_outline(
            self.resolution,
            &mut seen,
            &mut scratchpad,
            &predicate,
        );

        if outlines.is_empty()
            && self.containment_mode == ContainmentMode::Covers
        {
            let centroid = self.geom.centroid().expect("centroid");
            return Either::Left(std::iter::once(
                LatLng::from_radians(centroid.y(), centroid.x())
                    .expect("valid coordinate")
                    .to_cell(self.resolution),
            ));
        }

        // Next, compute the outermost layer of inner cells to seed the
        // propagation step.
        let mut candidates = outermost_inner_cells(
            &outlines,
            &mut seen,
            &mut scratchpad,
            &predicate,
        );
        let mut next_gen = Vec::with_capacity(candidates.len() * 7);
        let mut new_seen = HashSet::with_capacity(seen.len());

        if self.containment_mode == ContainmentMode::ContainsBoundary {
            outlines.retain(|&(_, is_fully_contained)| is_fully_contained);
            candidates.retain(|&(_, is_fully_contained)| is_fully_contained);
        }

        // Last step: inward propagation from the outermost layers.
        let inward_propagation = std::iter::from_fn(move || {
            if candidates.is_empty() {
                return None;
            }

            for &(cell, _) in &candidates {
                debug_assert!(
                    self.geom.relate(&cell_boundary(cell)).is_covers(),
                    "cell index {cell} in polygon"
                );

                let count = neighbors(cell, &mut scratchpad);
                next_gen.extend(scratchpad[0..count].iter().filter_map(
                    |candidate| {
                        // SAFETY: candidate comes from `ring_disk_*`.
                        let index = CellIndex::new_unchecked(*candidate);
                        new_seen.insert(index);
                        seen.insert(index).then_some((index, true))
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

        Either::Right(
            outlines
                .into_iter()
                .chain(inward_propagation.flatten())
                .map(|(cell, _)| cell),
        )
    }

    // Return the cell indexes that traces the ring outline.
    fn hex_outline(
        &self,
        resolution: Resolution,
        already_seen: &mut HashSet<CellIndex>,
        scratchpad: &mut [u64],
        predicate: &ContainmentPredicate<'_>,
    ) -> Vec<(CellIndex, bool)> {
        // IIUC, the collect is necessary to consume the iterator and release
        // the mutable borrow on `already_seen`.
        #[allow(
            clippy::needless_collect,
            reason = "needed because mutable borrow"
        )]
        // Compute the set of cells making the outlines of the polygon.
        let outlines = self
            .interiors()
            .chain(self.exteriors())
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
                    .and_then(|index| {
                        let result = predicate.apply(index);
                        result
                            .is_a_match
                            .then_some((index, result.is_fully_contained))
                    })
            }));

            acc
        })
    }

    /// Returns the exterior rings of each polygon.
    fn exteriors(&self) -> impl Iterator<Item = &LineString> {
        self.geom.0.iter().map(Polygon::exterior)
    }

    /// Returns the interior rings of each polygon.
    fn interiors(&self) -> impl Iterator<Item = &LineString> {
        self.geom
            .0
            .iter()
            .flat_map(|polygon| polygon.interiors().iter())
    }
}

// -----------------------------------------------------------------------------

/// A builder to configure a tiler.
pub struct TilerBuilder {
    resolution: Resolution,
    containment_mode: ContainmentMode,
    convert_to_rads: bool,
    transmeridian_heuristic_enabled: bool,
}

impl TilerBuilder {
    /// Initializes a new plotter builder with default settings.
    #[must_use]
    pub const fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            containment_mode: ContainmentMode::ContainsCentroid,
            convert_to_rads: true,
            transmeridian_heuristic_enabled: true,
        }
    }

    /// Disable the degrees-to-radians conversion pre-processing.
    #[must_use]
    pub const fn disable_radians_conversion(mut self) -> Self {
        self.convert_to_rads = false;
        self
    }

    /// Set the containment mode defining if a cell is in a polygon or not.
    #[must_use]
    pub const fn containment_mode(mut self, mode: ContainmentMode) -> Self {
        self.containment_mode = mode;
        self
    }

    /// Disable the transmeridian heuristic.
    ///
    /// It's safe to do if you are sure that none of the input polygon are
    /// spread over the 180th meridian.
    /// It's necessary to disable if you have a non-transmeridian polygon with
    /// an arc greater than 180° (which would trip the heuristic).
    #[must_use]
    pub const fn disable_transmeridian_heuristic(mut self) -> Self {
        self.transmeridian_heuristic_enabled = false;
        self
    }

    /// Builds the plotter.
    #[must_use]
    pub fn build(self) -> Tiler {
        Tiler {
            resolution: self.resolution,
            containment_mode: self.containment_mode,
            convert_to_rads: self.convert_to_rads,
            transmeridian_heuristic_enabled: self
                .transmeridian_heuristic_enabled,
            geom: MultiPolygon::new(Vec::new()),
        }
    }
}

// -----------------------------------------------------------------------------

/// Containment mode used to decide if a cell is contained in a polygon or not.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainmentMode {
    /// This mode will select every cells whose centroid are contained inside
    /// the polygon.
    ///
    /// This is the fasted option and ensures that every cell is uniquely
    /// assigned (e.g. two adjacent polygon with zero overlap also have zero
    /// overlapping cells).
    ///
    /// On the other hand, some cells may cover area outside of the polygon
    /// (overshooting) and some parts of the polygon may be left uncovered.
    ContainsCentroid,

    /// This mode will select every cells whose boundaries are entirely within
    /// the polygon.
    ///
    /// This ensures that every cell is uniquely assigned  (e.g. two adjacent
    /// polygon with zero overlap also have zero overlapping cells) and avoids
    /// any coverage overshooting.
    ///
    /// Some parts of the polygon may be left uncovered (more than with
    /// `ContainsCentroid`).
    ContainsBoundary,

    /// This mode will select every cells whose boundaries are within the
    /// polygon, even partially.
    ///
    /// This guarantees a complete coverage of the polygon, but some cells may
    /// belong to two different polygons if they are adjacent/close enough. Some
    /// cells may cover area outside of the polygon.
    ///
    /// Note that if the geometry is fully contained within a cell, this mode
    /// returns nothing (because there are no boundaries intersection).
    IntersectsBoundary,

    /// This mode behaves the same as `IntersectsBoundary`, but also handles the
    /// case where the geometry is being covered by a cell without intersecting
    /// with its boundaries. In such cases, the covering cell is returned.
    Covers,
}

/// The result of a predicate application.
struct PredicateResult {
    /// Whether the cell is a match for this predicate.
    is_a_match: bool,
    /// Whether the cell is fully contained by the geometry.
    ///
    /// This is especially useful to quickly prune intersecting cells in
    /// Contains mode.
    is_fully_contained: bool,
}

enum ContainmentPredicate<'geom> {
    ContainsCentroid(&'geom MultiPolygon, MultiBBoxes),
    IntersectsBoundary(PreparedGeometry<'geom>),
}

impl<'geom> ContainmentPredicate<'geom> {
    /// Initializes a new containment predicate according to the mode selected.
    fn new(
        geom: &'geom MultiPolygon,
        containment_mode: ContainmentMode,
    ) -> Self {
        match containment_mode {
            // For this one we can use our good ol' PIP-based approach.
            ContainmentMode::ContainsCentroid => {
                // Pre-compute the bounding boxes for each ring.
                let bboxes = MultiBBoxes(
                    geom.iter()
                        .map(|polygon| BBoxes {
                            exterior: polygon
                                .exterior()
                                .bounding_rect()
                                .expect("exterior bbox"),
                            interiors: polygon
                                .interiors()
                                .iter()
                                .map(|ring| {
                                    ring.bounding_rect().expect("interior bbox")
                                })
                                .collect(),
                        })
                        .collect(),
                );

                Self::ContainsCentroid(geom, bboxes)
            }
            // For the others, using a related-based approach boosted by a
            // PreparedGeometry is the way to go.
            ContainmentMode::ContainsBoundary
            | ContainmentMode::IntersectsBoundary
            | ContainmentMode::Covers => {
                let prepared_geom = PreparedGeometry::from(geom);
                Self::IntersectsBoundary(prepared_geom)
            }
        }
    }

    /// Applies the predicate with the given cell.
    fn apply(&self, cell: CellIndex) -> PredicateResult {
        match self {
            Self::ContainsCentroid(geom, bboxes) => {
                let ll = LatLng::from(cell);
                let coord = coord! { x: ll.lng_radians(), y: ll.lat_radians() };

                let is_a_match =
                    geom.iter().enumerate().any(|(poly_idx, polygon)| {
                        ring_contains_centroid(
                            polygon.exterior(),
                            &bboxes.0[poly_idx].exterior,
                            coord,
                        ) && !polygon.interiors().iter().enumerate().any(
                            |(ring_idx, ring)| {
                                ring_contains_centroid(
                                    ring,
                                    &bboxes.0[poly_idx].interiors[ring_idx],
                                    coord,
                                )
                            },
                        )
                    });

                PredicateResult {
                    is_a_match,
                    is_fully_contained: true,
                }
            }
            Self::IntersectsBoundary(geom) => {
                let boundary = cell_boundary(cell);
                let relation = geom.relate(&boundary);

                PredicateResult {
                    is_a_match: relation.is_intersects(),
                    is_fully_contained: relation.is_covers(),
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------

// Compute the outermost layer of inner cells.
//
// Those are the last ones that requires a PiP check, due to their
// proximity with the outline.
fn outermost_inner_cells(
    outlines: &[(CellIndex, bool)],
    already_seen: &mut HashSet<CellIndex>,
    scratchpad: &mut [u64],
    predicate: &ContainmentPredicate<'_>,
) -> Vec<(CellIndex, bool)> {
    outlines.iter().fold(Vec::new(), |mut acc, &(cell, _)| {
        let count = neighbors(cell, scratchpad);

        acc.extend(scratchpad[0..count].iter().filter_map(|candidate| {
            // SAFETY: candidate comes from `ring_disk_*`.
            let index = CellIndex::new_unchecked(*candidate);

            already_seen
                .insert(index)
                .then_some(index)
                .and_then(|index| {
                    let result = predicate.apply(index);
                    result
                        .is_a_match
                        .then_some((index, result.is_fully_contained))
                })
        }));
        acc
    })
}

// Return the cell indexes that traces the ring outline (rough approximation)
fn get_edge_cells(
    ring: &LineString,
    resolution: Resolution,
) -> impl Iterator<Item = CellIndex> + '_ {
    ring.lines().flat_map(move |line @ Line { start, end }| {
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
fn line_hex_estimate(line: &Line, resolution: Resolution) -> u64 {
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

/// Returns an estimated number of hexagons that fit within the
/// cartesian-projected bounding box.
pub fn bbox_hex_estimate(bbox: &Rect, resolution: Resolution) -> usize {
    // Area of a regular hexagon is `3/2*sqrt(3) * r * r`.
    //
    // The pentagon has the most distortion (smallest edges) and shares its
    // edges with hexagons, so the most-distorted hexagons have this area,
    // shrunk by 20% off chance that the bounding box perfectly bounds a
    // pentagon.
    const PENT_AREA_RADS2: [f64; 16] = [
        0.05505118472518226,
        0.006358420186890303,
        0.0009676234334810151,
        0.00012132336301389888,
        0.000019309418286620768,
        0.0000024521770265310696,
        0.0000003928026439666205,
        0.00000004997535264470275,
        0.000000008012690511075445,
        0.0000000010197039091132572,
        0.00000000016351353999538285,
        0.000000000020809697203105007,
        0.000000000003336979666606075,
        0.0000000000004246859893033221,
        0.00000000000006810153522091642,
        0.000000000000008667056198238203,
    ];
    let pentagon_area_rads2 = PENT_AREA_RADS2[usize::from(resolution)];

    let min = bbox.min();
    let max = bbox.max();
    let p1 =
        LatLng::from_radians(min.y, min.x).expect("finite bbox-min coordinate");
    let p2 =
        LatLng::from_radians(max.y, max.x).expect("finite bbox-max coordinate");
    let diagonal = p1.distance_rads(p2);
    let d1 = (p1.lng_radians() - p2.lng_radians()).abs();
    let d2 = (p1.lat_radians() - p2.lat_radians()).abs();
    let (width, length) = if d1 < d2 { (d1, d2) } else { (d2, d1) };
    // Derived constant based on: https://math.stackexchange.com/a/1921940
    // Clamped to 3 as higher values tend to rapidly drag the estimate to zero.
    #[allow(clippy::suspicious_operation_groupings, reason = "false positive")]
    let area = (diagonal * diagonal) / (length / width);

    // Divide the two to get an estimate of the number of hexagons needed.
    let estimate = (area / pentagon_area_rads2).ceil();

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "truncate on purpose"
    )]
    let estimate = estimate as usize;

    cmp::max(estimate, 1)
}

// -----------------------------------------------------------------------------

// Check for arcs > 180 degrees (π radians) longitude to flag as transmeridian.
fn is_transmeridian(geom: &Polygon) -> bool {
    geom.exterior()
        .lines()
        .any(|line| (line.start.x - line.end.x).abs() > PI)
}

// Fix a transmeridian polygon by splitting it into multiple polygons that are
// on either side.
fn fix_transmeridian(mut polygon: Polygon) -> MultiPolygon {
    let west = Rect::new(
        coord! { x: PI, y: -FRAC_PI_2},
        coord! { x: TWO_PI, y: FRAC_PI_2},
    )
    .to_polygon();
    let east = Rect::new(
        coord! { x: 0., y: -FRAC_PI_2},
        coord! { x: PI, y: FRAC_PI_2},
    )
    .to_polygon();

    shift_transmeridian(&mut polygon);
    let mut fixed = polygon.intersection(&west);
    unshift_transmeridian(&mut fixed);
    fix_clipping_boundary(&mut fixed, true);

    let mut other = polygon.intersection(&east);
    fix_clipping_boundary(&mut other, false);
    fixed.0.extend(other.0);

    fixed
}

/// Shift the coordinates of a polygon to the right of the 180th meridian.
fn shift_transmeridian(geom: &mut Polygon) {
    geom.exterior_mut(shift_transmeridian_ring);
    geom.interiors_mut(|interiors| {
        for interior in interiors {
            shift_transmeridian_ring(interior);
        }
    });
}

/// Unshift the coordinates of a shifted polygon.
fn unshift_transmeridian(geom: &mut MultiPolygon) {
    for polygon in geom.iter_mut() {
        polygon.exterior_mut(unshift_transmeridian_ring);
        polygon.interiors_mut(|interiors| {
            for interior in interiors {
                unshift_transmeridian_ring(interior);
            }
        });
    }
}

// Fix clipping boundary to be robust against rounding errors/imprecisions.
fn fix_clipping_boundary(geom: &mut MultiPolygon, is_west: bool) {
    for polygon in geom.iter_mut() {
        polygon.exterior_mut(|exterior| {
            fix_ring_clipping_boundary(exterior, is_west);
        });
        polygon.interiors_mut(|interiors| {
            for interior in interiors {
                fix_ring_clipping_boundary(interior, is_west);
            }
        });
    }
}

// Check that a polygon ring is valid.
pub fn ring_is_valid(ring: &LineString) -> Result<(), InvalidGeometry> {
    // Closed ring have at least 4 coordinate (e.g. triangle).
    if ring.0.len() < 4 {
        return Err(InvalidGeometry::new(
            "invalid ring (not enough coordinate)",
        ));
    }
    if !ring.coords().all(|coord| super::coord_is_valid(*coord)) {
        return Err(InvalidGeometry::new(
            "every coordinate of the exterior must be valid",
        ));
    }

    Ok(())
}

/// Shift the coordinates of a ring to the right of the 180th meridian.
fn shift_transmeridian_ring(ring: &mut LineString) {
    for coord in ring.coords_mut() {
        coord.x += f64::from(coord.x < 0.) * TWO_PI;
    }
}

/// Unshift the coordinates of a shifted ring.
fn unshift_transmeridian_ring(ring: &mut LineString) {
    for coord in ring.coords_mut() {
        coord.x -= f64::from(coord.x >= PI) * TWO_PI;
    }
}

// Fix points coordinates on the clipping boundary.
//
// Even though we clip at exactly -180/180°, due to rounding error the value
// after clipping might be slightly different which can be a problem when
// computing the intersection matrix.
fn fix_ring_clipping_boundary(ring: &mut LineString, is_west: bool) {
    const ROUNDING_EPSILON: f64 = 1e-6;
    let (bad_value, fixed_value) = if is_west {
        let mut bad_value = PI;
        for coord in ring.coords() {
            if float_eq!(coord.x, PI, abs <= ROUNDING_EPSILON) {
                bad_value = coord.x;
                break;
            }
            bad_value = bad_value.min(coord.x);
        }
        (bad_value, -PI)
    } else {
        let mut bad_value = -PI;
        for coord in ring.coords() {
            if float_eq!(coord.x, -PI, abs <= ROUNDING_EPSILON) {
                bad_value = coord.x;
                break;
            }
            bad_value = bad_value.max(coord.x);
        }
        (bad_value, PI)
    };

    #[expect(clippy::float_cmp, reason = "we want exact equality")]
    for coord in ring.coords_mut() {
        if coord.x == bad_value {
            coord.x = fixed_value;
        }
    }
}

// The independant bounding boxes for a MultiPolygon.
struct MultiBBoxes(Vec<BBoxes>);

// The independant bounding boxes for a Polygon.
struct BBoxes {
    exterior: Rect,
    interiors: Vec<Rect>,
}

// Simple Point-in-Polygon check for a ring.
fn ring_contains_centroid(
    ring: &LineString,
    bbox: &Rect,
    mut coord: Coord,
) -> bool {
    if !bbox.intersects(&coord) {
        return false;
    }

    match coord_pos_relative_to_ring(coord, ring) {
        CoordPos::Inside => true,
        CoordPos::Outside => false,
        CoordPos::OnBoundary => {
            // If the centroid lies on a boundary, it could belong to two
            // polygons.
            // To avoid this, adjust the latitude northward.
            //
            // NOTE: This currently means that a point at the north pole cannot
            // be contained in any polygon. This is acceptable in current usage,
            // because the point we test in this function at present is always a
            // cell center or vertex, and no cell has a center or vertex on the
            // north pole. If we need to expand this algo to more generic uses
            // we might need to handle this edge case.
            coord.y += f64::EPSILON;
            coord_pos_relative_to_ring(coord, ring) == CoordPos::Inside
        }
    }
}

// Return the cell boundary, in radians.
fn cell_boundary(cell: CellIndex) -> MultiPolygon {
    let boundary = LineString(
        cell.boundary()
            .iter()
            .copied()
            .map(|ll| {
                coord! {
                    x: ll.lng_radians(),
                    y: ll.lat_radians()
                }
            })
            .collect(),
    );
    let polygon = Polygon::new(boundary, Vec::new());
    if is_transmeridian(&polygon) {
        fix_transmeridian(polygon)
    } else {
        MultiPolygon::new(vec![polygon])
    }
}
