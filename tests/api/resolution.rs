use h3o::Resolution;

#[test]
fn try_from_u8() {
    assert!(Resolution::try_from(0).is_ok(), "lower bound");
    assert!(Resolution::try_from(11).is_ok(), "valid value");
    assert!(Resolution::try_from(15).is_ok(), "upper bound");

    assert!(Resolution::try_from(16).is_err(), "out of range");
}

#[test]
fn try_from_str() {
    assert!("0".parse::<Resolution>().is_ok(), "lower bound");
    assert!("11".parse::<Resolution>().is_ok(), "valid value");
    assert!("15".parse::<Resolution>().is_ok(), "upper bound");

    assert!("One".parse::<Resolution>().is_err(), "invalid");
    assert!("16".parse::<Resolution>().is_err(), "out of range");
}

#[test]
fn into_u8() {
    assert_eq!(u8::from(Resolution::Zero), 0, "lower bound");
    assert_eq!(u8::from(Resolution::Eleven), 11, "valid value");
    assert_eq!(u8::from(Resolution::Fifteen), 15, "upper bound");
}

// Resolutions are displayed as numerical value.
#[test]
fn display() {
    let result = Resolution::Eleven.to_string();
    let expected = "11".to_owned();

    assert_eq!(result, expected);
}
