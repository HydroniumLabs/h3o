use crate::{
    LatLng,
    math::{atan, sqrt, tan},
};

/// A triangle on unit sphere.
pub struct Triangle {
    /// The first vertex coordinate.
    a: LatLng,
    /// The second vertex coordinate.
    b: LatLng,
    /// The third vertex coordinate.
    c: LatLng,
}

impl Triangle {
    /// Returns a triangle `ABC`.
    pub const fn new(a: LatLng, b: LatLng, c: LatLng) -> Self {
        Self { a, b, c }
    }

    /// Computes the area on unit sphere, in radians².
    pub fn area(&self) -> f64 {
        area_from_edges(
            self.a.distance_rads(self.b),
            self.b.distance_rads(self.c),
            self.c.distance_rads(self.a),
        )
    }
}

/// Computes the area on unit sphere, in radians², from its edges.
///
/// For the math, see [here](https://en.wikipedia.org/wiki/Spherical_trigonometry#Area_and_spherical_excess)
///
/// # Arguments
///
/// * `a` - length of triangle side `A`, in radians
/// * `b` - length of triangle side `B`, in radians
/// * `c` - length of triangle side `C`, in radians
fn area_from_edges(mut a: f64, mut b: f64, mut c: f64) -> f64 {
    let mut s = (a + b + c) * 0.5;

    a = (s - a) * 0.5;
    b = (s - b) * 0.5;
    c = (s - c) * 0.5;
    s *= 0.5;

    4. * atan(sqrt(tan(s) * tan(a) * tan(b) * tan(c)))
}
