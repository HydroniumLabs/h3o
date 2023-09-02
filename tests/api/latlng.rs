use float_eq::assert_float_eq;
use h3o::{CellIndex, LatLng, Resolution};
use std::f64::consts::{FRAC_PI_2, PI};

const EPSILON: f64 = 1e-9 * PI / 180.0;

#[test]
fn nan() {
    let ll = LatLng::new(f64::NAN, 10.);
    assert!(ll.is_err(), "NaN latitude");

    let ll = LatLng::new(10., f64::NAN);
    assert!(ll.is_err(), "NaN longitude");
}

#[test]
fn infinity() {
    let ll = LatLng::new(f64::INFINITY, 10.);
    assert!(ll.is_err(), "infinite latitude");

    let ll = LatLng::new(10., f64::NEG_INFINITY);
    assert!(ll.is_err(), "infinite longitude");
}

#[test]
fn display() {
    let ll = LatLng::new(2.4, 8.2).expect("ll");
    let result = ll.to_string();
    let expected = "(2.4000000000, 8.2000000000)".to_owned();

    assert_eq!(result, expected);
}

#[test]
fn debug() {
    let ll = LatLng::new(2.4, 8.2).expect("ll");
    let result = format!("{ll:?}");

    assert!(result.contains("2.4"));
    assert!(result.contains("0.04188"));
    assert!(result.contains("8.2"));
    assert!(result.contains("0.14311"));
}

#[test]
#[allow(clippy::float_cmp)] // On purpose.
fn lat_lng() {
    let lat = 2.349014;
    let lng = 48.864716;
    let ll = LatLng::new(lat, lng).expect("ll");

    assert_eq!(ll.lat_radians(), lat.to_radians(), "lat radians");
    assert_eq!(ll.lng_radians(), lng.to_radians(), "lng radians");

    assert_eq!(ll.lat(), lat, "lat degrees");
    assert_eq!(ll.lng(), lng, "lng degrees");
}

#[test]
fn distance_rads() {
    let p1 = LatLng::new(10., 10.).expect("p1");
    let p2 = LatLng::new(0., 10.).expect("p2");

    assert_float_eq!(
        p1.distance_rads(p1),
        0.,
        abs <= EPSILON,
        "0 distance as expected"
    );

    assert_float_eq!(
        p1.distance_rads(p2),
        10.0_f64.to_radians(),
        abs <= EPSILON,
        "distance along longitude as expected"
    );
}

#[test]
fn distance_wrapped_longitude() {
    let negative_lng = LatLng::from_radians(0., -(PI + FRAC_PI_2)).expect("ll");
    let zero = LatLng::default();

    assert_float_eq!(
        negative_lng.distance_rads(zero),
        FRAC_PI_2,
        abs <= EPSILON,
        "distance with wrapped longitude"
    );
    assert_float_eq!(
        zero.distance_rads(negative_lng),
        FRAC_PI_2,
        abs <= EPSILON,
        "distance with wrapped longitude, swapped arguments"
    );
}

#[test]
fn to_cell_icosahedron_center() {
    let ll = LatLng::new(28.173218757257807, 23.03222744086644).expect("ll");
    let result = ll.to_cell(Resolution::Three);
    let expected = CellIndex::try_from(0x833e00fffffffff).expect("cell");

    assert_eq!(result, expected);
}

#[cfg(feature = "geo")]
#[test]
fn latlng_from_geo_coord() {
    let coord = geo::geometry::Coord {
        x: 23.03222744086644,
        y: 28.173218757257807,
    };
    // Geo coord are interpreted as degrees.
    let expected =
        LatLng::new(28.173218757257807, 23.03222744086644).expect("ll");
    let result = LatLng::try_from(coord).expect("ll from coord");

    assert_eq!(result, expected);
}
