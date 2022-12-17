use super::*;

#[test]
fn zero_is_invalid() {
    assert!(VertexIndex::try_from(0).is_err());
}

#[test]
fn vertex() {
    assert!(Vertex::try_from(0).is_ok()); // Lower bound.
    assert!(Vertex::try_from(3).is_ok()); // Valid value.
    assert!(Vertex::try_from(5).is_ok()); // Upper bound.
    assert!(Vertex::try_from(6).is_err()); // Out of range.

    assert_eq!(u8::from(Vertex(0)), 0); // Lower bound.
    assert_eq!(u8::from(Vertex(3)), 3); // Valid value.
    assert_eq!(u8::from(Vertex(5)), 5); // Upper bound.
}

#[test]
fn ordering_by_index() {
    let mut cells = vec![
        VertexIndex::new_unchecked(0x25a194e699ab7fff),
        VertexIndex::new_unchecked(0x259194e69d4fffff),
    ];
    let expected = vec![
        VertexIndex::new_unchecked(0x25a194e699ab7fff),
        VertexIndex::new_unchecked(0x259194e69d4fffff),
    ];

    cells.sort_unstable();

    // Make sure that
    // `0x259194e69d4fffff` (12-5-1-6-3-2-3-5-2-3) comes AFTER
    // `0x25a194e699ab7fff` (12-5-1-6-3-2-3-1-5-2-6) when sorting.
    assert_eq!(cells, expected);
}

#[test]
fn ordering() {
    let mut cells = vec![
        VertexIndex::new_unchecked(0x259194e69d4fffff),
        VertexIndex::new_unchecked(0x239194e69d4fffff),
    ];
    let expected = vec![
        VertexIndex::new_unchecked(0x239194e69d4fffff),
        VertexIndex::new_unchecked(0x259194e69d4fffff),
    ];

    cells.sort_unstable();

    // Make sure that
    // `0x259194e69d4fffff` (12-5-1-6-3-2-3-5-2-3/V5) comes AFTER
    // `0x239194e69d4fffff` (12-5-1-6-3-2-3-5-2-3/V3)
    assert_eq!(cells, expected);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", VertexIndex::new_unchecked(0x2302bfffffffffff)),
        "21-777777777777777_3 (2302bfffffffffff)"
    );
    assert_eq!(
        format!("{:?}", VertexIndex::new_unchecked(0x23c2bae305336bff)),
        "21-656140514665777_3 (23c2bae305336bff)"
    );
    assert_eq!(
        format!("{:?}", VertexIndex::new_unchecked(0x25f2834782b9c2ab)),
        "20-064360256341253_5 (25f2834782b9c2ab)"
    );
}
