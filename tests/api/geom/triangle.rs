use h3o::{
    geom::{PolyfillConfig, ToCells, Triangle},
    Resolution,
};

fn triangle_rads() -> geo::Triangle {
    geo::Triangle::new(
        geo::coord! { x: 0.18729839227657055, y: 1.044910527020031 },
        geo::coord! { x: 0.314525021194105,   y: 1.034815187125519 },
        geo::coord! { x: 0.4379538402932019,  y: 1.0496570489924186 },
    )
}

fn triangle_degs() -> geo::Triangle {
    geo::Triangle::new(
        geo::coord! { x: 10.731407387033187, y: 59.868963167038345 },
        geo::coord! { x: 18.020956265684987, y: 59.29054279833275 },
        geo::coord! { x: 25.092906670346963, y: 60.14091884342227 },
    )
}

#[test]
fn from_radians() {
    let result = Triangle::from_radians(triangle_rads());

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = Triangle::from_degrees(triangle_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result = Triangle::from_degrees(geo::Triangle::new(
        geo::coord! { x: 0., y: 0. },
        geo::coord! { x: 1., y: 2. },
        geo::coord! { x: f64::NAN, y: -1. },
    ));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let geom = Triangle::from_radians(triangle_rads()).expect("geom");
    let result = geo::Triangle::from(geom);
    let expected = triangle_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Triangle::from_degrees(triangle_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
