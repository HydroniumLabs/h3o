use super::{FloatAdder, atan2, cos, mul_add, sin};
use core::f64::consts::PI;

// Not using `geo_traits::CoordTrait` because I want to avoid depending on geo
// ecosystem outside of the geo feature flag.
/// Coordinates on a 2-dimensional plane.
pub trait Coord2d {
    /// Returns the x & y component of the coordinate, in radians.
    fn xy(self) -> (f64, f64);
}

/// Computes the  area in radians² enclosed by the given linear ring.
///
/// The area enclosed by the ring is determined by the vertex order.
///
/// # Preconditions
///
/// - The ring should represent a simple curve with no self-intersections.
/// - Vertices should be ordered according to the "right hand rule" (CCW).
/// - The ring doesn't need to be closed by repeating the first coordinate at
///   the end!
///
/// # Limitations
///
/// The edge arcs between adjacent vertices are assumed to be the shortest
/// geodesic path between them (i.e. all arcs are interpreted to be less than
/// 180 degrees).
///
/// You should avoid arcs that are exactly π (i.e, two antipodal vertices).
/// "Large" ring (e.g., that cannot be contained in a hemisphere) can still be
/// constructed by using intermediate vertices with arcs less than
/// 180 degrees, and the area will still be computed correctly.
pub fn linear_ring_area<CoordType>(ring: &[CoordType]) -> f64
where
    CoordType: Coord2d + Copy,
{
    let mut adder = FloatAdder::default();

    for (a, b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
        adder += cagnoli(a.xy(), b.xy());
    }

    // The Cagnoli sum above yields a signed area, with the sign switching
    // with the orientation of the vertices.
    // Since we want our area to always be positive, we normalize into [0, 4*pi]
    // by adding 4*pi when the signed area is negative.
    if f64::from(adder) < 0. {
        adder += 4. * PI;
    }

    adder.into()
}

/// Computes the Cagnoli contribution for an arc from `a` to `b`.
///
/// This function is inspired from following
/// [d3-geo](https://github.com/d3/d3-geo/blob/8c53a90ae70c94bace73ecb02f2c792c649c86ba/src/area.js#L51-L70)
fn cagnoli((a_x, a_y): (f64, f64), (b_x, b_y): (f64, f64)) -> f64 {
    let a_lat = mul_add(a_y, 0.5, PI * 0.25);
    let b_lat = mul_add(b_y, 0.5, PI * 0.25);
    let sin_a = sin(a_lat) * sin(b_lat);
    let cos_a = cos(a_lat) * cos(b_lat);

    let delta = b_x - a_x;
    let sin_d = sin(delta);
    let cos_d = cos(delta);

    -2. * atan2(sin_a * sin_d, sin_a * cos_d + cos_a)
}

#[cfg(test)]
#[path = "./area_tests.rs"]
mod tests;
