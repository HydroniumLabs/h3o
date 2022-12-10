use h3o::Edge;

#[test]
fn try_from_u8() {
    assert!(Edge::try_from(1).is_ok(), "lower bound");
    assert!(Edge::try_from(3).is_ok(), "valid value");
    assert!(Edge::try_from(6).is_ok(), "upper bound");

    assert!(Edge::try_from(0).is_err(), "out of range, low");
    assert!(Edge::try_from(7).is_err(), "out of range, high");
}

#[test]
fn into_u8() {
    let edge = Edge::try_from(1).expect("edge");
    assert_eq!(u8::from(edge), 1, "lower bound");

    let edge = Edge::try_from(3).expect("edge");
    assert_eq!(u8::from(edge), 3, "valid value");

    let edge = Edge::try_from(6).expect("edge");
    assert_eq!(u8::from(edge), 6, "upper bound");
}

#[test]
fn display() {
    let result = Edge::try_from(3).expect("edge").to_string();
    let expected = "3".to_owned();

    assert_eq!(result, expected);
}
