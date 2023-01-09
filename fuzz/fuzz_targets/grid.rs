#![no_main]

use h3o::{max_grid_disk_size, CellIndex};
use libfuzzer_sys::fuzz_target;
use std::os::raw::c_int;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    index: CellIndex,
    k: u32,
}

fuzz_target!(|args: Args| {
    // Avoid overly large output (very slow, OOM risk).
    if max_grid_disk_size(args.k) > 10_000 {
        return;
    }

    let mut res = args.index.grid_disk::<Vec<_>>(args.k);
    res.sort_unstable();
    let mut h3 = grid_disk(args.index, args.k);
    h3.sort_unstable();
    assert_eq!(res, h3, "grid_disk");

    assert_eq!(
        args.index
            .grid_disk_fast(args.k)
            .collect::<Option<Vec<_>>>(),
        grid_disk_unsafe(args.index, args.k),
        "gridDiskUnsafe"
    );

    assert_eq!(
        args.index
            .grid_disk_distances_fast(args.k)
            .collect::<Option<Vec<_>>>(),
        grid_disk_distances_unsafe(args.index, args.k),
        "gridDiskDistancesUnsafe"
    );

    let mut res = args
        .index
        .grid_disk_distances_safe(args.k)
        .collect::<Vec<_>>();
    res.sort_unstable_by_key(|(cell, _)| *cell);
    let mut h3 = grid_disk_distances_safe(args.index, args.k);
    h3.sort_unstable_by_key(|(cell, _)| *cell);
    assert_eq!(res, h3, "gridDiskDistancesSafe");

    let mut res = args.index.grid_disk_distances::<Vec<_>>(args.k);
    res.sort_unstable_by_key(|(cell, _)| *cell);
    let mut h3 = grid_disk_distances(args.index, args.k);
    h3.sort_unstable_by_key(|(cell, _)| *cell);
    assert_eq!(res, h3, "gridDiskDistances");

    assert_eq!(
        args.index
            .grid_ring_fast(args.k)
            .collect::<Option<Vec<_>>>(),
        grid_ring_unsafe(args.index, args.k),
        "gridRingUnsafe"
    );

    assert_eq!(
        CellIndex::grid_disks_fast([args.index].iter().copied(), args.k)
            .collect::<Option<Vec<_>>>(),
        grid_disks_unsafe([args.index].iter().copied(), args.k),
        "gridDisksUnsafe"
    );
});

// H3 wrappers {{{

fn grid_disk(origin: CellIndex, k: u32) -> Vec<CellIndex> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDisk(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistances");
    }

    cells
        .into_iter()
        .filter_map(|cell| {
            (cell != 0).then(|| CellIndex::try_from(cell).expect("cell index"))
        })
        .collect()
}

fn grid_disk_distances(origin: CellIndex, k: u32) -> Vec<(CellIndex, u32)> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDiskDistances(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistances");
    }

    cells
        .into_iter()
        .zip(distances.into_iter())
        .filter_map(|(cell, distance)| {
            (cell != 0).then(|| {
                (
                    CellIndex::try_from(cell).expect("cell index"),
                    distance as u32,
                )
            })
        })
        .collect()
}

fn grid_disk_distances_safe(
    origin: CellIndex,
    k: u32,
) -> Vec<(CellIndex, u32)> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDiskDistancesSafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistancesSafe");
    }

    cells
        .into_iter()
        .zip(distances.into_iter())
        .filter_map(|(cell, distance)| {
            (cell != 0).then(|| {
                (
                    CellIndex::try_from(cell).expect("cell index"),
                    distance as u32,
                )
            })
        })
        .collect()
}

fn grid_disk_distances_unsafe(
    origin: CellIndex,
    k: u32,
) -> Option<Vec<(CellIndex, u32)>> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDiskDistancesUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .zip(distances.into_iter())
            .filter_map(|(cell, distance)| {
                (cell != 0).then(|| {
                    (
                        CellIndex::try_from(cell).expect("cell index"),
                        distance as u32,
                    )
                })
            })
            .collect()
    })
}

fn grid_disk_unsafe(origin: CellIndex, k: u32) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDiskUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

fn grid_disks_unsafe(
    origins: impl IntoIterator<Item = CellIndex>,
    k: u32,
) -> Option<Vec<CellIndex>> {
    let mut origins = origins.into_iter().map(u64::from).collect::<Vec<_>>();
    let size = origins.len()
        * usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDisksUnsafe(
            origins.as_mut_ptr(),
            origins.len() as c_int,
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

fn grid_ring_unsafe(origin: CellIndex, k: u32) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(if k == 0 { 1 } else { 6 * k })
        .expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridRingUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

// }}}
