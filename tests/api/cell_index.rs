use h3o::CellIndex;

#[test]
fn try_from_str() {
    let result = "8a1fb46622dffff".parse::<CellIndex>();
    let expected = CellIndex::try_from(0x8a1fb46622dffff);
    assert_eq!(result, expected, "valid string");

    let result = "no bueno".parse::<CellIndex>();
    assert!(result.is_err(), "invalid string");
}

// Resolutions are displayed as numerical value.
#[test]
fn display() {
    let index = CellIndex::try_from(0x8a1fb46622dffff).expect("index");

    // Default display is the lower hex one.
    let result = index.to_string();
    let expected = "8a1fb46622dffff".to_owned();
    assert_eq!(result, expected, "default display");

    // Upper hex.
    let result = format!("{index:X}");
    let expected = "8A1FB46622DFFFF".to_owned();
    assert_eq!(result, expected, "upper hex");

    // Octal.
    let result = format!("{index:o}");
    let expected = "42417664314213377777".to_owned();
    assert_eq!(result, expected, "octal");

    // Binary.
    let result = format!("{index:b}");
    let expected =
        "100010100001111110110100011001100010001011011111111111111111"
            .to_owned();
    assert_eq!(result, expected, "binary");
}
