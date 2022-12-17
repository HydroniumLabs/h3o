use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::CellIndex;
use std::os::raw::c_int;

const HEXAGON: u64 = 0x8b1fb46622defff;
const PENTAGON: u64 = 0x8b0800000000fff;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("gridDiskDistances");

    for k in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 100] {
        bench_h3o(&mut group, "h3o/Hexagon", HEXAGON, k);
        bench_h3(&mut group, "h3/Hexagon", HEXAGON, k);

        bench_h3o(&mut group, "h3o/Pentagon", PENTAGON, k);
        bench_h3(&mut group, "h3/Pentagon", PENTAGON, k);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    k: u32,
) where
    T: Measurement,
{
    let index = CellIndex::try_from(index).expect("hex index");
    group.bench_with_input(BenchmarkId::new(name, k), &index, |b, &index| {
        b.iter_with_large_drop(|| {
            black_box(index).grid_disk_distances::<Vec<_>>(black_box(k))
        })
    });
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    k: u32,
) where
    T: Measurement,
{
    let size =
        usize::try_from(h3o::max_grid_disk_size(k)).expect("grid too large");

    group.bench_with_input(
        BenchmarkId::new(name, k),
        &index,
        |b, &pentagon| {
            b.iter_with_large_drop(|| unsafe {
                let mut cells = vec![0u64; size];
                let mut distances: Vec<c_int> = vec![0; size];

                h3ron_h3_sys::gridDiskDistances(
                    black_box(pentagon),
                    black_box(k as c_int),
                    cells.as_mut_ptr(),
                    distances.as_mut_ptr(),
                )
            })
        },
    );
}
