use super::*;
use core::convert::TryFrom;

#[test]
fn disk_distance_safe_is_fused() {
    let origin = CellIndex::try_from(0x85002c27fffffff).unwrap();
    let mut iter = Children::new(origin, Resolution::Six);

    for _ in 0..7 {
        assert!(iter.next().is_some());
    }
    for _ in 0..7 {
        assert!(iter.next().is_none());
    }
}
