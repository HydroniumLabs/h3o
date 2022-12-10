use h3o::BaseCell;

#[test]
fn is_pentagon() {
    let cell = BaseCell::try_from(4).expect("pentagonal cell");
    assert!(cell.is_pentagon(), "pentagon");

    let cell = BaseCell::try_from(8).expect("hexagonal cell");
    assert!(!cell.is_pentagon(), "hexagon");
}

#[test]
fn count() {
    assert_eq!(BaseCell::count(), 122);
}

#[test]
fn try_from_u8() {
    assert!(BaseCell::try_from(0).is_ok(), "lower bound");
    assert!(BaseCell::try_from(42).is_ok(), "valid value");
    assert!(BaseCell::try_from(121).is_ok(), "upper bound");

    assert!(BaseCell::try_from(122).is_err(), "out of range");
}

#[test]
fn into_u8() {
    let cell = BaseCell::try_from(0).expect("base cell");
    assert_eq!(u8::from(cell), 0, "lower bound");

    let cell = BaseCell::try_from(42).expect("base cell");
    assert_eq!(u8::from(cell), 42, "valid value");

    let cell = BaseCell::try_from(121).expect("base cell");
    assert_eq!(u8::from(cell), 121, "upper bound");
}

#[test]
fn display() {
    let result = BaseCell::try_from(33).expect("base cell").to_string();
    let expected = "33".to_owned();

    assert_eq!(result, expected);
}
