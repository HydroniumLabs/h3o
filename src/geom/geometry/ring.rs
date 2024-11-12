use super::bbox;
use crate::{error::InvalidGeometry, CellIndex, LatLng, TWO_PI};
use alloc::{borrow::Cow, vec::Vec};
use core::f64::consts::PI;
use geo::{
    algorithm::{
        centroid::Centroid,
        coordinate_position::{coord_pos_relative_to_ring, CoordPos},
    },
    coord,
    sweep::Intersections,
    Contains, Coord, Intersects, Polygon,
};

/// A closed ring and its bounding box.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    geom: Polygon,
    bbox: geo::Rect,
    is_transmeridian: bool,
}

impl Ring {
    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in radians.
    pub fn from_radians(
        mut ring: geo::LineString,
    ) -> Result<Self, InvalidGeometry> {
        let is_transmeridian = fix_transmeridian(&mut ring);
        let bbox = bbox::compute_from_ring(&ring)?;

        Ok(Self {
            geom: Polygon::new(ring, Vec::new()),
            bbox,
            is_transmeridian,
        })
    }

    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in degrees.
    pub fn from_degrees(
        mut ring: geo::LineString,
    ) -> Result<Self, InvalidGeometry> {
        for coord in ring.coords_mut() {
            coord.x = coord.x.to_radians();
            coord.y = coord.y.to_radians();
        }
        Self::from_radians(ring)
    }

    /// Initialize a new ring from a cell boundary.
    ///
    /// The transmeridian check has already been done.
    fn from_cell_boundary(
        ring: geo::LineString,
        is_transmeridian: bool,
    ) -> Self {
        let bbox = bbox::compute_from_ring(&ring)
            .expect("cell boundary is a valid geometry");

        Self {
            geom: Polygon::new(ring, Vec::new()),
            bbox,
            is_transmeridian,
        }
    }

    pub fn geom(&self) -> &geo::LineString {
        self.geom.exterior()
    }

    pub fn centroid(&self) -> LatLng {
        let centroid = self.geom.centroid().expect("centroid");

        LatLng::from_radians(centroid.y(), centroid.x())
            .expect("valid coordinate")
    }

    pub const fn bbox(&self) -> geo::Rect {
        self.bbox
    }

    pub fn contains_centroid(&self, mut coord: Coord) -> bool {
        if self.is_transmeridian {
            coord.x += f64::from(coord.x < 0.) * TWO_PI;
        }

        if !self.bbox.intersects(&coord) {
            return false;
        }

        match coord_pos_relative_to_ring(coord, self.geom.exterior()) {
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
                coord_pos_relative_to_ring(coord, self.geom.exterior())
                    == CoordPos::Inside
            }
        }
    }

    pub fn intersects_boundary(
        &self,
        mut cell_boundary: Cow<'_, Self>,
    ) -> bool {
        self.fixup_cell_boundary(&mut cell_boundary);

        if !self.bbox.intersects(&cell_boundary.bbox) {
            return false;
        }

        cell_boundary
            .geom
            .exterior()
            .lines()
            .any(|line| line.intersects(self.geom.exterior()))
    }

    pub fn contains_boundary(&self, mut cell_boundary: Cow<'_, Self>) -> bool {
        self.fixup_cell_boundary(&mut cell_boundary);

        if !self.bbox.contains(&cell_boundary.bbox) {
            return false;
        }

        self.geom.contains(&cell_boundary.geom)
    }

    /// If the cell is not transmeridian but the ring is, we need to translate
    /// the cell boundaries.
    fn fixup_cell_boundary(&self, cell_boundary: &mut Cow<'_, Self>) {
        if self.is_transmeridian && !cell_boundary.is_transmeridian {
            cell_boundary.to_mut().geom.exterior_mut(|boundary| {
                for coord in boundary.coords_mut() {
                    coord.x += f64::from(coord.x < 0.) * TWO_PI;
                }
            });
            // Don't forget to fixup the pre-computed bbox as well!
            cell_boundary.to_mut().bbox =
                bbox::compute_from_ring(cell_boundary.geom.exterior())
                    .expect("cell boundary is a valid geometry");
        }
    }
}

// -----------------------------------------------------------------------------

// The boundary of a H3 cell.
//
// When the cell cross over the antimeridian, it is represented by two
// projections, one westward and the other eastward.
pub enum CellBoundary {
    Regular(Ring),
    Transmeridian(Ring, Ring),
}

impl From<CellIndex> for CellBoundary {
    fn from(value: CellIndex) -> Self {
        let mut boundary = geo::LineString(
            value
                .boundary()
                .iter()
                .copied()
                .map(|ll| coord! { x: ll.lng_radians(), y: ll.lat_radians() })
                .collect(),
        );
        boundary.close();

        if is_transmeridian(&boundary) {
            let mut fixed_east = boundary.clone();
            for coord in fixed_east.coords_mut() {
                coord.x += f64::from(coord.x < 0.) * TWO_PI;
            }
            let mut fixed_west = boundary;
            for coord in fixed_west.coords_mut() {
                coord.x -= f64::from(coord.x > 0.) * TWO_PI;
            }
            Self::Transmeridian(
                Ring::from_cell_boundary(fixed_east, true),
                Ring::from_cell_boundary(fixed_west, true),
            )
        } else {
            Self::Regular(Ring::from_cell_boundary(boundary, false))
        }
    }
}

// -----------------------------------------------------------------------------

// Check for arcs > 180 degrees (π radians) longitude to flag as transmeridian
// and fix the shape accordingly.
fn fix_transmeridian(ring: &mut geo::LineString) -> bool {
    let is_transmeridian = is_transmeridian(ring);

    // The heuristic above can be fooled by geometries with very long arcs.
    // Make sure that the "corrected" geometry is not self-intersecting.
    // This is not bullet-proof but will catch a bunch a false positives.
    if is_transmeridian {
        for coord in ring.coords_mut() {
            coord.x += f64::from(coord.x < 0.) * TWO_PI;
        }
        let count = ring.lines().collect::<Intersections<_>>().count();
        if count > ring.lines().len() {
            // The "fixed" shaped is self-intersecting, revert the changes.
            for coord in ring.coords_mut() {
                coord.x -= f64::from(coord.x >= PI) * TWO_PI;
            }
            return false;
        }
    }

    is_transmeridian
}

// Check for arcs > 180 degrees (π radians) longitude to flag as transmeridian.
fn is_transmeridian(ring: &geo::LineString) -> bool {
    ring.lines()
        .any(|line| (line.start.x - line.end.x).abs() > PI)
}

impl From<Ring> for geo::LineString {
    fn from(value: Ring) -> Self {
        let (ring, _) = value.geom.into_inner();
        ring
    }
}
