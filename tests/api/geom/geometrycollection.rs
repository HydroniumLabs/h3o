use geo::{line_string, point, polygon};
use h3o::{
    geom::{GeometryCollection, PolyfillConfig, ToCells},
    Resolution,
};

fn geometrycollection_rads() -> geo::GeometryCollection {
    let line = geo::Line::new(
        geo::coord! { x: 0.05470401801197459, y: 0.8005260881667454 },
        geo::coord! { x: 0.0420053741471695, y: 0.8218402563603641 },
    );
    let linestring = geo::LineString::new(vec![
        geo::coord! { x: -0.009526982062241713, y: 0.8285232894553574 },
        geo::coord! { x: 0.04142734140306332, y: 0.8525145186317127 },
    ]);
    let lines = geo::MultiLineString::new(vec![line_string![
        (x: 1.996408325715777, y: 0.534292570530397),
        (x: 2.208424012168513, y: 0.7995167582816788),
        (x: 2.1213562369319434, y: 0.5449632604075227),
    ]]);
    let point = geo::Point::new(-0.7527922512723104, -0.4009198312650009);
    let points = geo::MultiPoint::new(vec![
        point![x: -2.1489548115593986, y: 0.8584581881195188],
        point![x: -1.382430711985295,  y: 0.7628836324009612],
    ]);
    let polygon = polygon!(
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
    );
    let polygons = geo::MultiPolygon::new(vec![polygon.clone()]);
    let rect = geo::Rect::new(
        geo::coord! { x: 1.808355449236779, y: 0.02086683484240935 },
        geo::coord! { x: 1.816212429233187, y: 0.02571835428268519 },
    );
    let triangle = geo::Triangle::new(
        geo::coord! { x: 0.18729839227657055, y: 1.044910527020031 },
        geo::coord! { x: 0.314525021194105,   y: 1.034815187125519 },
        geo::coord! { x: 0.4379538402932019,  y: 1.0496570489924186 },
    );
    let inner = geo::GeometryCollection::new_from(vec![point.into()]);
    geo::GeometryCollection::new_from(vec![
        line.into(),
        linestring.into(),
        lines.into(),
        point.into(),
        points.into(),
        polygon.into(),
        polygons.into(),
        rect.into(),
        triangle.into(),
        geo::Geometry::GeometryCollection(inner),
    ])
}

fn geometrycollection_degs() -> geo::GeometryCollection {
    let line = geo::Line::new(
        geo::coord! { x: 3.13430935449378,  y: 45.866766242072146 },
        geo::coord! { x: 2.406730655500752, y: 47.08797812339847 },
    );
    let linestring = geo::LineString::new(vec![
        geo::coord! { x: -0.5458558636632915, y: 47.47088771408784 },
        geo::coord! { x: 2.373611818843102,   y: 48.84548389122412 },
    ]);
    let lines = geo::MultiLineString::new(vec![line_string![
        (x: 114.385771248293,   y: 30.612709316587612),
        (x: 126.53337527260373, y: 45.8089358995214),
        (x: 121.54475921995464, y: 31.22409481103989)
    ]]);
    let point = geo::Point::new(-43.13181884805516, -22.97101425458166);
    let points = geo::MultiPoint::new(vec![
        point![x: -123.12604106668468, y: 49.18603106769609],
        point![x: -79.20744526602287,  y: 43.71001239618482],
    ]);
    let polygon = polygon!(
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
    );
    let polygons = geo::MultiPolygon::new(vec![polygon.clone()]);
    let rect = geo::Rect::new(
        geo::coord! { x: 103.61113510075143, y: 1.19558156826659 },
        geo::coord! { x: 104.0613068942643,  y: 1.473553156420067 },
    );
    let triangle = geo::Triangle::new(
        geo::coord! { x: 10.731407387033187, y: 59.868963167038345 },
        geo::coord! { x: 18.020956265684987, y: 59.29054279833275 },
        geo::coord! { x: 25.092906670346963, y: 60.14091884342227 },
    );
    let inner = geo::GeometryCollection::new_from(vec![point.into()]);
    geo::GeometryCollection::new_from(vec![
        line.into(),
        linestring.into(),
        lines.into(),
        point.into(),
        points.into(),
        polygon.into(),
        polygons.into(),
        rect.into(),
        triangle.into(),
        geo::Geometry::GeometryCollection(inner),
    ])
}

#[test]
fn from_radians() {
    let geoms = geometrycollection_rads();
    let result = GeometryCollection::from_radians(geoms);

    assert!(result.is_ok());
}

#[test]
fn from_degrees() {
    let geoms = geometrycollection_degs();
    let result = GeometryCollection::from_degrees(geoms);

    assert!(result.is_ok());
}

#[test]
fn into_geo() {
    let geoms = geometrycollection_rads();
    let geom = GeometryCollection::from_radians(geoms).expect("geom");
    let result = geo::GeometryCollection::from(geom);
    let expected = geometrycollection_rads();

    assert_eq!(result, expected);
}

#[test]
fn to_cells() {
    let geom = GeometryCollection::from_degrees(geometrycollection_degs())
        .expect("geom");
    let config = PolyfillConfig::new(Resolution::Two);
    let bound = geom.max_cells_count(config);
    let result = geom.to_cells(config).count();

    assert!(result <= bound);
}
