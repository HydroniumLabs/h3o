use approx::assert_relative_eq;
use geo::{coord, polygon, Line, Point, Polygon};
use h3o::{
    geom::cell_to_multi_polygon, CellIndex, DirectedEdgeIndex, VertexIndex,
};

#[test]
fn from_cell() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("index");
    let result = Polygon::from(index);
    let expected = polygon![
      (x: -122.02648011977477, y: 37.38558967035685),
      (x: -122.02540378194031, y: 37.38727461225182),
      (x: -122.02665619162275, y: 37.38879129032762),
      (x: -122.02898493935817, y: 37.38862300294707),
      (x: -122.03006120911812, y: 37.38693806029814),
      (x: -122.02880879921976, y: 37.38542140578344),
    ];

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_directed_edge() {
    let index =
        DirectedEdgeIndex::try_from(0x13a1_94e6_99ab_7fff).expect("index");
    let result = Line::from(index);
    let expected = Line::new(
        coord!(x: 0.004346277485193205, y: 51.5333297602599),
        coord!(x: 0.005128094944356792, y: 51.53286048728922),
    );

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_vertex() {
    let index = VertexIndex::try_from(0x2302_bfff_ffff_ffff).expect("index");
    let result = Point::from(index);
    let expected = Point::new(-74.64046816708004, 30.219492199828117);

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn to_simple_polygon() {
    // hexagon cell in Le vigan at low res: https://h3geo.org/#hex=893961acb53ffff
    let cell = CellIndex::try_from(0x0893_961a_cb53_ffff).expect("index");
    let polygon = cell_to_multi_polygon(cell);
    assert_eq!(polygon.0.len(), 1);
    assert_eq!(polygon.0[0].exterior().0.len(), 7); // the last point is duplicated to close the polygon

    // pentagon cell in west australia at res 0: https://h3geo.org/#hex=80a7fffffffffff
    let cell = CellIndex::try_from(0x080a_7fff_ffff_ffff).expect("index");
    let polygon = cell_to_multi_polygon(cell);
    assert_eq!(polygon.0.len(), 1);
    assert_eq!(polygon.0[0].exterior().0.len(), 6);
}

#[test]
fn to_multi_polygon() {
    // hexagon cell in between us and russia at low res: https://h3geo.org/#hex=840d9edffffffff
    let cell = CellIndex::try_from(0x0840_d9ed_ffff_ffff).expect("index");
    let polygon = cell_to_multi_polygon(cell);
    assert_eq!(polygon.0.len(), 2);
    assert_eq!(polygon.0[0].exterior().0.len(), 6);
    assert_eq!(polygon.0[1].exterior().0.len(), 6);
    assert!(
        polygon.0[0]
            .exterior()
            .0
            .iter()
            .any(|point| point.x == 180.0 || point.x == -180.0),
        "{polygon:?}"
    );
    assert!(
        polygon.0[1]
            .exterior()
            .0
            .iter()
            .any(|point| point.x == 180.0 || point.x == -180.0),
        "{polygon:?}"
    );

    // pentagon cell on the anti-meridian at res 0: https://h3geo.org/#hex=807ffffffffffff
    let cell = CellIndex::try_from(0x0807_ffff_ffff_ffff).expect("index");
    let polygon = cell_to_multi_polygon(cell);
    assert_eq!(polygon.0.len(), 2);
    assert_eq!(polygon.0[0].exterior().0.len(), 4);
    assert_eq!(polygon.0[1].exterior().0.len(), 7);
    assert!(
        polygon.0[0]
            .exterior()
            .0
            .iter()
            .any(|point| point.x == 180.0 || point.x == -180.0),
        "{polygon:?}"
    );
    assert!(
        polygon.0[1]
            .exterior()
            .0
            .iter()
            .any(|point| point.x == 180.0 || point.x == -180.0),
        "{polygon:?}"
    );
}
