use criterion::{black_box, Criterion};
use h3o::Resolution;

pub fn bench_km2(c: &mut Criterion) {
    let mut group = c.benchmark_group("getHexagonAreaAvgKm2");

    group.bench_function("h3o", |b| {
        b.iter(|| black_box(Resolution::Three).area_km2())
    });
    group.bench_function("h3", |b| {
        let mut out: f64 = 0.;
        b.iter(|| unsafe {
            h3ron_h3_sys::getHexagonAreaAvgKm2(black_box(3), &mut out)
        })
    });

    group.finish();
}

pub fn bench_m2(c: &mut Criterion) {
    let mut group = c.benchmark_group("getHexagonAreaAvgM2");

    group.bench_function("h3o", |b| {
        b.iter(|| black_box(Resolution::Three).area_m2())
    });
    group.bench_function("h3", |b| {
        let mut out: f64 = 0.;
        b.iter(|| unsafe {
            h3ron_h3_sys::getHexagonAreaAvgM2(black_box(3), &mut out)
        })
    });

    group.finish();
}
