use criterion::{black_box, BenchmarkId, Criterion};
use h3o::{CellIndex, Resolution};

pub fn bench(c: &mut Criterion) {
    const INDEX: u64 = 0x8f734e64992d6d8;
    let mut group = c.benchmark_group("cellToParent");

    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            &resolution,
            |b, &resolution| {
                let resolution =
                    Resolution::try_from(resolution).expect("resolution");
                let index = CellIndex::try_from(INDEX).expect("cell index");

                b.iter(|| black_box(index).parent(black_box(resolution)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            &resolution,
            |b, &resolution| {
                let mut out: u64 = 0;
                b.iter(|| unsafe {
                    h3ron_h3_sys::cellToParent(
                        black_box(INDEX),
                        black_box(resolution),
                        &mut out,
                    )
                })
            },
        );
    }

    group.finish();
}
