use super::*;
use crate::{LatLng, Resolution, CCW, CW};

#[test]
fn from_ij_zero() {
    let ij = CoordIJ::new(0, 0);
    let ijk = CoordIJK::try_from(ij).expect("valid IJ");

    assert_eq!(ijk, CoordIJK::new(0, 0, 0), "ijk zero");
}

#[test]
fn from_ij_roundtrip() {
    for direction in Direction::iter() {
        let ijk = CoordIJK::new(0, 0, 0).neighbor(direction);
        let ij = CoordIJ::from(&ijk);
        let recovered = CoordIJK::try_from(ij).expect("valid IJ");

        assert_eq!(ijk, recovered, "roundtrip for direction {direction:?}");
    }
}

#[test]
fn from_cube_roundtrip() {
    for direction in Direction::iter() {
        let ijk = CoordIJK::new(0, 0, 0).neighbor(direction);
        let cube = CoordCube::from(ijk);
        let recovered = CoordIJK::from(cube);

        assert_eq!(ijk, recovered, "roundtrip for direction {direction:?}");
    }
}

#[test]
fn into_cell() {
    let ijk = CoordIJK::new(0, 0, 0);
    assert_eq!(
        Direction::try_from(ijk).expect("direction"),
        Direction::Center,
        "unit IJK zero"
    );

    let ijk = CoordIJK::new(1, 0, 0);
    assert_eq!(
        Direction::try_from(ijk).expect("direction"),
        Direction::I,
        "unit IJK on I axis"
    );

    let ijk = CoordIJK::new(2, 0, 0);
    assert!(Direction::try_from(ijk).is_err(), "non-unit IJK");

    let ijk = CoordIJK::new(2, 2, 2);
    assert_eq!(
        Direction::try_from(ijk).expect("direction"),
        Direction::Center,
        "denormalized unit IJK zero"
    );
}

#[test]
fn neighbor() {
    let ijk = CoordIJK::new(0, 0, 0);
    let i = CoordIJK::new(1, 0, 0);

    assert_eq!(
        ijk.neighbor(Direction::Center),
        ijk,
        "Center neighbor is self"
    );
    assert_eq!(ijk.neighbor(Direction::I), i, "I neighbor as expected");
}

#[test]
fn distance() {
    let z = CoordIJK::new(0, 0, 0);
    let i = CoordIJK::new(1, 0, 0);
    let ik = CoordIJK::new(1, 0, 1);
    let ij = CoordIJK::new(1, 1, 0);
    let j2 = CoordIJK::new(0, 2, 0);

    assert_eq!(z.distance(&z), 0, "identity distance 0,0,0");
    assert_eq!(i.distance(&i), 0, "identity distance 1,0,0");
    assert_eq!(ik.distance(&ik), 0, "identity distance 1,0,1");
    assert_eq!(ij.distance(&ij), 0, "identity distance 1,1,0");
    assert_eq!(j2.distance(&j2), 0, "identity distance 0,2,0");

    assert_eq!(z.distance(&i), 1, "0,0,0 to 1,0,0");
    assert_eq!(z.distance(&j2), 2, "0,0,0 to 0,2,0");
    assert_eq!(z.distance(&ik), 1, "0,0,0 to 1,0,1");
    assert_eq!(i.distance(&ik), 1, "1,0,0 to 1,0,1");
    assert_eq!(ik.distance(&j2), 3, "1,0,1 to 0,2,0");
    assert_eq!(ij.distance(&ik), 2, "1,0,1 to 1,1,0");
}

#[test]
fn from_hex2d() {
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("ll");
    let (face, distance) = ll.closest_face();
    let resolutions = Resolution::range(Resolution::Zero, Resolution::Fifteen);
    let cases = [
        CoordIJK::new(1, 0, 0),
        CoordIJK::new(4, 0, 3),
        CoordIJK::new(10, 0, 4),
        CoordIJK::new(26, 0, 17),
        CoordIJK::new(69, 0, 26),
        CoordIJK::new(182, 0, 122),
        CoordIJK::new(486, 0, 183),
        CoordIJK::new(1274, 0, 852),
        CoordIJK::new(3401, 0, 1283),
        CoordIJK::new(8920, 0, 5968),
        CoordIJK::new(23808, 0, 8985),
        CoordIJK::new(62439, 0, 41777),
        CoordIJK::new(166655, 0, 62891),
        CoordIJK::new(437074, 0, 292437),
        CoordIJK::new(1166585, 0, 440238),
        CoordIJK::new(3059517, 0, 2047062),
    ];

    for (resolution, &expected) in resolutions.zip(cases.iter()) {
        let result: CoordIJK = ll.to_vec2d(resolution, face, distance).into();

        assert_eq!(result, expected, "resolution {resolution}");
    }
}

#[test]
fn test_checked_up_aperture7_ccw() {
    let ijk = CoordIJK::new(0, 0, 0);
    assert!(ijk.checked_up_aperture7::<{ CCW }>().is_some());

    let ijk = CoordIJK::new(i32::MAX, 0, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CCW }>().is_none(),
        "i + i overflows"
    );

    let ijk = CoordIJK::new(i32::MAX / 2, 0, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CCW }>().is_none(),
        "i * 3 overflows"
    );

    let ijk = CoordIJK::new(0, i32::MAX, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CCW }>().is_none(),
        "j + j overflows"
    );

    // This input should be invalid because j < 0
    let ijk = CoordIJK::new(i32::MAX / 3, -2, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CCW }>().is_none(),
        "(i * 3) - j overflows"
    );

    let ijk = CoordIJK::new(i32::MAX / 3, i32::MAX / 2, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CCW }>().is_none(),
        "i + (j * 2) overflows"
    );

    let ijk = CoordIJK::new(-1, 0, 0);
    assert!(ijk.checked_up_aperture7::<{ CCW }>().is_some());
}

#[test]
fn test_checked_up_aperture7_cw() {
    let ijk = CoordIJK::new(0, 0, 0);
    assert!(ijk.checked_up_aperture7::<{ CW }>().is_some());

    let ijk = CoordIJK::new(i32::MAX, 0, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CW }>().is_none(),
        "i + i overflows"
    );

    let ijk = CoordIJK::new(0, i32::MAX, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CW }>().is_none(),
        "j + j overflows"
    );

    let ijk = CoordIJK::new(0, i32::MAX / 2, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CW }>().is_none(),
        "3 * j overflows"
    );

    let ijk = CoordIJK::new(i32::MAX / 2, i32::MAX / 3, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CW }>().is_none(),
        "(i * 2) + j overflows"
    );

    // This input should be invalid because i < 0
    let ijk = CoordIJK::new(-2, i32::MAX / 3, 0);
    assert!(
        ijk.checked_up_aperture7::<{ CW }>().is_none(),
        "(j * 3) - i overflows"
    );

    let ijk = CoordIJK::new(-1, 0, 0);
    assert!(ijk.checked_up_aperture7::<{ CW }>().is_some());
}
