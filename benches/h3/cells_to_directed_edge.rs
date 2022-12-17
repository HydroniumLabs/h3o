use criterion::{black_box, Criterion};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    const ORIGIN: u64 = 0x0891_fb46_622f_ffff;
    const DESTINATION: u64 = 0x0891_fb46_622b_ffff;

    let mut group = c.benchmark_group("cellsToDirectedEdge");

    group.bench_function("h3o", |b| {
        let origin = CellIndex::try_from(ORIGIN).expect("cell index");
        let destination = CellIndex::try_from(DESTINATION).expect("cell index");
        b.iter(|| black_box(origin).edge(black_box(destination)))
    });
    group.bench_function("h3", |b| {
        let mut out: u64 = 0;
        b.iter(|| unsafe {
            h3ron_h3_sys::cellsToDirectedEdge(
                black_box(ORIGIN),
                black_box(DESTINATION),
                &mut out,
            )
        })
    });

    group.finish();
}
