use super::*;
use core::convert::TryFrom;

#[test]
fn disk_distance_safe_is_fused() {
    let origin = CellIndex::try_from(0x85002c27fffffff).unwrap();
    let mut iter = DiskDistancesSafe::new(origin, 1);

    for _ in 0..7 {
        assert!(iter.next().is_some());
    }
    for _ in 0..7 {
        assert!(iter.next().is_none());
    }
}

#[test]
fn disk_distance_unsafe_stop_after_none() {
    let origin = CellIndex::try_from(0x8908000001bffff).expect("origin");
    let iter = DiskDistancesUnsafe::new(origin, 1);

    // Only 5 because a pentagon was encountered.
    assert_eq!(iter.count(), 5);
}

#[test]
fn disk_distance_unsafe_is_fused() {
    let origin = CellIndex::try_from(0x85002c27fffffff).unwrap();
    let mut iter = DiskDistancesUnsafe::new(origin, 1);

    for _ in 0..7 {
        assert!(iter.next().is_some());
    }
    for _ in 0..7 {
        assert!(iter.next().is_none());
    }
}

#[test]
fn ring_unsafe_is_fused() {
    let origin = CellIndex::try_from(0x85002c27fffffff).unwrap();
    let mut iter = RingUnsafe::new(origin, 1).unwrap();

    for _ in 0..6 {
        assert!(iter.next().is_some());
    }
    for _ in 0..6 {
        assert!(iter.next().is_none());
    }
}
