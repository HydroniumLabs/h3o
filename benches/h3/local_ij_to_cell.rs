use super::constants::HEXAGONS;
use criterion::{BenchmarkId, Criterion};
use h3o::{CellIndex, CoordIJ, LocalIJ};
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("localIjToCell");

    for (resolution, index) in HEXAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            index,
            |b, &index| {
                let anchor = CellIndex::try_from(index).expect("anchor");
                let coord = LocalIJ::new(anchor, CoordIJ::new(-4, -3));
                b.iter(|| CellIndex::try_from(black_box(coord)))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            index,
            |b, &index| {
                let mut out: u64 = 0;
                let ij = h3ron_h3_sys::CoordIJ { i: -4, j: -3 };
                b.iter(|| unsafe {
                    h3ron_h3_sys::localIjToCell(index, &ij, 0, &mut out)
                })
            },
        );
    }

    group.finish();
}
