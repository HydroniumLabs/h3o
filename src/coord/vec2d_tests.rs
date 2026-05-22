use super::*;
use crate::LatLng;
use float_eq::assert_float_eq;

#[test]
fn magnitude() {
    let v = Vec2d::new(3.0, 4.0);
    let expected = 5.0;

    let result = v.magnitude();

    assert_float_eq!(result, expected, abs <= f64::EPSILON);
}

#[test]
fn intersection() {
    let line1 = (Vec2d::new(2.0, 2.0), Vec2d::new(6.0, 6.0));
    let line2 = (Vec2d::new(0.0, 4.0), Vec2d::new(10.0, 4.0));
    let expected = Vec2d::new(4.0, 4.0);

    let result = Vec2d::intersection(line1, line2);

    assert_float_eq!(
        result.x,
        expected.x,
        abs <= f64::EPSILON,
        "x as expected"
    );
    assert_float_eq!(
        result.y,
        expected.y,
        abs <= f64::EPSILON,
        "y as expected"
    );
}

#[test]
fn from_vec3d() {
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("ll");
    let nvec = Vec3d::from(ll);
    let (face, distance) = nvec.closest_face();
    let resolutions = Resolution::range(Resolution::Zero, Resolution::Fifteen);
    let cases = [
        Vec2d::new(1.1492609554036695, -0.46294792417975866),
        Vec2d::new(2.4722277255402307, -2.152658993406549),
        Vec2d::new(8.044826687825688, -3.240635469258311),
        Vec2d::new(17.305594078781617, -15.068612953845845),
        Vec2d::new(56.31378681477982, -22.684448284808177),
        Vec2d::new(121.13915855147133, -105.48029067692093),
        Vec2d::new(394.1965077034588, -158.79113799365726),
        Vec2d::new(847.9741098602993, -738.3620347384465),
        Vec2d::new(2759.3755539242115, -1111.5379659556008),
        Vec2d::new(5935.818769022096, -5168.534243169126),
        Vec2d::new(19315.62887746948, -7780.7657616892075),
        Vec2d::new(41550.73138315468, -36179.73970218389),
        Vec2d::new(135209.4021422864, -54465.36033182446),
        Vec2d::new(290855.11968208273, -253258.1779152872),
        Vec2d::new(946465.8149960049, -381257.52232277126),
        Vec2d::new(2035985.8377745796, -1772807.245407011),
    ];

    for (resolution, &expected) in resolutions.zip(cases.iter()) {
        let result = Vec2d::from_vec3d(nvec, resolution, face, distance);

        assert_float_eq!(
            result.x,
            expected.x,
            abs <= 5e-10,
            "x, resolution {resolution}"
        );
        assert_float_eq!(
            result.y,
            expected.y,
            abs <= 5e-10,
            "y, resolution {resolution}"
        );
    }
}
