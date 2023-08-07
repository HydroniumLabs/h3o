use geo::point;
use h3o::{
    geom::{MultiPoint, PolyfillConfig, ToCells},
    Resolution,
};

fn multipoint_rads() -> geo::MultiPoint {
    geo::MultiPoint::new(vec![
        point![x: -2.1489548115593986, y: 0.8584581881195188],
        point![x: -1.382430711985295,  y: 0.7628836324009612],
    ])
}

fn multipoint_degs() -> geo::MultiPoint {
    geo::MultiPoint::new(vec![
        point![x: -123.12604106668468, y: 49.18603106769609],
        point![x: -79.20744526602287,  y: 43.71001239618482],
    ])
}

#[test]
fn from_radians() {
    let points = multipoint_rads();
    let result = MultiPoint::from_radians(points);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let points = multipoint_degs();
    let result = MultiPoint::from_degrees(&points);

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let points = geo::MultiPoint::new(vec![
        point![x: 0., y: 0.],
        point![x: 1., y: f64::NAN],
    ]);
    let result = MultiPoint::from_degrees(&points);

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let points = multipoint_rads();
    let geom = MultiPoint::from_radians(points).expect("geom");
    let result = geo::MultiPoint::from(geom);
    let expected = multipoint_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let points = multipoint_degs();
    let geom = MultiPoint::from_degrees(&points).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
