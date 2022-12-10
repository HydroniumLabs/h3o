use h3o::IndexMode;

#[test]
fn into_u8() {
    let result = u8::from(IndexMode::Cell);
    let expected = 1;
    assert_eq!(result, expected);

    let result = u8::from(IndexMode::DirectedEdge);
    let expected = 2;
    assert_eq!(result, expected);

    let result = u8::from(IndexMode::UndirectedEdge);
    let expected = 3;
    assert_eq!(result, expected);

    let result = u8::from(IndexMode::Vertex);
    let expected = 4;
    assert_eq!(result, expected);
}

#[test]
fn display() {
    let result = IndexMode::Cell.to_string();
    let expected = "Cell".to_owned();
    assert_eq!(result, expected);

    let result = IndexMode::DirectedEdge.to_string();
    let expected = "DirectedEdge".to_owned();
    assert_eq!(result, expected);

    let result = IndexMode::UndirectedEdge.to_string();
    let expected = "UndirectedEdge".to_owned();
    assert_eq!(result, expected);

    let result = IndexMode::Vertex.to_string();
    let expected = "Vertex".to_owned();
    assert_eq!(result, expected);
}
