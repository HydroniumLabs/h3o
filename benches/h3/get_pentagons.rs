use criterion::{black_box, BenchmarkId, Criterion};
use h3o::Resolution;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getPentagons");

    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            &resolution,
            |b, &resolution| {
                let resolution =
                    Resolution::try_from(resolution).expect("resolution");
                b.iter(|| {
                    for pentagon in black_box(resolution).pentagons() {
                        black_box(pentagon);
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            &resolution,
            |b, &resolution| {
                let mut out = [0; 12];
                b.iter(|| unsafe {
                    h3ron_h3_sys::getPentagons(
                        black_box(resolution),
                        out.as_mut_ptr(),
                    );
                    for pentagon in out {
                        black_box(pentagon);
                    }
                })
            },
        );
    }

    group.finish();
}
