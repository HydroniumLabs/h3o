use criterion::{black_box, BenchmarkId, Criterion};
use h3o::Resolution;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getNumCells");

    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            &resolution,
            |b, &resolution| {
                let resolution =
                    Resolution::try_from(resolution).expect("resolution");
                b.iter(|| black_box(resolution).cell_count())
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            &resolution,
            |b, &resolution| {
                let mut out: i64 = 0;
                b.iter(|| unsafe {
                    h3ron_h3_sys::getNumCells(black_box(resolution), &mut out)
                })
            },
        );
    }

    group.finish();
}
