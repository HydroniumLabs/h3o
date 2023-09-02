use h3o::Direction;

#[test]
fn try_from_u8() {
    assert!(Direction::try_from(0).is_ok(), "lower bound");
    assert!(Direction::try_from(3).is_ok(), "valid value");
    assert!(Direction::try_from(6).is_ok(), "upper bound");

    assert!(Direction::try_from(7).is_err(), "out of range");
}

#[test]
fn into_u8() {
    assert_eq!(u8::from(Direction::Center), 0, "lower bound");
    assert_eq!(u8::from(Direction::I), 4, "valid value");
    assert_eq!(u8::from(Direction::IJ), 6, "upper bound");
}

#[test]
fn into_u64() {
    assert_eq!(u64::from(Direction::Center), 0, "lower bound");
    assert_eq!(u64::from(Direction::I), 4, "valid value");
    assert_eq!(u64::from(Direction::IJ), 6, "upper bound");
}

// Directions are displayed as numerical value.
#[test]
fn display() {
    let result = Direction::J.to_string();
    let expected = "2".to_owned();

    assert_eq!(result, expected);
}
