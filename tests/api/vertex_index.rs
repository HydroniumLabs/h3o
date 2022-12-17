use h3o::VertexIndex;

#[test]
fn try_from_str() {
    let result = "2222597fffffffff".parse::<VertexIndex>();
    let expected = VertexIndex::try_from(0x2222597fffffffff);
    assert_eq!(result, expected, "valid string");

    let result = "no bueno".parse::<VertexIndex>();
    assert!(result.is_err(), "invalid string");
}

// Resolutions are displayed as numerical value.
#[test]
fn display() {
    let index = VertexIndex::try_from(0x2222597fffffffff).expect("index");

    // Default display is the lower hex one.
    let result = index.to_string();
    let expected = "2222597fffffffff".to_owned();
    assert_eq!(result, expected, "default display");

    // Upper hex.
    let result = format!("{index:X}");
    let expected = "2222597FFFFFFFFF".to_owned();
    assert_eq!(result, expected, "upper hex");

    // Octal.
    let result = format!("{index:o}");
    let expected = "210422627777777777777".to_owned();
    assert_eq!(result, expected, "octal");

    // Binary.
    let result = format!("{index:b}");
    let expected =
        "10001000100010010110010111111111111111111111111111111111111111"
            .to_owned();
    assert_eq!(result, expected, "binary");
}
