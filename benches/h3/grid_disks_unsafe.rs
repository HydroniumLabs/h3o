use criterion::{black_box, Criterion};
use h3o::CellIndex;
use std::os::raw::c_int;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("gridDisksUnsafe");
    let mut indexes = vec![
        0x89283080ddbffff,
        0x89283080c37ffff,
        0x89283080c27ffff,
        0x89283080d53ffff,
        0x89283080dcfffff,
        0x89283080dc3ffff,
    ];

    group.bench_function("h3o", |b| {
        let indexes = indexes
            .iter()
            .map(|cell| CellIndex::try_from(*cell).expect("hex index"))
            .collect::<Vec<_>>();
        b.iter(|| {
            CellIndex::grid_disks_fast(
                black_box(indexes.iter().copied()),
                black_box(2),
            )
            .for_each(drop)
        })
    });
    group.bench_function("h3", |b| {
        let size = indexes.len()
            * usize::try_from(h3o::max_grid_disk_size(2))
                .expect("grid too large");
        let mut cells = vec![0; size];
        b.iter(|| unsafe {
            h3ron_h3_sys::gridDisksUnsafe(
                black_box(indexes.as_mut_ptr()),
                black_box(indexes.len() as c_int),
                black_box(2),
                cells.as_mut_ptr(),
            )
        })
    });

    group.finish();
}
