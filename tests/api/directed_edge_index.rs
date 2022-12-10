use h3o::DirectedEdgeIndex;

#[test]
fn try_from_str() {
    let result = "13a194e699ab7fff".parse::<DirectedEdgeIndex>();
    let expected = DirectedEdgeIndex::try_from(0x13a194e699ab7fff);
    assert_eq!(result, expected, "valid string");

    let result = "no bueno".parse::<DirectedEdgeIndex>();
    assert!(result.is_err(), "invalid string");
}

// Resolutions are displayed as numerical value.
#[test]
fn display() {
    let index = DirectedEdgeIndex::try_from(0x13a194e699ab7fff).expect("index");

    // Default display is the lower hex one.
    let result = index.to_string();
    let expected = "13a194e699ab7fff".to_owned();
    assert_eq!(result, expected, "default display");

    // Upper hex.
    let result = format!("{index:X}");
    let expected = "13A194E699AB7FFF".to_owned();
    assert_eq!(result, expected, "upper hex");

    // Octal.
    let result = format!("{index:o}");
    let expected = "116414516323152677777".to_owned();
    assert_eq!(result, expected, "octal");

    // Binary.
    let result = format!("{index:b}");
    let expected =
        "1001110100001100101001110011010011001101010110111111111111111"
            .to_owned();
    assert_eq!(result, expected, "binary");
}
