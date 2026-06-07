use super::{FloatAdder, atan2, mul_add, sin_cos};
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
    if ring.is_empty() {
        return 0.;
    }

    // Compute sincos for the first vertex.
    //
    // Used as the "a" side of the first edge/the "b" side of the last edge.
    let (fst_x, fst_y) = ring[0].xy();
    let fst_lat = mul_add(fst_y, 0.5, PI * 0.25);
    let sincos_fst = sin_cos(fst_lat);

    // Interior edges: (0,1), (1,2), …, (n-2, n-1).
    //
    // Each sincos is computed only once: the "b" side is forwarded to the next
    // iteration to be reused for the "a" side.
    let mut adder = FloatAdder::default();
    let mut sincos_a = sincos_fst;
    let mut a_x = fst_x;
    for b in &ring[1..] {
        let (b_x, b_y) = b.xy();
        let b_lat = mul_add(b_y, 0.5, PI * 0.25);
        let sincos_b = sin_cos(b_lat);

        adder += cagnoli(sincos_a, sincos_b, b_x - a_x);

        sincos_a = sincos_b;
        a_x = b_x;
    }

    // For the closing edge (n-1, 0), reuse the first vertex sincos.
    adder += cagnoli(sincos_a, sincos_fst, fst_x - a_x);

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
/// Uses pre-computed sincos(latitude) and longitude delta.
#[inline]
fn cagnoli(
    (sin_lat_a, cos_lat_a): (f64, f64),
    (sin_lat_b, cos_lat_b): (f64, f64),
    delta: f64,
) -> f64 {
    let sin_a = sin_lat_a * sin_lat_b;
    let cos_a = cos_lat_a * cos_lat_b;
    let (sin_d, cos_d) = sin_cos(delta);

    -2. * atan2(sin_a * sin_d, mul_add(sin_a, cos_d, cos_a))
}

#[cfg(test)]
#[path = "./area_tests.rs"]
mod tests;
