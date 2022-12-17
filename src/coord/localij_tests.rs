use super::*;

#[test]
fn from_ijk_zero() {
    let ijk = CoordIJK::new(0, 0, 0);
    let ij = CoordIJ::from(&ijk);

    assert_eq!(ij.i, 0, "ij.i zero");
    assert_eq!(ij.j, 0, "ij.j zero");
}
