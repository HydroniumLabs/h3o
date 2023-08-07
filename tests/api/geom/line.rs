use h3o::{
    geom::{Line, PolyfillConfig, ToCells},
    Resolution,
};

fn line_rads() -> geo::Line {
    geo::Line::new(
        geo::coord! { x: 0.05470401801197459, y: 0.8005260881667454 },
        geo::coord! { x: 0.0420053741471695, y: 0.8218402563603641 },
    )
}

fn line_degs() -> geo::Line {
    geo::Line::new(
        geo::coord! { x: 3.13430935449378,  y: 45.866766242072146 },
        geo::coord! { x: 2.406730655500752, y: 47.08797812339847 },
    )
}

#[test]
fn from_radians() {
    let result = Line::from_radians(line_rads());

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = Line::from_degrees(line_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result = Line::from_degrees(geo::Line::new(
        geo::coord! { x: 0., y: f64::NAN },
        geo::coord! { x: 1., y: 2. },
    ));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let geom = Line::from_radians(line_rads()).expect("geom");
    let result = geo::Line::from(geom);
    let expected = line_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Line::from_degrees(line_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
