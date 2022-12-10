use criterion::{black_box, BenchmarkId, Criterion};
use h3o::{CellIndex, Resolution};

pub fn bench(c: &mut Criterion) {
    const INDEX: u64 = 0x8073fffffffffff;
    let mut group = c.benchmark_group("cellToCenterChild");

    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            &resolution,
            |b, &resolution| {
                let resolution =
                    Resolution::try_from(resolution).expect("resolution");
                let index = CellIndex::try_from(INDEX).expect("cell index");

                b.iter(|| black_box(index).center_child(black_box(resolution)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            &resolution,
            |b, &resolution| {
                let mut out: u64 = 0;
                b.iter(|| unsafe {
                    h3ron_h3_sys::cellToCenterChild(
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
