use super::*;
use std::collections::HashSet;

macro_rules! exhaustive_test {
    ($name:ident, $parent_resolution:literal, $child_resolution:literal) => {
        #[test]
        fn $name() {
            let parent_resolution =
                Resolution::try_from($parent_resolution).unwrap();
            let child_resolution =
                Resolution::try_from($parent_resolution).unwrap();
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(parent_resolution))
            {
                assert_iterator_properties(index, child_resolution);
            }
        }
    };
}

// Exhaustive test of all cells, resolutions 0,1,2,3,4.
// Expanding to Gosper islands up to res 4.
exhaustive_test!(exhaustive_res0_to_0, 0, 0);
exhaustive_test!(exhaustive_res0_to_1, 0, 1);
exhaustive_test!(exhaustive_res0_to_2, 0, 2);
exhaustive_test!(exhaustive_res0_to_3, 0, 3);
exhaustive_test!(exhaustive_res0_to_4, 0, 4);
exhaustive_test!(exhaustive_res1_to_1, 1, 1);
exhaustive_test!(exhaustive_res1_to_2, 1, 2);
exhaustive_test!(exhaustive_res1_to_3, 1, 3);
exhaustive_test!(exhaustive_res1_to_4, 1, 4);
exhaustive_test!(exhaustive_res2_to_2, 2, 2);
exhaustive_test!(exhaustive_res2_to_3, 2, 3);
exhaustive_test!(exhaustive_res2_to_4, 2, 4);
exhaustive_test!(exhaustive_res3_to_3, 3, 3);
exhaustive_test!(exhaustive_res3_to_4, 3, 4);
exhaustive_test!(exhaustive_res4_to_4, 4, 4);

#[test]
fn finer_resolution() {
    let cells = [
        0x8508000ffffffff, // res 5 hexagon
        0x85080003fffffff, // res 5 pentagon
        0x8808000009fffff, // res 8 hexagon
        0x8808000001fffff, // res 8 pentagon
        0x8a0800000017fff, // res 10 hexagon
        0x8a0800000007fff, // res 10 pentagon
        0x8e754e64992d6c7, // res 14 hexagon
        0x8e0800000000007, // res 14 pentagon
        0x8f754e64992d6d8, // res 15 hexagon
        0x8f0800000000000, // res 15 pentagon
    ]
    .iter()
    .copied()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();

    for cell in cells {
        let parent_resolution = u8::from(cell.resolution());
        for gap in 0..=5 {
            if let Ok(children_resolution) =
                Resolution::try_from(parent_resolution + gap)
            {
                assert_iterator_properties(cell, children_resolution);
            }
        }
    }
}

#[test]
fn hexagon_2_to_2() {
    let cell = CellIndex::try_from(0x820887fffffffff).unwrap();
    let expected = [
        0x1320887fffffffff,
        0x1120887fffffffff,
        0x1520887fffffffff,
        0x1420887fffffffff,
        0x1620887fffffffff,
        0x1220887fffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();

    assert_edges(cell, Resolution::Two, &expected);
}

#[test]
fn pentagon_2_to_2() {
    let cell = CellIndex::try_from(0x820807fffffffff).unwrap();
    let expected = [
        0x1320807fffffffff,
        0x1520807fffffffff,
        0x1420807fffffffff,
        0x1620807fffffffff,
        0x1220807fffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();

    assert_edges(cell, Resolution::Two, &expected);
}

#[test]
fn hexagon_2_to_4() {
    let cell = CellIndex::try_from(0x82c64ffffffffff).unwrap();
    #[rustfmt::skip]
    let expected = [
        0x134c6497ffffffff, 0x114c6497ffffffff, 0x134c6493ffffffff,
        0x114c6493ffffffff, 0x154c6493ffffffff, 0x114c649bffffffff,
        0x154c649bffffffff, 0x144c649bffffffff, 0x154c6499ffffffff,
        0x114c64d3ffffffff, 0x154c64d3ffffffff, 0x114c64dbffffffff,
        0x154c64dbffffffff, 0x144c64dbffffffff, 0x154c64d9ffffffff,
        0x144c64d9ffffffff, 0x164c64d9ffffffff, 0x144c64ddffffffff,
        0x154c64cbffffffff, 0x144c64cbffffffff, 0x154c64c9ffffffff,
        0x144c64c9ffffffff, 0x164c64c9ffffffff, 0x144c64cdffffffff,
        0x164c64cdffffffff, 0x124c64cdffffffff, 0x164c64c5ffffffff,
        0x144c64e9ffffffff, 0x164c64e9ffffffff, 0x144c64edffffffff,
        0x164c64edffffffff, 0x124c64edffffffff, 0x164c64e5ffffffff,
        0x124c64e5ffffffff, 0x134c64e5ffffffff, 0x124c64e7ffffffff,
        0x164c64adffffffff, 0x124c64adffffffff, 0x164c64a5ffffffff,
        0x124c64a5ffffffff, 0x134c64a5ffffffff, 0x124c64a7ffffffff,
        0x134c64a7ffffffff, 0x114c64a7ffffffff, 0x134c64a3ffffffff,
        0x124c64b5ffffffff, 0x134c64b5ffffffff, 0x124c64b7ffffffff,
        0x134c64b7ffffffff, 0x114c64b7ffffffff, 0x134c64b3ffffffff,
        0x114c64b3ffffffff, 0x154c64b3ffffffff, 0x114c64bbffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<HashSet<_>>();

    let result = Gosper::new(cell, Resolution::Four).collect::<HashSet<_>>();

    assert_eq!(result, expected);
}

#[test]
fn pentagon_2_to_4() {
    let cell = CellIndex::try_from(0x820807fffffffff).unwrap();
    #[rustfmt::skip]
    let expected = [
        0x11408053ffffffff, 0x15408053ffffffff, 0x1140805bffffffff,
        0x1540805bffffffff, 0x1440805bffffffff, 0x15408059ffffffff,
        0x14408059ffffffff, 0x16408059ffffffff, 0x1440805dffffffff,
        0x1540804bffffffff, 0x1440804bffffffff, 0x15408049ffffffff,
        0x14408049ffffffff, 0x16408049ffffffff, 0x1440804dffffffff,
        0x1640804dffffffff, 0x1240804dffffffff, 0x16408045ffffffff,
        0x14408069ffffffff, 0x16408069ffffffff, 0x1440806dffffffff,
        0x1640806dffffffff, 0x1240806dffffffff, 0x16408065ffffffff,
        0x12408065ffffffff, 0x13408065ffffffff, 0x12408067ffffffff,
        0x1640802dffffffff, 0x1240802dffffffff, 0x16408025ffffffff,
        0x12408025ffffffff, 0x13408025ffffffff, 0x12408027ffffffff,
        0x13408027ffffffff, 0x11408027ffffffff, 0x13408023ffffffff,
        0x12408035ffffffff, 0x13408035ffffffff, 0x12408037ffffffff,
        0x13408037ffffffff, 0x11408037ffffffff, 0x13408033ffffffff,
        0x11408033ffffffff, 0x15408033ffffffff, 0x1140803bffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<HashSet<_>>();

    let result = Gosper::new(cell, Resolution::Four).collect::<HashSet<_>>();

    assert_eq!(result, expected);
}

#[test]
fn hexagon_1_to_3() {
    let cell = CellIndex::try_from(0x81c67ffffffffff).unwrap();
    #[rustfmt::skip]
    let expected = [
        0x133c64afffffffff, 0x123c64bfffffffff, 0x133c64bfffffffff,
        0x113c64bfffffffff, 0x133c649fffffffff, 0x113c649fffffffff,
        0x153c649fffffffff, 0x113c64dfffffffff, 0x153c64dfffffffff,
        0x113c66bfffffffff, 0x133c669fffffffff, 0x113c669fffffffff,
        0x153c669fffffffff, 0x113c66dfffffffff, 0x153c66dfffffffff,
        0x143c66dfffffffff, 0x153c66cfffffffff, 0x143c66cfffffffff,
        0x153c661fffffffff, 0x113c665fffffffff, 0x153c665fffffffff,
        0x143c665fffffffff, 0x153c664fffffffff, 0x143c664fffffffff,
        0x163c664fffffffff, 0x143c666fffffffff, 0x163c666fffffffff,
        0x143c675fffffffff, 0x153c674fffffffff, 0x143c674fffffffff,
        0x163c674fffffffff, 0x143c676fffffffff, 0x163c676fffffffff,
        0x123c676fffffffff, 0x163c672fffffffff, 0x123c672fffffffff,
        0x163c654fffffffff, 0x143c656fffffffff, 0x163c656fffffffff,
        0x123c656fffffffff, 0x163c652fffffffff, 0x123c652fffffffff,
        0x133c652fffffffff, 0x123c653fffffffff, 0x133c653fffffffff,
        0x123c65efffffffff, 0x163c65afffffffff, 0x123c65afffffffff,
        0x133c65afffffffff, 0x123c65bfffffffff, 0x133c65bfffffffff,
        0x113c65bfffffffff, 0x133c659fffffffff, 0x113c659fffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<HashSet<_>>();

    let result = Gosper::new(cell, Resolution::Three).collect::<HashSet<_>>();

    assert_eq!(result, expected);
}

#[test]
fn pentagon_1_to_3() {
    let cell = CellIndex::try_from(0x81083ffffffffff).unwrap();
    #[rustfmt::skip]
    let expected = [
        0x113082bfffffffff, 0x1330829fffffffff, 0x1130829fffffffff,
        0x1530829fffffffff, 0x113082dfffffffff, 0x153082dfffffffff,
        0x143082dfffffffff, 0x153082cfffffffff, 0x143082cfffffffff,
        0x1530821fffffffff, 0x1130825fffffffff, 0x1530825fffffffff,
        0x1430825fffffffff, 0x1530824fffffffff, 0x1430824fffffffff,
        0x1630824fffffffff, 0x1430826fffffffff, 0x1630826fffffffff,
        0x1430835fffffffff, 0x1530834fffffffff, 0x1430834fffffffff,
        0x1630834fffffffff, 0x1430836fffffffff, 0x1630836fffffffff,
        0x1230836fffffffff, 0x1630832fffffffff, 0x1230832fffffffff,
        0x1630814fffffffff, 0x1430816fffffffff, 0x1630816fffffffff,
        0x1230816fffffffff, 0x1630812fffffffff, 0x1230812fffffffff,
        0x1330812fffffffff, 0x1230813fffffffff, 0x1330813fffffffff,
        0x123081efffffffff, 0x163081afffffffff, 0x123081afffffffff,
        0x133081afffffffff, 0x123081bfffffffff, 0x133081bfffffffff,
        0x113081bfffffffff, 0x1330819fffffffff, 0x1130819fffffffff,
    ]
    .iter()
    .copied()
    .map(|value| DirectedEdgeIndex::try_from(value).unwrap())
    .collect::<HashSet<_>>();

    let result = Gosper::new(cell, Resolution::Three).collect::<HashSet<_>>();

    assert_eq!(result, expected);
}

// -----------------------------------------------------------------------------

fn assert_iterator_properties(index: CellIndex, child_resolution: Resolution) {
    let parent_resolution = index.resolution();
    let delta = u8::from(child_resolution) - u8::from(parent_resolution);
    let nb_sides = if index.is_pentagon() { 5 } else { 6 };
    let expected_count = nb_sides * 3_usize.pow(delta.into());

    let mut iter = Gosper::new(index, child_resolution);
    let mut edges = HashSet::with_capacity(expected_count);

    let first = iter.clone().next().unwrap();

    let mut prev = None;
    for edge in &mut iter {
        // Property 1: each edge is unique.
        assert!(edges.insert(edge));

        // Property 2: each edge is at `child_resolution`.
        assert_eq!(edge.origin().resolution(), child_resolution);

        // Property 3. the edge is on the Gosper island boundary.
        // => origin cell is a child of parent cell, destination cell is not.
        let origin = edge.origin();
        let destination = edge.destination();
        let parent_origin = origin.parent(parent_resolution).unwrap();
        let parent_destination = destination.parent(parent_resolution).unwrap();
        assert_eq!(parent_origin, index, "origin *is* a child of parent cell");
        assert_ne!(
            parent_destination, index,
            "destination *is not* a child of parent cell"
        );

        // Property 4: consecutive edges share an endpoint.
        if let Some(prev) = prev {
            assert_edge_connect(prev, edge);
        }
        prev = Some(edge);
    }
    // Property 5: loop closes (returns to where it started).
    assert_edge_connect(prev.unwrap(), first);
    // Property 6: the expected edges count is produced (not more, not less).
    assert_eq!(edges.len(), expected_count, "incorrect number of edges");

    // Property 7: iterator should stay exhausted (fused behavior).
    for _ in 0..100 {
        assert!(iter.next().is_none());
    }
}

// Check that edge_a's last vertex matches edge_b's first vertex, i.e. the edges
// connect end-to-start. Tolerance is relative to the shorter edge length to
// handle varying resolutions.
//
// TODO: Replace floating-point distance check with exact vertex comparison once
// an `edgeToVertexes` function exists.
fn assert_edge_connect(edge_a: DirectedEdgeIndex, edge_b: DirectedEdgeIndex) {
    let len_a = edge_a.length_rads();
    let len_b = edge_b.length_rads();
    let tolerance = len_a.min(len_b) / 1000.0;

    let end_a = edge_a.boundary().last().copied().unwrap();
    let start_b = edge_b.boundary().first().copied().unwrap();

    let distance = end_a.distance_rads(start_b);

    assert!(distance < tolerance);
}

// Assert that the Gosper iterator produces the expected edges.
//
// Order is checked, but starting point may vary.
fn assert_edges(
    index: CellIndex,
    resolution: Resolution,
    expected: &[DirectedEdgeIndex],
) {
    let mut result = Gosper::new(index, resolution).collect::<Vec<_>>();

    assert_eq!(result.len(), expected.len(), "wrong number of edges");

    let Some(first) = expected.first() else {
        return;
    };
    let offset = result
        .iter()
        .position(|edge| edge == first)
        .expect("first edge not found");
    result.rotate_left(offset);

    assert_eq!(result, expected, "edges mismatch");
}
