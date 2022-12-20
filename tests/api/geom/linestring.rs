use h3o::{
    geom::{LineString, ToCells},
    Resolution,
};

fn linestring_rads() -> geo::LineString {
    geo::LineString::new(vec![
        geo::coord! { x: -0.009526982062241713, y: 0.8285232894553574 },
        geo::coord! { x: 0.04142734140306332, y: 0.8525145186317127 },
    ])
}

fn linestring_degs() -> geo::LineString {
    geo::LineString::new(vec![
        geo::coord! { x: -0.5458558636632915, y: 47.47088771408784 },
        geo::coord! { x: 2.373611818843102,   y: 48.84548389122412 },
    ])
}

#[test]
fn from_radians() {
    let line = linestring_rads();
    let result = LineString::from_radians(&line);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = LineString::from_degrees(linestring_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result = LineString::from_degrees(geo::LineString::new(vec![
        geo::coord! { x: 0., y: 0. },
        geo::coord! { x: f64::NAN, y: 0. },
    ]));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let line = linestring_rads();
    let geom = LineString::from_radians(&line).expect("geom");
    let result = geo::LineString::from(geom);
    let expected = linestring_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = LineString::from_degrees(linestring_degs()).expect("geom");
    let bound = geom.max_cells_count(Resolution::Two);
    let result = geom.to_cells(Resolution::Two).count();

    assert!(result <= bound);
}
