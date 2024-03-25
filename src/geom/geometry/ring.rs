use super::bbox;
use crate::{error::InvalidGeometry, LatLng, TWO_PI};
use alloc::{borrow::Cow, vec::Vec};
use core::f64::consts::PI;
use geo::{
    algorithm::{
        centroid::Centroid,
        coordinate_position::{coord_pos_relative_to_ring, CoordPos},
    },
    sweep::Intersections,
    Contains, Coord, Intersects, Polygon,
};

/// A closed ring and its bounding box.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    geom: Polygon<f64>,
    bbox: geo::Rect<f64>,
    is_transmeridian: bool,
}

impl Ring {
    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in radians.
    pub fn from_radians(
        mut ring: geo::LineString<f64>,
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
        mut ring: geo::LineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        for coord in ring.coords_mut() {
            coord.x = coord.x.to_radians();
            coord.y = coord.y.to_radians();
        }
        Self::from_radians(ring)
    }

    pub fn geom(&self) -> &geo::LineString<f64> {
        self.geom.exterior()
    }

    pub fn centroid(&self) -> LatLng {
        let centroid = self.geom.centroid().expect("centroid");

        LatLng::from_radians(centroid.y(), centroid.x())
            .expect("valid coordinate")
    }

    pub const fn bbox(&self) -> geo::Rect<f64> {
        self.bbox
    }

    pub fn contains_centroid(&self, mut coord: Coord<f64>) -> bool {
        if self.is_transmeridian {
            coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
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

    /// If the cell is not transmeridian but the ring, we need to translate the
    /// cell boundaries.
    fn fixup_cell_boundary(&self, cell_boundary: &mut Cow<'_, Self>) {
        if self.is_transmeridian && !cell_boundary.is_transmeridian {
            cell_boundary.to_mut().geom.exterior_mut(|boundary| {
                for coord in boundary.coords_mut() {
                    coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
                }
            });
            // Don't forget to fixup the pre-computed bbox as well!
            cell_boundary.to_mut().bbox =
                bbox::compute_from_ring(cell_boundary.geom.exterior())
                    .expect("cell boundary is a valid geometry");
        }
    }
}

// Check for arcs > 180 degrees (π radians) longitude to flag as transmeridian
// and fix the shape accordingly.
fn fix_transmeridian(ring: &mut geo::LineString<f64>) -> bool {
    let is_transmeridian = ring
        .lines()
        .any(|line| (line.start.x - line.end.x).abs() > PI);

    // The heuristic above can be fooled by geometries with very long arcs.
    // Make sure that the "corrected" geometry is not self-intersecting.
    // This is not bullet-proof but will catch a bunch a false positives.
    if is_transmeridian {
        for coord in ring.coords_mut() {
            coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
        }
        let count = Intersections::<_>::from_iter(ring.lines()).count();
        if count > ring.lines().len() {
            // The "fixed" shaped is self-intersecting, revert the changes.
            for coord in ring.coords_mut() {
                coord.x -= f64::from(u8::from(coord.x >= PI)) * TWO_PI;
            }
            return false;
        }
    }

    is_transmeridian
}

impl From<Ring> for geo::LineString<f64> {
    fn from(value: Ring) -> Self {
        let (ring, _) = value.geom.into_inner();
        ring
    }
}
