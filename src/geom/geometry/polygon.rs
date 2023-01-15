use super::{bbox, Geometry, Ring};
use crate::{
    error::InvalidGeometry, geom::ToCells, CellIndex, LatLng, Resolution,
};
use ahash::{HashSet, HashSetExt};
use geo::{coord, Coord, CoordsIter};
use std::{borrow::Cow, boxed::Box, cmp, collections::VecDeque};

/// A bounded two-dimensional area.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon<'a> {
    exterior: Ring<'a>,
    interiors: Vec<Ring<'a>>,
}

impl<'a> Polygon<'a> {
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
    /// let polygon = Polygon::from_radians(&p)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        polygon: &'a geo::Polygon<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self {
            exterior: Ring::from_radians(Cow::Borrowed(polygon.exterior()))?,
            interiors: polygon
                .interiors()
                .iter()
                .map(Cow::Borrowed)
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
            exterior: Ring::from_radians(Cow::Owned(exterior))?,
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
            exterior: Ring::from_radians(Cow::Owned(exterior))?,
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

    fn contains(&self, coord: Coord<f64>) -> bool {
        self.exterior.contains(coord)
            && !self.interiors.iter().any(|ring| ring.contains(coord))
    }
}

impl From<Polygon<'_>> for geo::Polygon<f64> {
    fn from(value: Polygon<'_>) -> Self {
        Self::new(
            value.exterior.into(),
            value.interiors.into_iter().map(Into::into).collect(),
        )
    }
}

impl<'a> TryFrom<Geometry<'a>> for Polygon<'a> {
    type Error = InvalidGeometry;

    fn try_from(value: Geometry<'a>) -> Result<Self, Self::Error> {
        match value {
            Geometry::Polygon(polygon) => Ok(polygon),
            _ => Err(Self::Error::new("invalid type (polygon expected)")),
        }
    }
}

impl ToCells for Polygon<'_> {
    fn max_cells_count(&self, resolution: Resolution) -> usize {
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

    /// This implementation traces the polygon loop(s) in cartesian space with
    /// hexagons, tests them and their neighbors to be contained by the loop(s),
    /// and then any newly found hexagons are used to test again until no new
    /// hexagons are found.
    fn to_cells(
        &self,
        resolution: Resolution,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        // One of the goals of the polygon_to_cells algorithm is that two
        // adjacent polygons with zero overlap have zero overlapping hexagons.
        // That the hexagons are uniquely assigned. There are a few approaches
        // to take here, such as deciding based on which polygon has the
        // greatest overlapping area of the hexagon, or the most number of
        // contained points on the hexagon (using the center point as a
        // tiebreaker).
        //
        // But if the polygons are convex, both of these more complex algorithms
        // can be reduced down to checking whether or not the center of the
        // hexagon is contained in the polygon, and so this is the approach that
        // this polygon_to_cells algorithm will follow, as it's simpler, faster,
        // and the error for concave polygons is still minimal (only affecting
        // concave shapes on the order of magnitude of the hexagon size or
        // smaller, not impacting larger concave shapes).

        // Get the estimated number of cells and allocate some temporary memory.
        let cell_count = self.max_cells_count(resolution);

        // Set used for dedup.
        let mut seen = HashSet::with_capacity(cell_count);
        // IIUC, the collect is necessary to consume the iterator and release
        // the mutable borrow on `seen`.
        #[allow(clippy::needless_collect)]
        // Compute the initial set of cell, using polygon edges.
        let edge_cells = self
            .interiors()
            .chain(std::iter::once(self.exterior()))
            .flat_map(|ring| get_edge_cells(ring, resolution))
            .filter_map(|cell| seen.insert(cell).then_some(cell))
            .collect::<Vec<_>>();
        seen.clear();

        // Scratchpad memory to store a cell and its immediate neighbors.
        // Cell itself + at most 6 neighbors = 7.
        let mut scratchpad = [0; 7];
        // Expand the initial set with neighbors, because computed edge cells
        // may be just out of the shape (since we use a rough approximation).
        let mut candidates =
            edge_cells
                .into_iter()
                .fold(VecDeque::new(), |mut acc, cell| {
                    add_candidates(cell, &mut acc, &mut seen, &mut scratchpad);
                    acc
                });

        Box::new(std::iter::from_fn(move || {
            while let Some(cell) = candidates.pop_front() {
                let ll = LatLng::from(cell);
                let coord = coord! { x: ll.lng(), y: ll.lat() };
                if self.contains(coord) {
                    add_candidates(
                        cell,
                        &mut candidates,
                        &mut seen,
                        &mut scratchpad,
                    );
                    return Some(cell);
                }
            }
            None
        }))
    }
}

// ----------------------------------------------------------------------------

// Return the cell indexes that traces the ring outline.
fn get_edge_cells(
    ring: &geo::LineString<f64>,
    resolution: Resolution,
) -> impl Iterator<Item = CellIndex> + '_ {
    ring.lines().flat_map(move |line| {
        let count = line_hex_estimate(&line, resolution);

        assert!(count <= 1 << f64::MANTISSA_DIGITS);
        #[allow(clippy::cast_precision_loss)]
        (0..count).map(move |i| {
            let i = i as f64;
            let count = count as f64;
            let lat =
                (line.start.y * (count - i) / count) + (line.end.y * i / count);
            let lng =
                (line.start.x * (count - i) / count) + (line.end.x * i / count);

            let ll = LatLng::new(lat, lng).expect("finite line coordinate");
            ll.to_cell(resolution)
        })
    })
}

// Return the next round of candidates from the given cell.
fn add_candidates(
    cell: CellIndex,
    candidates: &mut VecDeque<CellIndex>,
    seen: &mut HashSet<CellIndex>,
    scratchpad: &mut [u64],
) {
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

    candidates.extend(scratchpad[0..count].iter().filter_map(|candidate| {
        // SAFETY: candidate comes from `ring_disk_*`.
        let index = CellIndex::new_unchecked(*candidate);
        seen.insert(index).then_some(index)
    }));
}

/// Returns an estimated number of hexagons that trace the cartesian-projected
/// line.
fn line_hex_estimate(line: &geo::Line<f64>, resolution: Resolution) -> u64 {
    // Get the area of the pentagon as the maximally-distorted area possible
    const PENT_DIAMETER_KM: [f64; 16] = [
        2_073.7217767817406,
        704.7608418567091,
        274.9288669449002,
        97.35076612345429,
        38.83754315003446,
        13.840222219535644,
        5.539293217388523,
        1.9758095817231678,
        0.7911454555939742,
        0.28223066278338094,
        0.11301706221901277,
        0.040318097800798244,
        0.01614521875314634,
        0.005759716669314975,
        0.0023064582837116258,
        0.0008228164451153948,
    ];
    let pentagon_diameter = PENT_DIAMETER_KM[usize::from(resolution)];

    let origin = LatLng::new(line.start.y, line.start.x)
        .expect("finite line-start coordinate");
    let destination = LatLng::new(line.end.y, line.end.x)
        .expect("finite line-end coordinate");
    let distance = origin.distance_km(destination);

    let dist_ceil = (distance / pentagon_diameter).ceil();
    assert!(dist_ceil.is_finite());

    // Truncate on purpose.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let estimate = dist_ceil as u64;

    cmp::max(estimate, 1)
}
