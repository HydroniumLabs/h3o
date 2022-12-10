use criterion::Criterion;
use h3o::BaseCell;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("res0CellCount");

    group.bench_function("h3o", |b| b.iter(BaseCell::count));
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::res0CellCount() })
    });

    group.finish();
}
