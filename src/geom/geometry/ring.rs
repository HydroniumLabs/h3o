use super::bbox;
use crate::{error::InvalidGeometry, TWO_PI};
use geo::{Coord, Intersects};
use std::{borrow::Cow, f64::consts::PI};

/// A closed ring and its bounding box.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring<'a> {
    geom: Cow<'a, geo::LineString<f64>>,
    bbox: geo::Rect<f64>,
}

impl<'a> Ring<'a> {
    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in radians.
    pub fn from_radians(
        ring: Cow<'a, geo::LineString<f64>>,
    ) -> Result<Self, InvalidGeometry> {
        let bbox = bbox::compute_from_ring(&ring)?;

        Ok(Self { geom: ring, bbox })
    }

    /// Initialize a new ring from a closed `geo::LineString` whose coordinates
    /// are in degrees.
    pub fn from_degrees(
        mut ring: geo::LineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        let geom = {
            for coord in ring.coords_mut() {
                coord.x = coord.x.to_radians();
                coord.y = coord.y.to_radians();
            }
            Cow::Owned(ring)
        };
        let bbox = bbox::compute_from_ring(&geom)?;

        Ok(Self { geom, bbox })
    }

    pub fn geom(&self) -> &geo::LineString<f64> {
        self.geom.as_ref()
    }

    pub const fn bbox(&self) -> geo::Rect<f64> {
        self.bbox
    }

    // Those strict comparisons are done on purpose.
    #[allow(clippy::float_cmp)]
    pub fn contains(&self, mut coord: Coord<f64>) -> bool {
        // Use the ray-tracing algorithm: count #times a
        // horizontal ray from point (to positive infinity).
        //
        // See: https://en.wikipedia.org/wiki/Point_in_polygon

        let is_transmeridian = self.bbox.max().x > PI;
        if is_transmeridian {
            coord.x += f64::from(u8::from(coord.x < 0.)) * TWO_PI;
        }

        if !self.bbox.intersects(&coord) {
            return false;
        }

        let mut contains = false;
        for geo::Line { mut start, mut end } in self.geom.lines() {
            // Ray casting algo requires the second point to always be higher
            // than the first, so swap if needed.
            if start.y > end.y {
                (start, end) = (end, start);
            }

            // If the latitude matches exactly, we'll hit an edge case where the
            // ray passes through the vertex twice on successive segment checks.
            // To avoid this, adjust the latiude northward if needed.
            //
            // NOTE: This currently means that a point at the north pole cannot
            // be contained in any polygon. This is acceptable in current usage,
            // because the point we test in this function at present is always a
            // cell center or vertex, and no cell has a center or vertex on the
            // north pole. If we need to expand this algo to more generic uses
            // we might need to handle this edge case.
            if coord.y == start.y || coord.y == end.y {
                coord.y += f64::EPSILON;
            }

            // If we're totally above or below the latitude ranges, the test ray
            // cannot intersect the line segment, so let's move on.
            if coord.y < start.y || coord.y > end.y {
                continue;
            }

            if is_transmeridian {
                start.x += f64::from(u8::from(start.x < 0.)) * TWO_PI;
                end.x += f64::from(u8::from(end.x < 0.)) * TWO_PI;
            }

            // Rays are cast in the longitudinal direction, in case a point
            // exactly matches, to decide tiebreakers, bias westerly.
            if start.x == coord.x || end.x == coord.x {
                coord.x -= f64::EPSILON;
            }

            // For the latitude of the point, compute the longitude of the
            // point that lies on the line segment defined by `a` and `b`
            // This is done by computing the percent above `a` the lat is,
            // and traversing the same percent in the longitudinal direction
            // of `a` to `b`.
            let ratio = (coord.y - start.y) / (end.y - start.y);
            let mut test_lng = (end.x - start.x).mul_add(ratio, start.x);
            test_lng +=
                f64::from(u8::from(is_transmeridian && test_lng < 0.)) * TWO_PI;

            // Intersection of the ray
            if test_lng > coord.x {
                contains = !contains;
            }
        }

        contains
    }
}

impl From<Ring<'_>> for geo::LineString<f64> {
    fn from(value: Ring<'_>) -> Self {
        value.geom.into_owned()
    }
}
