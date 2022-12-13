use h3o::DirectedEdgeIndex;

#[test]
fn display() {
    let index = DirectedEdgeIndex::try_from(0x13a194e699ab7fff).expect("edge");
    let result = index.boundary().to_string();
    let expected =
        "[(51.5333297603, 0.0043462775)-(51.5328604873, 0.0051280949)]"
            .to_owned();

    assert_eq!(result, expected);
}
