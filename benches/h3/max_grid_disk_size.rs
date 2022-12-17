use criterion::{black_box, Criterion};
use std::os::raw::c_int;

const K: u32 = 42;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("maxGridDiskSize");

    group.bench_function("h3o", |b| {
        b.iter(|| h3o::max_grid_disk_size(black_box(K)))
    });
    group.bench_function("h3", |b| {
        b.iter(|| {
            let mut out: i64 = 0;
            unsafe {
                h3ron_h3_sys::maxGridDiskSize(black_box(K as c_int), &mut out)
            }
        })
    });

    group.finish();
}
