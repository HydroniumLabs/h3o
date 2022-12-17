use criterion::{black_box, BenchmarkId, Criterion};
use h3o::CellIndex;
use std::os::raw::c_int;

const HEXAGON: u64 = 0x08b1_fb46_622d_efff;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("gridDiskDistancesUnsafe");

    for k in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 100] {
        let size = usize::try_from(h3o::max_grid_disk_size(k))
            .expect("grid too large");
        let mut cells = vec![0; size];
        let mut distances: Vec<c_int> = vec![0; size];

        group.bench_with_input(
            BenchmarkId::new("h3o", k),
            &HEXAGON,
            |b, &hexagon| {
                let index = CellIndex::try_from(hexagon).expect("hex index");
                b.iter(|| {
                    black_box(index)
                        .grid_disk_distances_fast(black_box(k))
                        .for_each(drop)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", k),
            &HEXAGON,
            |b, &hexagon| {
                b.iter(|| unsafe {
                    h3ron_h3_sys::gridDiskDistancesUnsafe(
                        black_box(hexagon),
                        black_box(k as c_int),
                        cells.as_mut_ptr(),
                        distances.as_mut_ptr(),
                    )
                })
            },
        );
    }

    group.finish();
}
