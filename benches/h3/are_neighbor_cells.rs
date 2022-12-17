use criterion::{black_box, Bencher, Criterion};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("areNeighborCells");

    let (origin, index) = (0x0890153a1017ffff, 0x890153a1003ffff);
    group.bench_function("h3o/SameParentCenter", |b| {
        bench_h3o(b, index, origin)
    });
    group.bench_function("h3/SameParentCenter", |b| bench_h3(b, index, origin));

    let (origin, index) = (0x0890153a1017ffff, 0x0890153a1013ffff);
    group
        .bench_function("h3o/SameParentOther", |b| bench_h3o(b, index, origin));
    group.bench_function("h3/SameParentOther", |b| bench_h3(b, index, origin));

    // This pair uses the fast unsafe implementation of grid disk.
    let (origin, index) = (0x0890153a1017ffff, 0x0890153a10bbffff);
    group
        .bench_function("h3o/DifferentParent", |b| bench_h3o(b, index, origin));
    group.bench_function("h3/DifferentParent", |b| bench_h3(b, index, origin));

    // This pair uses the slow safe implementation of grid disk.
    let (origin, index) = (0x08908000001bffff, 0x08908000000fffff);
    group.bench_function("h3o/DifferentParentFallback", |b| {
        bench_h3o(b, index, origin)
    });
    group.bench_function("h3/DifferentParentFallback", |b| {
        bench_h3(b, index, origin)
    });

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: u64, origin: u64) {
    let origin = CellIndex::try_from(origin).expect("origin");
    let index = CellIndex::try_from(index).expect("index");
    b.iter(|| black_box(origin).is_neighbor_with(black_box(index)))
}

fn bench_h3(b: &mut Bencher<'_>, index: u64, origin: u64) {
    let mut out = 0;
    b.iter(|| unsafe {
        h3ron_h3_sys::areNeighborCells(
            black_box(origin),
            black_box(index),
            &mut out,
        )
    })
}
