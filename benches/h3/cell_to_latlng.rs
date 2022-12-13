use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::{CellIndex, LatLng};

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellToLatLng");

    for (resolution, index) in HEXAGONS.iter().copied().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            &index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            &index,
            bench_h3,
        );
    }

    for (resolution, index) in PENTAGONS.iter().copied().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            &index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            &index,
            bench_h3,
        );
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: &u64) {
    let index = CellIndex::try_from(*index).expect("cell index");
    b.iter(|| LatLng::from(black_box(index)))
}

fn bench_h3(b: &mut Bencher<'_>, index: &u64) {
    let mut ll = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    b.iter(|| unsafe { h3ron_h3_sys::cellToLatLng(black_box(*index), &mut ll) })
}
