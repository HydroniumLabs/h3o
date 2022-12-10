use criterion::{black_box, Criterion};

pub fn bench(c: &mut Criterion) {
    const VALUE: f64 = 48.854501508844095;

    let mut group = c.benchmark_group("degsToRads");

    group.bench_function("h3o", |b| b.iter(|| black_box(VALUE).to_radians()));
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::degsToRads(black_box(VALUE)) })
    });

    group.finish();
}
