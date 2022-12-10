use criterion::Criterion;
use h3o::Resolution;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("pentagonCount");

    group.bench_function("h3o", |b| b.iter(Resolution::pentagon_count));
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::pentagonCount() })
    });

    group.finish();
}
