use std::convert::TryFrom;

use super::*;

#[test]
fn stop_after_none() {
    let origin = CellIndex::try_from(0x8908000001bffff).expect("origin");
    let iter = DiskDistancesUnsafe::new(origin, 1);

    assert_eq!(iter.count(), 5);
}
