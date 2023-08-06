use super::bbox;
use crate::{error::InvalidGeometry, TWO_PI};
use geo::{
    algorithm::coordinate_position::{coord_pos_relative_to_ring, CoordPos},
    Coord, Intersects,
};
use std::{borrow::Cow, f64::consts::PI};

/// A closed ring and its bounding box.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring<'a> {
    geom: Cow<'a, geo::LineString<f64>>,
    bbox: geo::Rect<f64>,
    is_transmeridian: bool,
}

impl<'a> Ring<'a> {
    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in radians.
    pub fn from_radians(
        mut ring: Cow<'a, geo::LineString<f64>>,
    ) -> Result<Self, InvalidGeometry> {
        let is_transmeridian = ring_is_transmeridian(&ring);
        if is_transmeridian {
            for coord in ring.to_mut().coords_mut() {
                coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
            }
        }
        let bbox = bbox::compute_from_ring(&ring)?;

        Ok(Self {
            geom: ring,
            bbox,
            is_transmeridian,
        })
    }

    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in degrees.
    pub fn from_degrees(
        mut ring: geo::LineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let is_transmeridian = ring_is_transmeridian(&ring);
        let geom = {
            for coord in ring.coords_mut() {
                coord.x = f64::from(u8::from(coord.x < 0.))
                    .mul_add(TWO_PI, coord.x.to_radians());
                coord.y = coord.y.to_radians();
            }
            Cow::Owned(ring)
        };
        let bbox = bbox::compute_from_ring(&geom)?;

        Ok(Self {
            geom,
            bbox,
            is_transmeridian,
        })
    }

    pub fn geom(&self) -> &geo::LineString<f64> {
        self.geom.as_ref()
    }

    pub const fn bbox(&self) -> geo::Rect<f64> {
        self.bbox
    }

    pub fn contains(&self, mut coord: Coord<f64>) -> bool {
        if self.is_transmeridian {
            coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
        }

        if !self.bbox.intersects(&coord) {
            return false;
        }

        coord_pos_relative_to_ring(coord, &self.geom) == CoordPos::Inside
    }
}

// Check for arcs > 180 degrees longitude, flagging as transmeridian.
fn ring_is_transmeridian(ring: &geo::LineString<f64>) -> bool {
    ring.lines()
        .any(|line| (line.start.x - line.end.x).abs() > PI)
}

impl From<Ring<'_>> for geo::LineString<f64> {
    fn from(value: Ring<'_>) -> Self {
        value.geom.into_owned()
    }
}
