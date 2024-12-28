use approx::assert_relative_eq;
use geo::{coord, polygon, Line, Point, Polygon};
use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};

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
