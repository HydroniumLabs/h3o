use criterion::{black_box, Bencher, Criterion};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellToLocalIj");

    let (anchor, index) = (0x823147fffffffff, 0x8230e7fffffffff);
    group.bench_function("h3o/Hexagon", |b| bench_h3o(b, anchor, index));
    group.bench_function("h3/Hexagon", |b| bench_h3(b, anchor, index));

    let (anchor, index) = (0x821f57fffffffff, 0x8208d7fffffffff);
    group.bench_function("h3o/Pentagon", |b| bench_h3o(b, anchor, index));
    group.bench_function("h3/Pentagon", |b| bench_h3(b, anchor, index));

    let (anchor, index) = (0x823147fffffffff, 0x8230e7fffffffff);
    group.bench_function("h3o/SameBase", |b| bench_h3o(b, anchor, index));
    group.bench_function("h3/SameBase", |b| bench_h3(b, anchor, index));

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, anchor: u64, index: u64) {
    let anchor = CellIndex::try_from(anchor).expect("anchor");
    let index = CellIndex::try_from(index).expect("index");
    b.iter(|| black_box(index).to_local_ij(black_box(anchor)))
}

fn bench_h3(b: &mut Bencher<'_>, anchor: u64, index: u64) {
    let mut out = h3ron_h3_sys::CoordIJ { i: 0, j: 0 };
    b.iter(|| unsafe {
        h3ron_h3_sys::cellToLocalIj(
            black_box(anchor),
            black_box(index),
            0,
            &mut out,
        )
    })
}
