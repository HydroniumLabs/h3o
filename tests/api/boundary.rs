use h3o::{CellIndex, DirectedEdgeIndex};

#[test]
fn display() {
    let index = DirectedEdgeIndex::try_from(0x13a194e699ab7fff).expect("edge");
    let result = index.boundary().to_string();
    let expected =
        "[(51.5333297603, 0.0043462775)-(51.5328604873, 0.0051280949)]"
            .to_owned();

    assert_eq!(result, expected);
}

#[cfg(feature = "geo")]
#[test]
fn geo_traits_support() {
    use geo_traits::{GeometryTrait as _, LineStringTrait as _};

    let index = CellIndex::try_from(0x813b7ffffffffff).unwrap();
    let boundary = index.boundary();

    // Boundary implements the LineStringTrait.
    assert_eq!(boundary.num_coords(), 6);
    assert_eq!(boundary[0], boundary.coord(0).unwrap());

    // And GeometryTrait.
    assert!(matches!(
        boundary.as_type(),
        geo_traits::GeometryType::LineString(_)
    ));
    assert_eq!(boundary.dim(), geo_traits::Dimensions::Xy);
}
