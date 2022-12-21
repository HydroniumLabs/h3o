//! Hex2d Coordinates
//!
//! A `Hex2d` coordinate system is a cartesian coordinate system associated with
//! a specific `ijk` coordinate system, where:
//!
//! - the origin of the `Hex2d` system is centered on the origin cell of the
//!   `ijk` system,
//! - the positive `x`-axis of the `Hex2d` system is aligned with the `i`-axis
//!   of the `ijk` system, and
//! - 1.0 unit distance in the `Hex2d` system is the distance between adjacent
//!   cell centers in the `ijk` coordinate system.

use super::{
    to_positive_angle, CoordIJK, AP7_ROT_RADS, EPSILON, RES0_U_GNOMONIC,
    SQRT7_POWERS,
};
use crate::{face, resolution::ExtendedResolution, Face, LatLng};
use float_eq::float_eq;

/// sin(60')
const SIN60: f64 = 0.8660254037844386;

// -----------------------------------------------------------------------------

/// 2D floating-point vector.
#[derive(Debug, Clone, Copy)]
pub struct Vec2d {
    /// `x` component.
    pub x: f64,
    /// `y` component.
    pub y: f64,
}

impl PartialEq for Vec2d {
    fn eq(&self, other: &Self) -> bool {
        float_eq!(self.x, other.x, abs <= f64::from(f32::EPSILON))
            && float_eq!(self.y, other.y, abs <= f64::from(f32::EPSILON))
    }
}

impl Eq for Vec2d {}

impl Vec2d {
    /// Initializes a new 2D vector with the specified component values.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculates the magnitude.
    pub fn magnitude(self) -> f64 {
        self.x.hypot(self.y)
    }

    /// Finds the intersection between two lines.
    ///
    /// Assumes that the lines intersect and that the intersection is not at an
    /// endpoint of either line.
    pub fn intersection(line1: (Self, Self), line2: (Self, Self)) -> Self {
        let s1 = Self {
            x: line1.1.x - line1.0.x,
            y: line1.1.y - line1.0.y,
        };
        let s2 = Self {
            x: line2.1.x - line2.0.x,
            y: line2.1.y - line2.0.y,
        };

        let t = s2
            .x
            .mul_add(line1.0.y - line2.0.y, -s2.y * (line1.0.x - line2.0.x))
            / (-s2.x).mul_add(s1.y, s1.x * s2.y);

        Self {
            x: t.mul_add(s1.x, line1.0.x),
            y: t.mul_add(s1.y, line1.0.y),
        }
    }

    /// Computes the spherical coordinates of the cell center point.
    ///
    /// # Arguments
    ///
    /// * `vec2d` - The 2D hex coordinates of the cell.
    /// * `face` -  The icosahedral face upon which the 2D hex coordinate system
    ///             is centered.
    /// * `resolution` - The H3 resolution of the cell.
    /// * `is_substrate` - Indicates whether or not this grid is actually a
    ///                    substrate grid relative to the specified resolution.
    pub fn to_latlng(
        self,
        face: Face,
        resolution: ExtendedResolution,
        is_substrate: bool,
    ) -> LatLng {
        let face = usize::from(face);

        let r = {
            let mut r = self.magnitude();
            if r < EPSILON {
                return face::CENTER_GEO[face];
            }

            // Scale for current resolution length `u`.
            r /= SQRT7_POWERS[usize::from(resolution)];

            // Scale accordingly if this is a substrate grid.
            if is_substrate {
                r /= 3.;
                // Substrate grid are always adjusted to the next class II.
                debug_assert!(!resolution.is_class3());
            }

            // Perform inverse gnomonic scaling of `r`.
            (r * RES0_U_GNOMONIC).atan()
        };

        let theta = {
            let mut theta = self.y.atan2(self.x);

            // Adjust theta for Class III.
            // If a substrate grid, then it's already adjusted for Class III.
            if !is_substrate && resolution.is_class3() {
                theta = to_positive_angle(theta + AP7_ROT_RADS);
            }

            // Find `theta` as an azimuth.
            to_positive_angle(face::AXES_AZ_RADS_CII[face][0] - theta)
        };

        // Now find the point at `(r,theta)` from the face center
        face::CENTER_GEO[face].coord_at(theta, r)
    }
}

impl From<Vec2d> for CoordIJK {
    // Returns the containing hex in `IJK` coordinates for a 2D cartesian
    // coordinate vector (from DGGRID).
    fn from(value: Vec2d) -> Self {
        // Quantize into the IJ system and then normalize.
        let k = 0;

        let a1 = value.x.abs();
        let a2 = value.y.abs();

        // First do a reverse conversion.
        let x2 = a2 / SIN60;
        let x1 = a1 + x2 / 2.;

        // Check if we have the center of a hex.
        #[allow(clippy::cast_possible_truncation)] // on purpose.
        let m1 = x1 as i32;
        #[allow(clippy::cast_possible_truncation)] // on purpose.
        let m2 = x2 as i32;

        // Otherwise round correctly.
        let r1 = x1 - f64::from(m1);
        let r2 = x2 - f64::from(m2);

        let (mut i, mut j) = if r1 < 0.5 {
            if r1 < 1. / 3. {
                let i = m1;
                let j = m2 + i32::from(r2 >= (1. + r1) / 2.);
                (i, j)
            } else {
                let i = m1 + i32::from((1. - r1) <= r2 && r2 < (2. * r1));
                let j = m2 + i32::from(r2 >= (1. - r1));
                (i, j)
            }
        } else if r1 < 2. / 3. {
            let j = m2 + i32::from(r2 >= (1. - r1));
            let i = m1
                + i32::from(2.0_f64.mul_add(r1, -1.) >= r2 || r2 >= (1. - r1));
            (i, j)
        } else {
            let i = m1 + 1;
            let j = m2 + i32::from(r2 >= (r1 / 2.));
            (i, j)
        };

        // Now fold across the axes if necessary.
        if value.x < 0. {
            let offset = j % 2;
            let axis_i = (j + offset) / 2;
            let diff = i - axis_i;
            i -= 2 * diff + offset;
        }

        if value.y < 0. {
            i -= (2 * j + 1) / 2;
            j = -j;
        }

        Self::new(i, j, k).normalize()
    }
}

#[cfg(test)]
#[path = "./vec2d_tests.rs"]
mod tests;
