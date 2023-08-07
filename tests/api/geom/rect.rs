use h3o::{
    geom::{PolyfillConfig, Rect, ToCells},
    Resolution,
};

fn rect_rads() -> geo::Rect {
    geo::Rect::new(
        geo::coord! { x: 1.808355449236779, y: 0.02086683484240935 },
        geo::coord! { x: 1.816212429233187, y: 0.02571835428268519 },
    )
}

fn rect_degs() -> geo::Rect {
    geo::Rect::new(
        geo::coord! { x: 103.61113510075143, y: 1.19558156826659 },
        geo::coord! { x: 104.0613068942643,  y: 1.473553156420067 },
    )
}

#[test]
fn from_radians() {
    let result = Rect::from_radians(rect_rads());

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = Rect::from_degrees(rect_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let result = Rect::from_degrees(geo::Rect::new(
        geo::coord! { x: f64::NAN, y: 2.},
        geo::coord! { x: 2., y: 1.},
    ));

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let geom = Rect::from_radians(rect_rads()).expect("geom");
    let result = geo::Rect::from(geom);
    let expected = rect_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Rect::from_degrees(rect_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
