use approx::assert_relative_eq;
use geo::{Line, MultiPolygon, Point, coord};
use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};

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
fn from_cell() {
    // Hexagon cell in Le Vigan.
    // https://h3geo.org/#hex=893961acb53ffff
    let cell = CellIndex::try_from(0x0893_961a_cb53_ffff).expect("index");
    let polygon = MultiPolygon::from(cell);
    assert_eq!(polygon.0.len(), 1);
    // 7 instead of 6 because the last point is duplicated to close the polygon.
    assert_eq!(polygon.0[0].exterior().0.len(), 7);

    // Pentagon cell in West Australia
    // https://h3geo.org/#hex=80a7fffffffffff
    let cell = CellIndex::try_from(0x080a_7fff_ffff_ffff).expect("index");
    let polygon = MultiPolygon::from(cell);
    assert_eq!(polygon.0.len(), 1);
    assert_eq!(polygon.0[0].exterior().0.len(), 6);
}

#[test]
fn from_transmeridian_cell() {
    // Transmeridian hexagon cell in between the US and Russia.
    // https://h3geo.org/#hex=840d9edffffffff
    let cell = CellIndex::try_from(0x0840_d9ed_ffff_ffff).expect("index");
    let polygon = MultiPolygon::from(cell);
    assert_eq!(polygon.0.len(), 2);
    assert_eq!(polygon.0[0].exterior().0.len(), 6);
    assert_eq!(polygon.0[1].exterior().0.len(), 6);

    // Make sure we return degrees, not radians.
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

    // Transmeridian pentagon cell.
    // https://h3geo.org/#hex=807ffffffffffff
    let cell = CellIndex::try_from(0x0807_ffff_ffff_ffff).expect("index");
    let polygon = MultiPolygon::from(cell);
    assert_eq!(polygon.0.len(), 2);
    assert_eq!(polygon.0[0].exterior().0.len(), 4);
    assert_eq!(polygon.0[1].exterior().0.len(), 7);
    // Make sure we return degrees, not radians.
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
