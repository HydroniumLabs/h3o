use criterion::{black_box, Criterion};

const VALUE: f64 = 0.8377580409552;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("radsToDegs");

    group.bench_function("h3o", |b| b.iter(|| black_box(VALUE).to_degrees()));
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::radsToDegs(black_box(VALUE)) })
    });

    group.finish();
}
