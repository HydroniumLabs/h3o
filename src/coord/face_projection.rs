use super::{
    AP7_ROT_RADS, EPSILON, INV_RES0_U_GNOMONIC, SQRT7_POWERS, Vec2d, Vec3d,
};
use crate::{
    Face, Resolution, face,
    math::{acos, mul_add, sin_cos, tan},
};

// Res-independent part of the gnomonic projection of an n-vector onto a face.
//
// Encoding at a given resolution only scales the radius and selects the Class
// II or Class III axes => the trigonometry can be shared between resolutions.
#[derive(Debug, Clone, Copy)]
pub struct FaceProjection {
    // (r0, theta)
    polar: Option<(f64, f64)>,
    // sin_cos(theta), per class.
    sin_cos_by_class: [Option<(f64, f64)>; 2],
}

impl FaceProjection {
    /// Projects n-vector onto the given face.
    ///
    /// # Preconditions
    ///
    /// `value` is expected to be on the unit sphere.
    ///
    /// # Arguments
    ///
    /// * `value` - The n-vector to encode.
    /// * `face` -  The icosahedral face of the coordinate.
    /// * `distance` - The squared euclidean distance from the face center.
    pub fn new(value: Vec3d, face: Face, distance: f64) -> Self {
        float_eq::debug_assert_float_eq!(value.norm(), 1., abs <= f64::EPSILON);

        let face = usize::from(face);

        // cos(r) = 1 - 2 * sin^2(r/2) = 1 - 2 * (sqd / 4) = 1 - sqd/2
        let r = acos(mul_add(distance, -0.5, 1.));
        if r < EPSILON {
            return Self {
                polar: None,
                sin_cos_by_class: [None; 2],
            };
        }

        // Apply gnomonic scaling of `r` (`tan(r)`) and scale for resolution 0.
        let r0 = tan(r) * INV_RES0_U_GNOMONIC;

        // Compute counter-clockwise `theta` from Class II i-axis.
        let theta = face::AXES_AZ_RADS_CII[face]
            - face::CENTER_POINT[face].azimuth(&value);

        Self {
            polar: Some((r0, theta)),
            sin_cos_by_class: [None; 2],
        }
    }

    /// Returns the 2D hex coordinates at the given resolution.
    pub fn compute_vec2d(&mut self, resolution: Resolution) -> Vec2d {
        let Some((r0, mut theta)) = self.polar else {
            return Vec2d::new(0., 0.);
        };

        // Scale for current resolution.
        let r = r0 * SQRT7_POWERS[usize::from(resolution)];

        // Adjust `theta` for Class III.
        if resolution.is_class3() {
            theta -= AP7_ROT_RADS;
        }

        // Convert to local x, y.
        let (sin_t, cos_t) = self.sin_cos_by_class
            [usize::from(resolution.is_class3())]
        .get_or_insert_with(|| sin_cos(theta));

        Vec2d::new(r * *cos_t, r * *sin_t)
    }
}
