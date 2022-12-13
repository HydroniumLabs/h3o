use h3o::Face;

#[test]
fn try_from_u8() {
    assert!(Face::try_from(0).is_ok(), "lower bound");
    assert!(Face::try_from(11).is_ok(), "valid value");
    assert!(Face::try_from(19).is_ok(), "upper bound");

    assert!(Face::try_from(20).is_err(), "out of range");
}

// Faces are displayed as numerical value.
#[test]
fn display() {
    let result = Face::try_from(2).expect("face").to_string();
    let expected = "2".to_owned();

    assert_eq!(result, expected);
}
