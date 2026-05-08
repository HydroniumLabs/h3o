use super::*;
use crate::{CellIndex, LatLng, Resolution};
use float_eq::assert_float_eq;
use std::f64::consts::FRAC_PI_2;

const EPSILON_RADS2: f64 = 1e-14; // Tolerance, in radian².
const EPSILON_KM2: f64 = 1e-6; // Tolerance, in km².
const EPSILON_M2: f64 = 1e0; // Tolerance, in m².
const EARTH_RADS2: f64 = 4. * PI; // Earth surface, in radian².
const EARTH_KM2: f64 = 510065621.7240886; // Earth surface, in km²
const EARTH_M2: f64 = 510065621724088.6; // Earth surface, in m²

// Ring representing a triangle covering ⅛ of the globe, with points ordered
// according to right-hand rule (counter-clockwise).
//
// The triangle starts at the north pole, moves down 90 degrees to the equator,
// and then sweeps out 90 degrees along the equator before returning to the
// north pole.
//
// The globe has an area of 4π radians², so this ⅛ triangle piece of the globe
// should have area π/2.
#[test]
fn triangle_basic() {
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(0., FRAC_PI_2).unwrap(),
    ];
    let expected = FRAC_PI_2;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// Reverse the order of the points in the triangle from the previous test, so
// that they are in clockwise order.
//
// Since the points are in clockwise order, the ring represents the whole globe
// minus the triangle above.
#[test]
fn triangle_reversed() {
    let ring = &[
        LatLng::from_radians(0., FRAC_PI_2).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
    ];
    let expected = 7. * FRAC_PI_2;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// Stitch two ⅛ triangles together, sharing an edge along the equator to create
// a ¼ slice of the globe, with vertices at the north and south pole.
#[test]
fn slice() {
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(-FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., FRAC_PI_2).unwrap(),
    ];
    let expected = PI;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// ¾ slice of the globe, from north to south pole, formed by reversing order of
// points from example above.
#[test]
fn slice_reversed() {
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., FRAC_PI_2).unwrap(),
        LatLng::from_radians(-FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
    ];
    let expected = 3. * PI;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// Stitch two ¼ triangles together to cover the eastern hemisphere.
#[test]
fn hemisphere_east() {
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(-FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., PI).unwrap(),
    ];
    let expected = 2. * PI;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// Stitch 4 ⅛ triangles together to cover the northern hemisphere.
#[test]
fn hemisphere_north() {
    let ring = &[
        LatLng::from_radians(0., -PI).unwrap(),
        LatLng::from_radians(0., -FRAC_PI_2).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(0., FRAC_PI_2).unwrap(),
    ];
    let expected = 2. * PI;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// Demonstrate that edge arcs between points in a ring should be less than 180
// degrees (π radians).
//
// Create a triangle from north pole to equator and back to the north pole that
// sweeps out an edge arc of tπ radians along the equator, so it should have an
// area of tπ for t in [0,1].
//
// However, there is a discontinuity at t = 1 (i.e., π radians), where expected
// area goes to (2 + t)*π for 1 < t < 2.
//
// Recall that the area in radians of the entire globe is 4π.
#[test]
fn percentage_slice() {
    let mut t = 0.;
    while t <= 1.2 {
        let ring = &[
            LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
            LatLng::from_radians(0., -FRAC_PI_2).unwrap(),
            LatLng::from_radians(0., t * PI - FRAC_PI_2).unwrap(),
        ];
        let result = linear_ring_area(ring);

        if t < 0.99 {
            // When t < 1, the largest angle in the triangle is less than π.
            let expected = t * PI;
            assert_float_eq!(result, expected, abs <= 1e-13);
        } else if t > 1.01 {
            // Discontinuity at t == 1. For t > 1, the triangle "flips", because
            // the shortest geodesic path is on the other side of the globe.
            // The triangle is now oriented in clockwise order, and the area
            // computed is the area *outside* of the triangle, which starts at
            // 3π.
            let expected = (2. + t) * PI;
            assert_float_eq!(result, expected, abs <= 1e-13);
        }
        // Note that we avoid testing t == 1, since the triangle
        // isn't well defined because there are many possible geodesic
        // shortest paths when consecutive points are antipodal
        // (180 degrees apart).

        t += 0.01;
    }
}

// A large polygon with t > 1 is *still* representable and we can compute its
// area accurately; we just need to add intermediate vertices so that no edge
// arc is greater than 180 degrees.
#[test]
fn percentage_slice_large() {
    let t = 1.2;
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., -FRAC_PI_2).unwrap(),
        LatLng::from_radians(0., 0.).unwrap(),
        LatLng::from_radians(0., t * PI - FRAC_PI_2).unwrap(),
    ];
    let expected = t * PI;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

#[test]
fn invalid_ring_line() {
    let ring = &[
        LatLng::from_radians(FRAC_PI_2, 0.).unwrap(),
        LatLng::from_radians(0., -FRAC_PI_2).unwrap(),
    ];
    let expected = 0.;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

#[test]
fn invalid_ring_point() {
    let ring = &[LatLng::from_radians(0., 0.).unwrap()];
    let expected = 0.;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

#[test]
fn invalid_ring_empty() {
    let ring: &[LatLng] = &[];
    let expected = 0.;
    let result = linear_ring_area(ring);

    assert_float_eq!(result, expected, abs <= EPSILON_RADS2);
}

// -----------------------------------------------------------------------------

// Apply a cell area calculation function to every cell on the earth at a given
// resolution, and check that it sums up the total earth area.
macro_rules! area_earth_test {
    ($name:ident, $resolution:literal, $area_fn:ident, $expected:ident, $epsilon: ident) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let area: f64 = CellIndex::base_cells()
                .flat_map(|index| {
                    index.children(resolution).map(|child| child.$area_fn())
                })
                .fold(FloatAdder::default(), |mut adder, area| {
                    adder += area;
                    adder
                })
                .into();

            assert_float_eq!(area, $expected, abs <= $epsilon);
        }
    };
}

area_earth_test!(earth_rads2_res0, 0, area_rads2, EARTH_RADS2, EPSILON_RADS2);
area_earth_test!(earth_rads2_res1, 1, area_rads2, EARTH_RADS2, EPSILON_RADS2);
area_earth_test!(earth_rads2_res2, 2, area_rads2, EARTH_RADS2, EPSILON_RADS2);
area_earth_test!(earth_rads2_res3, 3, area_rads2, EARTH_RADS2, EPSILON_RADS2);
area_earth_test!(earth_rads2_res4, 4, area_rads2, EARTH_RADS2, EPSILON_RADS2);

area_earth_test!(earth_km2_res0, 0, area_km2, EARTH_KM2, EPSILON_KM2);
area_earth_test!(earth_km2_res1, 1, area_km2, EARTH_KM2, EPSILON_KM2);
area_earth_test!(earth_km2_res2, 2, area_km2, EARTH_KM2, EPSILON_KM2);
area_earth_test!(earth_km2_res3, 3, area_km2, EARTH_KM2, EPSILON_KM2);
area_earth_test!(earth_km2_res4, 4, area_km2, EARTH_KM2, EPSILON_KM2);

area_earth_test!(earth_m2_res0, 0, area_m2, EARTH_M2, EPSILON_M2);
area_earth_test!(earth_m2_res1, 1, area_m2, EARTH_M2, EPSILON_M2);
area_earth_test!(earth_m2_res2, 2, area_m2, EARTH_M2, EPSILON_M2);
area_earth_test!(earth_m2_res3, 3, area_m2, EARTH_M2, EPSILON_M2);
area_earth_test!(earth_m2_res4, 4, area_m2, EARTH_M2, EPSILON_M2);
