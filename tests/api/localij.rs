use h3o::{CellIndex, CoordIJ, LocalIJ};

#[test]
fn display() {
    let anchor = CellIndex::try_from(0x8508282bfffffff).expect("cell");
    let local_ij = LocalIJ::new(anchor, CoordIJ::new(-4, -3));
    let expected = "8508282bfffffff (-4, -3)".to_owned();

    assert_eq!(local_ij.to_string(), expected);
}

#[test]
fn to_cell_overflow() {
    let origin_res2 = CellIndex::try_from(0x820407fffffffff).expect("cell");

    let ij = CoordIJ::new(553648127, -2145378272);
    assert!(CellIndex::try_from(LocalIJ::new(origin_res2, ij)).is_err());

    let ij = CoordIJ::new(i32::MAX - 10, -11);
    assert!(CellIndex::try_from(LocalIJ::new(origin_res2, ij)).is_err());

    let origin_res3 = CellIndex::try_from(0x830400fffffffff).expect("cell");
    let ij = CoordIJ::new(553648127, -2145378272);
    assert!(CellIndex::try_from(LocalIJ::new(origin_res3, ij)).is_err());

    let ij = CoordIJ::new(i32::MAX - 10, -10);
    assert!(CellIndex::try_from(LocalIJ::new(origin_res3, ij)).is_err());

    let ij = CoordIJ::new(i32::MAX - 10, -9);
    assert!(CellIndex::try_from(LocalIJ::new(origin_res3, ij)).is_err());
}
