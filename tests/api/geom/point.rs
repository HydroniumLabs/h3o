use h3o::{
    geom::{Point, ToCells},
    Resolution,
};

fn point_rads() -> geo::Point {
    geo::Point::new(-0.7527922512723104, -0.4009198312650009)
}

fn point_degs() -> geo::Point {
    geo::Point::new(-43.13181884805516, -22.97101425458166)
}

#[test]
fn from_radians() {
    let result = Point::from_radians(point_rads());

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = Point::from_degrees(point_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result = Point::from_degrees(geo::Point::new(1.234, f64::NAN));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let geom = Point::from_radians(point_rads()).expect("geom");
    let result = geo::Point::from(geom);
    let expected = point_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Point::from_degrees(point_degs()).expect("geom");
    let bound = geom.max_cells_count(Resolution::Two);
    let result = geom.to_cells(Resolution::Two).count();

    assert!(result <= bound);
}
