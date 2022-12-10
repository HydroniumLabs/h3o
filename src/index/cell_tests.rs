use super::*;

#[test]
fn direction_at() {
    use Resolution::{
        Eight, Eleven, Fifteen, Five, Four, Fourteen, Nine, One, Seven, Six,
        Ten, Thirteen, Three, Twelve, Two, Zero,
    };

    let cell = CellIndex::new_unchecked(0x8c2bae305336bff);

    assert_eq!(cell.resolution(), Twelve);

    assert_eq!(cell.direction_at(Zero), None);
    assert_eq!(cell.direction_at(One), Some(Direction::IJ));
    assert_eq!(cell.direction_at(Two), Some(Direction::IK));
    assert_eq!(cell.direction_at(Three), Some(Direction::IJ));
    assert_eq!(cell.direction_at(Four), Some(Direction::K));
    assert_eq!(cell.direction_at(Five), Some(Direction::I));
    assert_eq!(cell.direction_at(Six), Some(Direction::Center));
    assert_eq!(cell.direction_at(Seven), Some(Direction::IK));
    assert_eq!(cell.direction_at(Eight), Some(Direction::K));
    assert_eq!(cell.direction_at(Nine), Some(Direction::I));
    assert_eq!(cell.direction_at(Ten), Some(Direction::IJ));
    assert_eq!(cell.direction_at(Eleven), Some(Direction::IJ));
    assert_eq!(cell.direction_at(Twelve), Some(Direction::IK));
    assert_eq!(cell.direction_at(Thirteen), None);
    assert_eq!(cell.direction_at(Fourteen), None);
    assert_eq!(cell.direction_at(Fifteen), None);
}

#[test]
fn ordering() {
    let mut cells = vec![
        CellIndex::new_unchecked(0x8a194e699ab7fff),
        CellIndex::new_unchecked(0x89194e69d4fffff),
    ];
    let expected = vec![
        CellIndex::new_unchecked(0x8a194e699ab7fff),
        CellIndex::new_unchecked(0x89194e69d4fffff),
    ];

    cells.sort_unstable();

    // Make sure that
    // `0x89194e69d4fffff` (12-5-1-6-3-2-3-5-2-3) comes AFTER
    // `0x8a194e699ab7fff` (12-5-1-6-3-2-3-1-5-2-6) when sorting.
    assert_eq!(cells, expected);
}

#[test]
fn debug_impl() {
    assert_eq!(
        format!("{:?}", CellIndex::new_unchecked(0x802bfffffffffff)),
        "21-777777777777777 (802bfffffffffff)"
    );
    assert_eq!(
        format!("{:?}", CellIndex::new_unchecked(0x8c2bae305336bff)),
        "21-656140514665777 (8c2bae305336bff)"
    );
    assert_eq!(
        format!("{:?}", CellIndex::new_unchecked(0x8f2834782b9c2ab)),
        "20-064360256341253 (8f2834782b9c2ab)"
    );
}
