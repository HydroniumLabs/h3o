use super::*;
use core::convert::TryFrom;

#[test]
fn disk_distance_safe_is_fused() {
    let origin = CellIndex::try_from(0x85002c27fffffff).unwrap();
    let destination = CellIndex::try_from(0x85015337fffffff).unwrap();
    let mut iter = GridPathCells::new(origin, destination).unwrap();

    for _ in 0..9 {
        assert!(iter.next().is_some());
    }
    for _ in 0..9 {
        assert!(iter.next().is_none());
    }
}
