use geo::polygon;
use h3o::{
    geom::{Polygon, ToCells},
    Resolution,
};

fn polygon_rads() -> geo::Polygon {
    polygon!(
        exterior: [
            (x: 0.6559997912129759, y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9735034901250053),
            (x: 0.6559997912129759, y: 0.9735034901250053),
            (x: 0.6559997912129759, y: 0.9726707149994819),
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
    )
}

fn polygon_degs() -> geo::Polygon {
    polygon!(
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
    )
}

#[test]
fn from_radians() {
    let polygon = polygon_rads();
    let result = Polygon::from_radians(&polygon);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let result = Polygon::from_degrees(polygon_degs());

    assert!(result.is_ok());
}

#[test]
fn invalid_nan() {
    let result = Polygon::from_degrees(polygon![
        (x: -1., y: 3.),
        (x: -1., y: 1.),
        (x: -2., y: f64::NAN),
        (x: -2., y: 3.)
    ]);

    assert!(result.is_err());
}

#[test]
fn invalid_point() {
    let result = Polygon::from_degrees(polygon![
        (x: -1., y: 3.),
    ]);

    assert!(result.is_err());
}

#[test]
fn invalid_line() {
    let result = Polygon::from_degrees(polygon![
        (x: -1., y: 3.),
        (x: -1., y: 1.),
    ]);

    assert!(result.is_err());
}

#[test]
fn into_geo() {
    let shape = polygon_rads();
    let geom = Polygon::from_radians(&shape).expect("geom");
    let result = geo::Polygon::from(geom);
    let expected = polygon_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Polygon::from_degrees(polygon_degs()).expect("geom");
    let bound = geom.max_cells_count(Resolution::Two);
    let result = geom.to_cells(Resolution::Two).count();

    assert!(result <= bound);
}
