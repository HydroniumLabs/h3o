use criterion::{black_box, Criterion};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getRes0Cells");

    group.bench_function("h3o", |b| {
        b.iter(|| {
            for base_cell in CellIndex::base_cells() {
                let _ = black_box(base_cell);
            }
        })
    });
    group.bench_function("h3", |b| {
        let mut out = [0; 122];
        b.iter(|| unsafe {
            h3ron_h3_sys::getRes0Cells(black_box(out.as_mut_ptr()));
            for base_cell in out {
                let _ = black_box(base_cell);
            }
        })
    });

    group.finish();
}
