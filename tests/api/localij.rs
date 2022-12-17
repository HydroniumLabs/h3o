use h3o::{CellIndex, LocalIJ};

#[test]
fn display() {
    let anchor = CellIndex::try_from(0x8508282bfffffff).expect("cell");
    let local_ij = LocalIJ::new_unchecked(anchor, -4, -3);
    let expected = "8508282bfffffff (-4, -3)".to_owned();

    assert_eq!(local_ij.to_string(), expected);
}
