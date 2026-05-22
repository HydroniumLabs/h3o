use super::*;
use crate::coord::EPSILON_RAD;
use float_eq::assert_float_eq;
use std::f64::consts::{FRAC_PI_2, PI};

#[test]
fn distance() {
    let v1 = Vec3d::new(0., 0., 0.);
    let v2 = Vec3d::new(1., 0., 0.);
    let v3 = Vec3d::new(0., 1., 1.);
    let v4 = Vec3d::new(1., 1., 1.);
    let v5 = Vec3d::new(1., 1., 2.);

    assert_float_eq!(
        v1.distance_squared(&v1),
        0.,
        abs <= f64::EPSILON,
        "distance to self is 0"
    );
    assert_float_eq!(
        v1.distance_squared(&v2),
        1.,
        abs <= f64::EPSILON,
        "distance to <1,0,0> is 1"
    );
    assert_float_eq!(
        v1.distance_squared(&v3),
        2.,
        abs <= f64::EPSILON,
        "distance to <0,1,1> is 2"
    );
    assert_float_eq!(
        v1.distance_squared(&v4),
        3.,
        abs <= f64::EPSILON,
        "distance to <1,1,1> is 3"
    );
    assert_float_eq!(
        v1.distance_squared(&v5),
        6.,
        abs <= f64::EPSILON,
        "distance to <1,1,2> is 6"
    );
}

#[test]
#[expect(clippy::float_cmp, reason = "we want exact equality")]
fn normalize_small_nonzero() {
    // 1e-163 squared underflows to 0, so norm == 0.
    // vec3Normalize should produce the zero vector.
    let mut v = Vec3d::new(1e-163, 0., 0.);

    assert_ne!(v.x, 0., "vector is nonzero");
    assert_eq!(v.norm(), 0.0, "norm underflows to zero");

    v.normalize();
    assert!(
        v.x == 0. && v.y == 0. && v.z == 0.,
        "underflowed vector normalizes to zero"
    );
}

#[test]
fn normalize_half_epsilon() {
    // f64::EPSILON/2 is small but normalizes fine.
    let mut v = Vec3d::new(f64::EPSILON / 2.0, 0., 0.);

    assert!(v.norm() < f64::EPSILON, "norm is small but nonzero");

    v.normalize();
    assert!(
        (v.x - 1.).abs() < f64::EPSILON && v.y == 0. && v.z == 0.,
        "still normalizable to unit vector"
    );
}

#[test]
fn closest_face() {
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).unwrap();
    let nvec = Vec3d::from(ll);
    let (face, distance) = nvec.closest_face();

    assert_eq!(u8::from(face), 3, "face");
    assert_float_eq!(
        distance,
        0.1922249255922707,
        abs <= EPSILON_RAD,
        "distance"
    );
}

#[test]
fn from_latlng() {
    let origin = Vec3d::new(0., 0., 0.);

    let c1 = LatLng::new_unchecked(0., 0.);
    let p1 = Vec3d::from(c1);
    assert_float_eq!(
        origin.distance_squared(&p1),
        1.,
        abs <= EPSILON_RAD,
        "Geo point is on the unit sphere"
    );

    let c2 = LatLng::new_unchecked(FRAC_PI_2, 0.);
    let p2 = Vec3d::from(c2);
    assert_float_eq!(
        p1.distance_squared(&p2),
        2.,
        abs <= EPSILON_RAD,
        "Geo point is on another axis"
    );

    let c3 = LatLng::new_unchecked(PI, 0.);
    let p3 = Vec3d::from(c3);
    assert_float_eq!(
        p1.distance_squared(&p3),
        4.,
        abs <= EPSILON_RAD,
        "Geo point is the other side of the sphere"
    );

    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("ll");
    let v3d = Vec3d::from(ll);

    assert_float_eq!(v3d.x, 0.6574080802540908, abs <= EPSILON_RAD, "x");
    assert_float_eq!(v3d.y, 0.0272433711102147, abs <= EPSILON_RAD, "y");
    assert_float_eq!(v3d.z, 0.7530421068885735, abs <= EPSILON_RAD, "z");
}

#[test]
#[expect(clippy::float_cmp, reason = "we want exact equality")]
fn dot_product() {
    let a = Vec3d::new(1., 0., 0.);
    let b = Vec3d::new(-1., 0., 0.);

    assert_eq!(a.dot(&b), -1.);
}

#[test]
fn cross_product_orthogonality() {
    let i = Vec3d::new(1., 0., 0.);
    let j = Vec3d::new(0., 1., 0.);
    let k = i.cross(&j);

    assert_float_eq!(k.x, 0., abs <= f64::EPSILON, "x component zero");
    assert_float_eq!(k.y, 0., abs <= f64::EPSILON, "y component zero");
    assert_float_eq!(k.z, 1., abs <= f64::EPSILON, "y component one");
    assert!(k.dot(&i).abs() < f64::EPSILON, "cross is orthogonal to i");
    assert!(k.dot(&j).abs() < f64::EPSILON, "cross is orthogonal to j");
}

#[test]
fn normalize_and_magnitude() {
    let mut v = Vec3d::new(3., -4., 12.);
    assert_float_eq!(
        v.norm_squared(),
        169.,
        abs <= f64::EPSILON,
        "squared norm"
    );
    assert_float_eq!(v.norm(), 13., abs <= f64::EPSILON, "norm");

    v.normalize();
    assert_float_eq!(
        v.norm(),
        1.,
        abs <= f64::EPSILON,
        "normalized norm is unit"
    );

    let mut zero = Vec3d::new(0., 0., 0.);
    zero.normalize();
    assert!(
        zero.x == 0.0 && zero.y == 0.0 && zero.z == 0.0,
        "zero vector remains unchanged when normalizing"
    );
}

#[test]
fn from_latlng_unit_sphere() {
    let ll = LatLng::new_unchecked(0.5, -1.3);
    let res = Vec3d::from(ll);

    assert_float_eq!(res.norm(), 1., abs <= f64::EPSILON);
}

#[test]
fn from_cell_unit_sphere() {
    let cell = LatLng::new_unchecked(0.6, -1.2).to_cell(Resolution::Five);
    let res = Vec3d::from(cell);

    assert_float_eq!(res.norm(), 1., abs <= f64::EPSILON);
}

#[test]
fn cell_to_vec3d_roundtrip() {
    let cell = LatLng::new_unchecked(-0.4, 0.8).to_cell(Resolution::Nine);
    let nvec = Vec3d::from(cell);

    assert_eq!(cell, nvec.to_cell(Resolution::Nine));
}
