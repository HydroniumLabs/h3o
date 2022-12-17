use super::h3api;
use h3o::{CellIndex, Resolution};

fn assert_compact(cells: Vec<CellIndex>) {
    let result = CellIndex::compact(cells.iter().copied())
        .map(|iter| iter.collect::<Vec<_>>())
        .ok()
        .map(|mut cells| {
            cells.sort_unstable();
            cells
        });
    let reference = h3api::compact_cells(&cells).map(|mut cells| {
        cells.sort_unstable();
        cells
    });

    assert_eq!(result, reference);
}

#[test]
fn grid_disk() {
    let index = CellIndex::try_from(0x89283470c27ffff).expect("cell index");
    let cells = index.grid_disk::<Vec<_>>(9);

    assert_compact(cells);
}

#[test]
fn res0() {
    let cells = CellIndex::base_cells().collect::<Vec<_>>();

    assert_compact(cells);
}

#[test]
fn res0_children() {
    let index = CellIndex::try_from(0x8001fffffffffff).expect("cell index");
    let cells = index.children(Resolution::One).collect::<Vec<_>>();

    assert_compact(cells);
}

#[test]
fn children() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("cell index");
    let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();

    assert_compact(cells);
}

#[test]
fn uncompactable() {
    let cells = [0x89283470803ffff, 0x8928347081bffff, 0x8928347080bffff]
        .into_iter()
        .map(|hex| CellIndex::try_from(hex).expect("cell index"))
        .collect();

    assert_compact(cells);
}

#[test]
fn duplicate() {
    let cells =
        vec![CellIndex::try_from(0x8500924bfffffff).expect("cell index"); 10];

    assert_compact(cells);
}

#[test]
fn duplicate_minimum() {
    let index = CellIndex::try_from(0x8a0092492497fff).expect("cell index");
    let mut cells = index.children(Resolution::Eleven).collect::<Vec<_>>();
    let cell = cells[0];
    cells.push(cell);

    assert_compact(cells);
}

#[test]
fn duplicate_pentagon_limit() {
    let index = CellIndex::try_from(0x8a0800000007fff).expect("cell index");
    let mut cells = index.children(Resolution::Eleven).collect::<Vec<_>>();
    let cell = cells[0];
    cells.push(cell);

    assert_compact(cells);
}

#[test]
fn duplicate_minimum_bis() {
    let index = CellIndex::try_from(0x8a0092492497fff).expect("cell index");
    let mut cells = index.children(Resolution::Eleven).collect::<Vec<_>>();
    let cell = cells[0];
    if let Some(last) = cells.last_mut() {
        *last = cell;
    }

    // This duplicate is not detected by H3 (see their test `compactCells_duplicateIgnored`).
    // But we do.
    let result = CellIndex::compact(cells);

    assert!(result.is_err());
}

#[test]
fn empty() {
    let cells = Vec::new();

    assert_compact(cells);
}

#[test]
fn disparate() {
    let cells = [
        0x81003ffffffffff,
        0x81023ffffffffff,
        0x81043ffffffffff,
        0x81063ffffffffff,
        0x81083ffffffffff,
        0x810a3ffffffffff,
        0x810c3ffffffffff,
    ]
    .into_iter()
    .map(|hex| CellIndex::try_from(hex).expect("cell index"))
    .collect();

    assert_compact(cells);
}

#[test]
fn resolution_mismatch() {
    let cells = [0x81003ffffffffff, 0x8500924bfffffff, 0x89283470803ffff]
        .into_iter()
        .map(|hex| CellIndex::try_from(hex).expect("cell index"));

    // This duplicate is not detected by H3, but we do.
    let result = CellIndex::compact(cells);

    assert!(result.is_err());
}
