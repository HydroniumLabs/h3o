use geo::polygon;
use h3o::{
    geom::{MultiPolygon, PolyfillConfig, ToCells},
    Resolution,
};

fn multipolygon_rads() -> geo::MultiPolygon {
    geo::MultiPolygon::new(vec![polygon!(
        exterior: [
            (x: 0.6559997912129759,  y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9735034901250053),
            (x: 0.6559997912129759,  y: 0.9735034901250053),
            (x: 0.6559997912129759,  y: 0.9726707149994819),
        ],
        interiors: [
            [
                (x: 0.6519638891502207, y: 0.9700359208861727),
                (x: 0.6608813108508083, y: 0.9700359208861727),
                (x: 0.6608813108508083, y: 0.9757587482556928),
                (x: 0.6519638891502207, y: 0.9757587482556928),
                (x: 0.6519638891502207, y: 0.9700359208861727),
            ],
        ],
    )])
}

fn multipolygon_degs() -> geo::MultiPolygon {
    geo::MultiPolygon::new(vec![polygon!(
        exterior: [
            (x: 37.58601939796671, y: 55.72992682544245),
            (x: 37.66530173673016, y: 55.72992682544245),
            (x: 37.66530173673016, y: 55.777641325418415),
            (x: 37.58601939796671, y: 55.777641325418415),
            (x: 37.58601939796671, y: 55.72992682544245),
        ],
        interiors: [
            [
                (x: 37.35477924324269, y: 55.57896424286392),
                (x: 37.86570987082473, y: 55.57896424286392),
                (x: 37.86570987082473, y: 55.90685809801937),
                (x: 37.35477924324269, y: 55.90685809801937),
                (x: 37.35477924324269, y: 55.57896424286392),
            ],
        ],
    )])
}

#[test]
fn from_radians() {
    let polygons = multipolygon_rads();
    let result = MultiPolygon::from_radians(polygons);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let polygons = multipolygon_degs();
    let result = MultiPolygon::from_degrees(polygons);

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let polygons = geo::MultiPolygon::new(vec![polygon![
        (x: -1., y: 3.),
        (x: -1., y: 1.),
        (x: -2., y: f64::NAN),
        (x: -2., y: 3.),]]);
    let result = MultiPolygon::from_degrees(polygons);

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let polygons = multipolygon_rads();
    let geom = MultiPolygon::from_radians(polygons).expect("geom");
    let result = geo::MultiPolygon::from(geom);
    let expected = multipolygon_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = MultiPolygon::from_degrees(multipolygon_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
