use h3o::{
    geom::{Point, PolyfillConfig, ToCells},
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
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}

// https://github.com/nmandery/h3ronpy/issues/25
#[test]
fn h3ronpy_25() {
    // Manhattan Central Park
    let pt = geo::Point::new(-73.9575, 40.7938);
    let cells: Vec<_> =
        h3o::geom::Geometry::from_degrees(geo::Geometry::Point(pt))
            .unwrap()
            .to_cells(PolyfillConfig::new(Resolution::Eight))
            .collect();

    //  Using h3-py v3.7.x:
    //
    // $ python
    // Python 3.10.6 (main, May 29 2023, 11:10:38) [GCC 11.3.0] on linux
    // Type "help", "copyright", "credits" or "license" for more information.
    // >>> import h3.api.numpy_int as h3
    // >>> h3.geo_to_h3(40.7938, -73.9575, 8)
    // 613229523021856767
    assert_eq!(
        cells[0],
        h3o::CellIndex::try_from(613229523021856767).unwrap()
    )
}
