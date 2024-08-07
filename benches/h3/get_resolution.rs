use criterion::{black_box, Criterion};
use h3o::CellIndex;

const INPUT: u64 = 0x8f734e64992d6d8;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getResolution");

    group.bench_function("h3o", |b| {
        let index = CellIndex::try_from(INPUT).expect("cell index");
        b.iter(|| black_box(index).resolution())
    });
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::getResolution(black_box(INPUT)) })
    });

    group.finish();
}
