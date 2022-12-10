use criterion::{black_box, Criterion};
use h3o::DirectedEdgeIndex;

const INPUT: u64 = 0x13f2834782b9c2ab;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getDirectedEdgeOrigin");

    group.bench_function("h3o", |b| {
        let index = DirectedEdgeIndex::try_from(INPUT).expect("edge index");
        b.iter(|| black_box(index).origin())
    });
    group.bench_function("h3", |b| {
        let mut out: u64 = 0;
        b.iter(|| unsafe {
            h3ron_h3_sys::getDirectedEdgeOrigin(black_box(INPUT), &mut out)
        })
    });

    group.finish();
}
