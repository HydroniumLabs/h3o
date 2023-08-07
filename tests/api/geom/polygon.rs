use geo::polygon;
use h3o::{
    geom::{ContainmentMode, PolyfillConfig, Polygon, ToCells},
    Resolution,
};
use std::{fs::File, io::BufReader, path::PathBuf};

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
    let result = Polygon::from_radians(polygon);

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
    let geom = Polygon::from_radians(shape).expect("geom");
    let result = geo::Polygon::from(geom);
    let expected = polygon_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = Polygon::from_degrees(polygon_degs()).expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}

#[test]
fn to_cells_paris_centroid() {
    let geom = load_polygon("Paris");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::ContainsCentroid),
        )
        .count();

    assert_eq!(result, 164, "Paris/mode=Centroid");
}

#[test]
fn to_cells_paris_contains() {
    let geom = load_polygon("Paris");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::ContainsBoundary),
        )
        .count();

    assert_eq!(result, 118, "Paris/mode=Contains");
}

#[test]
fn to_cells_paris_intersects() {
    let geom = load_polygon("Paris");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::IntersectsBoundary),
        )
        .count();

    assert_eq!(result, 203, "Paris/mode=Intersects");
}

#[test]
fn to_cells_rabi_centroid() {
    let geom = load_polygon("Rabi");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::ContainsCentroid),
        )
        .count();

    assert_eq!(result, 163, "Rabi/mode=Centroid");
}

#[test]
fn to_cells_rabi_contains() {
    let geom = load_polygon("Rabi");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::ContainsBoundary),
        )
        .count();

    assert_eq!(result, 132, "Rabi/mode=Contains");
}

#[test]
fn to_cells_rabi_intersects() {
    let geom = load_polygon("Rabi");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Eight)
                .containment_mode(ContainmentMode::IntersectsBoundary),
        )
        .count();

    assert_eq!(result, 193, "Rabi/mode=Intersects");
}

#[test]
fn to_cells_holes_centroid() {
    let geom = load_polygon("Holes");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Four)
                .containment_mode(ContainmentMode::ContainsCentroid),
        )
        .count();

    assert_eq!(result, 233, "Holes/mode=Centroid");
}

#[test]
fn to_cells_holes_contains() {
    let geom = load_polygon("Holes");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Four)
                .containment_mode(ContainmentMode::ContainsBoundary),
        )
        .count();

    assert_eq!(result, 170, "Holes/mode=Contains");
}

#[test]
fn to_cells_holes_intersects() {
    let geom = load_polygon("Holes");
    let result = geom
        .to_cells(
            PolyfillConfig::new(Resolution::Four)
                .containment_mode(ContainmentMode::IntersectsBoundary),
        )
        .count();

    assert_eq!(result, 285, "Holes/mode=Intersects");
}

//------------------------------------------------------------------------------

pub fn load_polygon(name: &str) -> h3o::geom::Polygon {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let filepath = format!("dataset/{name}/shape.geojson");
    path.push(filepath);

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    let geojson = geojson::GeoJson::from_reader(reader).expect("GeoJSON");
    let geometry = h3o::geom::Geometry::try_from(&geojson).expect("geometry");
    h3o::geom::Polygon::try_from(geometry).expect("polygon")
}
