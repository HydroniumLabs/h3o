use super::{
    AP7_ROT_RADS, EPSILON, FaceIJK, INV_SQRT7_POWERS, RES0_U_GNOMONIC, Vec2d,
    to_positive_angle,
};
use crate::{
    CellIndex, Face, LatLng, Resolution, face,
    math::{atan, atan2, cos, mul_add, sin, sqrt},
    resolution::ExtendedResolution,
};

/// 1/3
const ONE_THIRD: f64 = 0.3333333333333333;

const NORTH_POLE: Vec3d = Vec3d {
    x: 0.,
    y: 0.,
    z: 1.,
};

/// A 3D normal vector to the Earth ellipsoid, represented as a n-vector.
///
/// See: `<https://www.ffi.no/en/research/n-vector/n-vector-explained>`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3d {
    /// Initializes a new 3D vector with the specified component values.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        sqrt(self.norm_squared())
    }

    /// Computes the n-vector of the cell center point.
    ///
    /// # Arguments
    ///
    /// * `value` - The 2D hex coordinates of the cell.
    /// * `face` -  The icosahedral face upon which the 2D hex coordinate system
    ///   is centered.
    /// * `resolution` - The H3 resolution of the cell.
    /// * `is_substrate` - Indicates whether or not this grid is actually a
    ///   substrate grid relative to the specified resolution.
    pub fn from_vec2d(
        value: Vec2d,
        face: Face,
        resolution: ExtendedResolution,
        is_substrate: bool,
    ) -> Self {
        let face = usize::from(face);

        let r = {
            let mut r = value.magnitude();
            if r < EPSILON {
                return face::CENTER_POINT[face];
            }

            // Scale for current resolution length `u`.
            r *= INV_SQRT7_POWERS[usize::from(resolution)];

            // Scale accordingly if this is a substrate grid.
            if is_substrate {
                r *= ONE_THIRD;
                // Substrate grid are always adjusted to the next class II.
                debug_assert!(!resolution.is_class3());
            }

            // Perform inverse gnomonic scaling of `r`.
            atan(r * RES0_U_GNOMONIC)
        };

        let theta = {
            let mut theta = atan2(value.y, value.x);

            // Adjust theta for Class III.
            // If a substrate grid, then it's already adjusted for Class III.
            if !is_substrate && resolution.is_class3() {
                theta = to_positive_angle(theta + AP7_ROT_RADS);
            }

            // Find `theta` as an azimuth.
            to_positive_angle(face::AXES_AZ_RADS_CII[face] - theta)
        };
        let center = face::CENTER_POINT[face];
        if r < EPSILON {
            return center;
        }

        // Now find the point at `(r,theta)` from the face center
        let (north, east) = center.tangent_basis();
        let dir = linear_combination(cos(theta), &north, sin(theta), &east);
        let mut res = linear_combination(cos(r), &center, sin(r), &dir);
        res.normalize();

        res
    }

    /// Encode the n-vector into the H3 cell that contains it at the specified
    /// resolution.
    ///
    /// # Preconditions
    ///
    /// `self` is expected to be on the unit sphere.
    pub fn to_cell(self, resolution: Resolution) -> CellIndex {
        float_eq::debug_assert_float_eq!(self.norm(), 1., abs <= f64::EPSILON);
        FaceIJK::from_vec3d(self, resolution).to_cell(resolution)
    }

    /// Calculates the azimuth, in radians, from `self` to `other`.
    #[inline]
    pub fn azimuth(&self, other: &Self) -> f64 {
        let (north, east) = self.tangent_basis();

        // Project `other` onto tangent plane at `self`.
        let mut proj = linear_combination(1.0, other, -other.dot(self), self);
        proj.normalize();

        atan2(proj.dot(&east), proj.dot(&north))
    }

    /// Finds the closest icosahedral face from `self`.
    ///
    /// Returns both the face and the squared euclidean distance to that face
    /// center.
    ///
    /// # Preconditions
    ///
    /// `self` is expected to be on the unit sphere.
    #[must_use]
    pub fn closest_face(self) -> (Face, f64) {
        // The distance between two farthest points is 2.0, therefore the square
        // of the distance between two points should always be less or equal
        // than 4.
        const MAX_DIST: f64 = 5.0;

        float_eq::debug_assert_float_eq!(self.norm(), 1., abs <= f64::EPSILON);

        let (face, distance) = face::CENTER_POINT.iter().enumerate().fold(
            (0, MAX_DIST),
            |(face, distance), (i, center)| {
                let dist = self.distance_squared(center);

                if dist < distance {
                    // SAFETY: `face` is always in range because it's a index of
                    // `CENTER_POINT`.
                    (i, dist)
                } else {
                    (face, distance)
                }
            },
        );
        (Face::new_unchecked(face), distance)
    }

    /// Determines the n-vector of the center point of a cell given by a
    /// `FaceIJK` address at a specified resolution.
    ///
    /// # Arguments
    ///
    /// * `value` - The `FaceIJK` address of the cell.
    /// * `resolution` - The H3 resolution of the cell.
    fn from_face_ijk(value: FaceIJK, resolution: Resolution) -> Self {
        Self::from_vec2d(
            Vec2d::from(value.coord),
            value.face,
            resolution.into(),
            false,
        )
    }

    /// Computes the local north and east directions on the tangent plane at
    /// `self.`
    ///
    /// Will not work if `self` is at a pole, but icosahedron face centers
    /// are never at the poles.
    fn tangent_basis(&self) -> (Self, Self) {
        let mut north =
            linear_combination(1.0, &NORTH_POLE, -NORTH_POLE.dot(self), self);
        north.normalize();
        let east = north.cross(self);

        (north, east)
    }

    fn distance_squared(&self, other: &Self) -> f64 {
        linear_combination(1.0, self, -1.0, other).norm_squared()
    }

    fn normalize(&mut self) {
        let norm = self.norm();

        if norm > 0. {
            let scale = 1. / norm;
            // If the norm is nonzero, we normalize `self` using it.
            self.x *= scale;
            self.y *= scale;
            self.z *= scale;
        } else {
            // If the norm is 0 (either from true zero vector, or from squaring
            // underflowing to 0), we set the vector to be exactly zero.
            self.x = 0.;
            self.y = 0.;
            self.z = 0.;
        }
    }

    fn cross(&self, other: &Self) -> Self {
        Self {
            x: mul_add(self.y, other.z, -(self.z * other.y)),
            y: mul_add(self.z, other.x, -(self.x * other.z)),
            z: mul_add(self.x, other.y, -(self.y * other.x)),
        }
    }

    fn dot(&self, other: &Self) -> f64 {
        mul_add(self.x, other.x, mul_add(self.y, other.y, self.z * other.z))
    }

    fn norm_squared(&self) -> f64 {
        self.dot(self)
    }
}

impl From<LatLng> for Vec3d {
    #[inline]
    fn from(value: LatLng) -> Self {
        let r = cos(value.lat_radians());

        Self {
            x: cos(value.lng_radians()) * r,
            y: sin(value.lng_radians()) * r,
            z: sin(value.lat_radians()),
        }
    }
}

impl From<CellIndex> for Vec3d {
    fn from(value: CellIndex) -> Self {
        Self::from_face_ijk(FaceIJK::from(value), value.resolution())
    }
}

#[inline]
pub fn linear_combination(a: f64, v1: &Vec3d, b: f64, v2: &Vec3d) -> Vec3d {
    Vec3d {
        x: mul_add(a, v1.x, b * v2.x),
        y: mul_add(a, v1.y, b * v2.y),
        z: mul_add(a, v1.z, b * v2.z),
    }
}

#[cfg(test)]
#[path = "./vec3d_tests.rs"]
mod tests;
