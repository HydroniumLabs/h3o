use criterion::{black_box, Bencher, Criterion};
use h3o::CellIndex;
use std::os::raw::c_int;

const PENTAGON: u64 = 0x8f0800000000000;
const HEXAGON: u64 = 0x8f734e64992d6d8;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("maxFaceCount");

    group.bench_function("h3o/Hexagon", |b| bench_h3o(b, HEXAGON));
    group.bench_function("h3/Hexagon", |b| bench_h3(b, HEXAGON));

    group.bench_function("h3o/Pentagon", |b| bench_h3o(b, PENTAGON));
    group.bench_function("h3/Pentagon", |b| bench_h3(b, PENTAGON));

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: u64) {
    let index = CellIndex::try_from(index).expect("cell index");
    b.iter(|| black_box(index).max_face_count())
}

fn bench_h3(b: &mut Bencher<'_>, index: u64) {
    b.iter(|| {
        let mut out: c_int = 0;
        unsafe { h3ron_h3_sys::maxFaceCount(black_box(index), &mut out) }
    })
}
