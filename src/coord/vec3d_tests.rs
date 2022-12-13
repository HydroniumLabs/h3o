use super::*;
use float_eq::assert_float_eq;

#[test]
fn distance() {
    let v1 = Vec3d::new(0., 0., 0.);
    let v2 = Vec3d::new(1., 0., 0.);
    let v3 = Vec3d::new(0., 1., 1.);
    let v4 = Vec3d::new(1., 1., 1.);
    let v5 = Vec3d::new(1., 1., 2.);

    assert_float_eq!(
        v1.distance(&v1),
        0.,
        abs <= f64::EPSILON,
        "distance to self is 0"
    );
    assert_float_eq!(
        v1.distance(&v2),
        1.,
        abs <= f64::EPSILON,
        "distance to <1,0,0> is 1"
    );
    assert_float_eq!(
        v1.distance(&v3),
        2.,
        abs <= f64::EPSILON,
        "distance to <0,1,1> is 2"
    );
    assert_float_eq!(
        v1.distance(&v4),
        3.,
        abs <= f64::EPSILON,
        "distance to <1,1,1> is 3"
    );
    assert_float_eq!(
        v1.distance(&v5),
        6.,
        abs <= f64::EPSILON,
        "distance to <1,1,2> is 6"
    );
}
