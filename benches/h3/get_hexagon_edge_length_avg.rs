use criterion::{black_box, Criterion};
use h3o::Resolution;

pub fn bench_km(c: &mut Criterion) {
    let mut group = c.benchmark_group("getHexagonEdgeLengthAvgKm");

    group.bench_function("h3o", |b| {
        b.iter(|| black_box(Resolution::Three).edge_length_km())
    });
    group.bench_function("h3", |b| {
        let mut out: f64 = 0.;
        b.iter(|| unsafe {
            h3ron_h3_sys::getHexagonEdgeLengthAvgKm(black_box(3), &mut out)
        })
    });

    group.finish();
}

pub fn bench_m(c: &mut Criterion) {
    let mut group = c.benchmark_group("getHexagonEdgeLengthAvgM");

    group.bench_function("h3o", |b| {
        b.iter(|| black_box(Resolution::Three).edge_length_m())
    });
    group.bench_function("h3", |b| {
        let mut out: f64 = 0.;
        b.iter(|| unsafe {
            h3ron_h3_sys::getHexagonEdgeLengthAvgM(black_box(3), &mut out)
        })
    });

    group.finish();
}
