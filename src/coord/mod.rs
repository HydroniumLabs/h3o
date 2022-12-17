//! Coordinate systems used by H3 internally.
//!
//! See [Coordinate systems](https://h3geo.org/docs/next/core-library/coordsystems)

mod cube;
mod faceijk;
mod ijk;
mod latlng;
mod localij;
mod vec2d;
mod vec3d;

pub use cube::CoordCube;
pub use faceijk::{FaceIJK, Overage};
pub use ijk::{CoordIJ, CoordIJK};
pub use latlng::LatLng;
pub use localij::{LocalIJ, LocalIJK};
pub use vec3d::Vec3d;

use vec2d::Vec2d;

use crate::TWO_PI;

// -----------------------------------------------------------------------------

/// Threshold epsilon.
const EPSILON: f64 = 0.0000000000000001_f64;

/// Scaling factor from `hex2d` resolution 0 unit length (or distance between
/// adjacent cell center points on the plane) to gnomonic unit length.
const RES0_U_GNOMONIC: f64 = 0.381966011250105_f64;

/// Rotation angle between Class II and Class III resolution axes.
///
/// `asin(sqrt(3/28))`
const AP7_ROT_RADS: f64 = 0.3334731722518321_f64;

/// √3/2
const SQRT3_2: f64 = 0.8660254037844386_f64;

/// Power of √7 for each resolution.
const SQRT7_POWERS: &[f64] = &[
    1.0,
    2.6457513110645907,
    7.,
    18.520259177452136,
    49.00000000000001,
    129.64181424216497,
    343.0000000000001,
    907.4926996951549,
    2401.000000000001,
    6352.448897866085,
    16807.000000000007,
    44467.1422850626,
    117649.00000000007,
    311269.9959954382,
    823543.0000000006,
    2178889.971968068,
    5764801_f64,
];

// -----------------------------------------------------------------------------

/// Normalizes radians to a value between 0 and 2π.
pub fn to_positive_angle(mut angle: f64) -> f64 {
    if angle < 0. {
        angle += TWO_PI;
    } else if angle >= TWO_PI {
        angle -= TWO_PI;
    }
    debug_assert!((0.0..=TWO_PI).contains(&angle), "{angle}");

    angle
}
