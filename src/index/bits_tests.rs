use super::*;
use crate::{CellIndex, Resolution};

#[test]
fn first_axe_none() {
    let index =
        CellIndex::try_from(0x8f2800000000000).expect("valid cell index");
    let result = first_axe(index.into());
    assert_eq!(result, None, "zero til the end");

    let index =
        CellIndex::try_from(0x8029fffffffffff).expect("valid cell index");
    let result = first_axe(index.into());
    assert_eq!(result, None, "no cell");

    let index =
        CellIndex::try_from(0x832800fffffffff).expect("valid cell index");
    let result = first_axe(index.into());
    assert_eq!(result, None, "some zero then unused");
}

#[test]
fn first_axe_some() {
    let resolutions = Resolution::range(Resolution::One, Resolution::Fifteen);
    let indexes = [
        0x8f287478ab9c2ab, // 20-1-6-4-3-6-1-2-5-6-3-4-1-2-5-3
        0x8f283478ab9c2ab, // 20-0-6-4-3-6-1-2-5-6-3-4-1-2-5-3
        0x8f280478ab9c2ab, // 20-0-0-4-3-6-1-2-5-6-3-4-1-2-5-3
        0x8f280078ab9c2ab, // 20-0-0-0-3-6-1-2-5-6-3-4-1-2-5-3
        0x8f280018ab9c2ab, // 20-0-0-0-0-6-1-2-5-6-3-4-1-2-5-3
        0x8f280000ab9c2ab, // 20-0-0-0-0-0-1-2-5-6-3-4-1-2-5-3
        0x8f2800002b9c2ab, // 20-0-0-0-0-0-0-2-5-6-3-4-1-2-5-3
        0x8f2800000b9c2ab, // 20-0-0-0-0-0-0-0-5-6-3-4-1-2-5-3
        0x8f280000019c2ab, // 20-0-0-0-0-0-0-0-0-6-3-4-1-2-5-3
        0x8f280000001c2ab, // 20-0-0-0-0-0-0-0-0-0-3-4-1-2-5-3
        0x8f28000000042ab, // 20-0-0-0-0-0-0-0-0-0-0-4-1-2-5-3
        0x8f28000000002ab, // 20-0-0-0-0-0-0-0-0-0-0-0-1-2-5-3
        0x8f28000000000ab, // 20-0-0-0-0-0-0-0-0-0-0-0-0-2-5-3
        0x8f280000000002b, // 20-0-0-0-0-0-0-0-0-0-0-0-0-0-5-3
        0x8f2800000000003, // 20-0-0-0-0-0-0-0-0-0-0-0-0-0-0-3
    ];

    for (resolution, value) in resolutions.zip(indexes.into_iter()) {
        let index = CellIndex::try_from(value).expect("valid cells");
        let expected = index.direction_at(resolution).and_then(Direction::axe);

        let result = first_axe(index.into());

        assert_eq!(result, expected, "resolution {resolution:?}");
    }
}
