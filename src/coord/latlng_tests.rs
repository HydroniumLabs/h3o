use super::*;
use float_eq::assert_float_eq;

#[test]
fn coord_at_noop() {
    let start = LatLng::new(15., 10.).expect("start");
    let expected = LatLng::new(15., 10.).expect("expected");
    let result = start.coord_at(0., 0.);

    assert_eq!(result, expected);
}

#[test]
fn coord_at_due_north_south() {
    // Due north to north pole.
    let start = LatLng::new(45., 1.).expect("start");
    let expected = LatLng::new(90., 0.).expect("expected");
    let result = start.coord_at(0., 45.0_f64.to_radians());
    assert_eq!(
        result, expected,
        "due north to north pole produces north pole"
    );

    // Due north to south pole, which doesn't get wrapped correctly.
    let start = LatLng::new(45., 1.).expect("start");
    let expected = LatLng::new(270., 1.).expect("expected");
    let result = start.coord_at(0., (45.0_f64 + 180.).to_radians());
    assert_eq!(
        result, expected,
        "due north to south pole produces south pole"
    );

    // Due south to south pole.
    let start = LatLng::new(-45., 2.).expect("start");
    let expected = LatLng::new(-90., 0.).expect("expected");
    let result = start.coord_at(180.0_f64.to_radians(), 45.0_f64.to_radians());
    assert_eq!(
        result, expected,
        "due south to south pole produces south pole"
    );

    // Due north to non-pole.
    let start = LatLng::new(-45., 10.).expect("start");
    let expected = LatLng::new(-10., 10.).expect("expected");
    let result = start.coord_at(0., 35.0_f64.to_radians());
    assert_eq!(result, expected, "due north produces expected result");
}

#[test]
fn coord_at_pole_to_pole() {
    // Azimuth doesn't really matter in this case. Any azimuth from the
    // north pole is south, any azimuth from the south pole is north.

    let start = LatLng::new(90., 0.).expect("start");
    let expected = LatLng::new(-90., 0.).expect("expected");
    let result = start.coord_at(12.0_f64.to_radians(), 180.0_f64.to_radians());
    assert_eq!(
        result, expected,
        "some direction to south pole produces south pole"
    );

    let start = LatLng::new(-90., 0.).expect("start");
    let expected = LatLng::new(90., 0.).expect("expected");
    let result = start.coord_at(34.0_f64.to_radians(), 180.0_f64.to_radians());
    assert_eq!(
        result, expected,
        "some direction to north pole produces north pole"
    );
}

#[test]
fn coord_at_invertible() {
    let start = LatLng::new(15., 10.).expect("start");
    let azimuth = 20.0_f64.to_radians();
    let degrees180 = 180.0_f64.to_radians();
    let distance = 15.0_f64.to_radians();

    let result = start.coord_at(azimuth, distance);
    assert_float_eq!(
        start.distance_rads(result),
        distance,
        abs <= EPSILON_RAD,
        "moved distance is as expected"
    );

    let start2 = result;
    let result = start2.coord_at(azimuth + degrees180, distance);
    assert!(start.distance_rads(result) < 0.01, "moved back to origin");
}

#[test]
fn to_vec2d() {
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("ll");
    let (face, distance) = ll.closest_face();
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
        let result = ll.to_vec2d(resolution, face, distance);

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

#[test]
fn into_vec3d() {
    let origin = Vec3d::new(0., 0., 0.);

    let c1 = LatLng::new_unchecked(0., 0.);
    let p1 = Vec3d::from(c1);
    assert_float_eq!(
        origin.distance(&p1),
        1.,
        abs <= EPSILON_RAD,
        "Geo point is on the unit sphere"
    );

    let c2 = LatLng::new_unchecked(FRAC_PI_2, 0.);
    let p2 = Vec3d::from(c2);
    assert_float_eq!(
        p1.distance(&p2),
        2.,
        abs <= EPSILON_RAD,
        "Geo point is on another axis"
    );

    let c3 = LatLng { lat: PI, lng: 0. };
    let p3 = Vec3d::from(c3);
    assert_float_eq!(
        p1.distance(&p3),
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
fn closest_face() {
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("ll");
    let (face, distance) = ll.closest_face();

    assert_eq!(u8::from(face), 3, "face");
    assert_float_eq!(
        distance,
        0.1922249255922707,
        abs <= EPSILON_RAD,
        "distance"
    );
}
