use h3o::Vertex;

#[test]
fn try_from_u8() {
    assert!(Vertex::try_from(0).is_ok(), "lower bound");
    assert!(Vertex::try_from(3).is_ok(), "valid value");
    assert!(Vertex::try_from(5).is_ok(), "upper bound");

    assert!(Vertex::try_from(6).is_err(), "out of range: low");
}

#[test]
fn into_u8() {
    let vertex = Vertex::try_from(0).expect("vertex");
    assert_eq!(u8::from(vertex), 0, "lower bound");

    let vertex = Vertex::try_from(3).expect("vertex");
    assert_eq!(u8::from(vertex), 3, "valid value");

    let vertex = Vertex::try_from(5).expect("vertex");
    assert_eq!(u8::from(vertex), 5, "upper bound");
}

#[test]
fn display() {
    let result = Vertex::try_from(3).expect("vertex").to_string();
    let expected = "3".to_owned();

    assert_eq!(result, expected);
}
