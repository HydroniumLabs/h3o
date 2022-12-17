use criterion::{black_box, Criterion};
use h3o::DirectedEdgeIndex;

const INPUT: u64 = 0x13f2834782b9c2ab;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("directedEdgeToCells");

    group.bench_function("h3o", |b| {
        let index = DirectedEdgeIndex::try_from(INPUT).expect("edge index");
        b.iter(|| black_box(index).cells())
    });
    group.bench_function("h3", |b| {
        let mut out = [0; 2];
        b.iter(|| unsafe {
            h3ron_h3_sys::directedEdgeToCells(
                black_box(INPUT),
                out.as_mut_ptr(),
            )
        })
    });

    group.finish();
}
