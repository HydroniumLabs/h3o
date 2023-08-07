use geo::line_string;
use h3o::{
    geom::{MultiLineString, PolyfillConfig, ToCells},
    Resolution,
};

fn multilinestring_rads() -> geo::MultiLineString {
    geo::MultiLineString::new(vec![line_string![
        (x: 1.996408325715777, y: 0.534292570530397),
        (x: 2.208424012168513, y: 0.7995167582816788),
        (x: 2.1213562369319434, y: 0.5449632604075227),
    ]])
}

fn multilinestring_degs() -> geo::MultiLineString {
    geo::MultiLineString::new(vec![line_string![
        (x: 114.385771248293,   y: 30.612709316587612),
        (x: 126.53337527260373, y: 45.8089358995214),
        (x: 121.54475921995464, y: 31.22409481103989),
    ]])
}

#[test]
fn from_radians() {
    let lines = multilinestring_rads();
    let result = MultiLineString::from_radians(lines);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = MultiLineString::from_degrees(multilinestring_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result =
        MultiLineString::from_degrees(geo::MultiLineString::new(vec![
            line_string![
                (x: 0., y: 0.),
                (x: 5., y: f64::NAN),
                (x: 0., y: 0.)
            ],
        ]));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let lines = multilinestring_rads();
    let geom = MultiLineString::from_radians(lines).expect("geom");
    let result = geo::MultiLineString::from(geom);
    let expected = multilinestring_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom =
        MultiLineString::from_degrees(multilinestring_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
