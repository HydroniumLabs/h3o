use super::*;

#[test]
fn zero_is_invalid() {
    assert!(DirectedEdgeIndex::try_from(0).is_err());
}

#[test]
fn edge() {
    assert!(Edge::try_from(0).is_err()); // Out of range.
    assert!(Edge::try_from(1).is_ok()); // Lower bound.
    assert!(Edge::try_from(3).is_ok()); // Valid value.
    assert!(Edge::try_from(6).is_ok()); // Upper bound.
    assert!(Edge::try_from(7).is_err()); // Out of range.

    assert_eq!(u8::from(Edge(1)), 1); // Lower bound.
    assert_eq!(u8::from(Edge(3)), 3); // Valid value.
    assert_eq!(u8::from(Edge(6)), 6); // Upper bound.
}

#[test]
fn ordering_by_index() {
    let mut cells = vec![
        DirectedEdgeIndex::new_unchecked(0x13a194e699ab7fff),
        DirectedEdgeIndex::new_unchecked(0x139194e69d4fffff),
    ];
    let expected = vec![
        DirectedEdgeIndex::new_unchecked(0x13a194e699ab7fff),
        DirectedEdgeIndex::new_unchecked(0x139194e69d4fffff),
    ];

    cells.sort_unstable();

    // Make sure that
    // `0x139194e69d4fffff` (12-5-1-6-3-2-3-5-2-3) comes AFTER
    // `0x13a194e699ab7fff` (12-5-1-6-3-2-3-1-5-2-6) when sorting.
    assert_eq!(cells, expected);
}

#[test]
fn ordering_by_edge() {
    let mut cells = vec![
        DirectedEdgeIndex::new_unchecked(0x159194e69d4fffff),
        DirectedEdgeIndex::new_unchecked(0x139194e69d4fffff),
    ];
    let expected = vec![
        DirectedEdgeIndex::new_unchecked(0x139194e69d4fffff),
        DirectedEdgeIndex::new_unchecked(0x159194e69d4fffff),
    ];

    cells.sort_unstable();

    // Make sure that
    // `0x139194e69d4fffff` (12-5-1-6-3-2-3-5-2-3/E3) comes AFTER
    // `0x159194e69d4fffff` (12-5-1-6-3-2-3-5-2-3/E5).
    assert_eq!(cells, expected);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", DirectedEdgeIndex::new_unchecked(0x1302bfffffffffff)),
        "21-777777777777777_3 (1302bfffffffffff)"
    );
    assert_eq!(
        format!("{:?}", DirectedEdgeIndex::new_unchecked(0x13c2bae305336bff)),
        "21-656140514665777_3 (13c2bae305336bff)"
    );
    assert_eq!(
        format!("{:?}", DirectedEdgeIndex::new_unchecked(0x15f2834782b9c2ab)),
        "20-064360256341253_5 (15f2834782b9c2ab)"
    );
}
