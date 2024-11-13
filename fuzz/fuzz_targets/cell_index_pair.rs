#![no_main]

use h3o::{CellIndex, DirectedEdgeIndex, LocalIJ};
use libfuzzer_sys::fuzz_target;
use std::os::raw::c_int;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    index1: CellIndex,
    index2: CellIndex,
}

fuzz_target!(|args: Args| {
    // Note that index1 and index2 need to be plausibly neighbors.
    assert_eq!(
        args.index1.is_neighbor_with(args.index2).ok(),
        are_neighbor_cells(args.index1, args.index2),
        "areNeighborCells"
    );

    assert_eq!(
        args.index1.edge(args.index2),
        cells_to_directed_edge(args.index1, args.index2),
        "cellsToDirectedEdge"
    );

    // Note that index and anchor need to be in the approximate area for these
    // tests to make sense.
    assert_eq!(
        args.index1.grid_distance(args.index2).unwrap_or(-1),
        grid_distance(args.index1, args.index2).unwrap_or(-1),
        "gridDistance"
    );
    assert_eq!(
        args.index1
            .grid_path_cells(args.index2)
            .ok()
            .and_then(|iter| iter.collect::<Result<Vec<_>, _>>().ok()),
        grid_path_cells(args.index1, args.index2),
        "gridPathCells"
    );

    if let Ok(coord) = args.index1.to_local_ij(args.index2) {
        assert_eq!(
            Some(coord),
            cell_to_local_ij(args.index2, args.index1),
            "cellToLocalIj"
        );
        assert_eq!(
            CellIndex::try_from(coord),
            Ok(args.index1),
            "localIjToCell"
        );
    }
});

// H3 wrappers {{{

fn are_neighbor_cells(origin: CellIndex, index: CellIndex) -> Option<bool> {
    let mut out: c_int = 0;
    let res = unsafe {
        h3ron_h3_sys::areNeighborCells(origin.into(), index.into(), &mut out)
    };
    (res == 0).then_some(out == 1)
}

fn cells_to_directed_edge(
    origin: CellIndex,
    destination: CellIndex,
) -> Option<DirectedEdgeIndex> {
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellsToDirectedEdge(
            origin.into(),
            destination.into(),
            &mut out,
        )
    };
    (res == 0).then(|| DirectedEdgeIndex::try_from(out).expect("edge index"))
}

fn grid_distance(src: CellIndex, dst: CellIndex) -> Option<i32> {
    let mut out = 0;
    let res =
        unsafe { h3ron_h3_sys::gridDistance(src.into(), dst.into(), &mut out) };
    (res == 0).then(|| i32::try_from(out).expect("distance overflow"))
}

fn grid_path_cells(src: CellIndex, dst: CellIndex) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(src.grid_path_cells_size(dst).ok()?)
        .expect("path too long");
    let mut out = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridPathCells(src.into(), dst.into(), out.as_mut_ptr())
    };

    (res == 0).then(|| {
        out.into_iter()
            .map(|cell| CellIndex::try_from(cell).expect("cell index"))
            .collect()
    })
}

fn cell_to_local_ij(origin: CellIndex, index: CellIndex) -> Option<LocalIJ> {
    let mut out = h3ron_h3_sys::CoordIJ { i: 0, j: 0 };
    let res = unsafe {
        h3ron_h3_sys::cellToLocalIj(origin.into(), index.into(), 0, &mut out)
    };
    (res == 0).then(|| LocalIJ::new(origin, h3o::CoordIJ::new(out.i, out.j)))
}

// }}}
