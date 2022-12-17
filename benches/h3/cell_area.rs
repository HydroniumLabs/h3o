use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::CellIndex;

pub fn bench_rads2(c: &mut Criterion) {
    bench_cell_area(
        c,
        "cellAreaRads",
        |b, &index| {
            let index = CellIndex::try_from(index).expect("cell index");
            b.iter(|| black_box(index).area_rads2())
        },
        |b, &index| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::cellAreaRads2(black_box(index), &mut result);
            })
        },
    );
}

pub fn bench_km2(c: &mut Criterion) {
    bench_cell_area(
        c,
        "cellAreaKm2",
        |b, &index| {
            let index = CellIndex::try_from(index).expect("cell index");
            b.iter(|| black_box(index).area_km2())
        },
        |b, &index| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::cellAreaKm2(black_box(index), &mut result);
            })
        },
    );
}

pub fn bench_m2(c: &mut Criterion) {
    bench_cell_area(
        c,
        "cellAreaM2",
        |b, &index| {
            let index = CellIndex::try_from(index).expect("cell index");
            b.iter(|| black_box(index).area_m2())
        },
        |b, &index| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::cellAreaM2(black_box(index), &mut result);
            })
        },
    );
}

// -----------------------------------------------------------------------------

fn bench_cell_area<F, G>(
    c: &mut Criterion,
    name: &'static str,
    bench_h3o: F,
    bench_h3: G,
) where
    F: FnMut(&mut Bencher<'_>, &u64) + Copy,
    G: FnMut(&mut Bencher<'_>, &u64) + Copy,
{
    let mut group = c.benchmark_group(name);

    for (resolution, index) in HEXAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            index,
            bench_h3,
        );
    }

    for (resolution, index) in PENTAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            index,
            bench_h3,
        );
    }

    group.finish();
}
