use super::constants::PENTAGONS;
use criterion::{black_box, BenchmarkId, Criterion};
use h3o::CellIndex;

const HEXAGON: u64 = 0x8f734e64992d6d8;

pub fn bench_hexagons(c: &mut Criterion) {
    let mut group = c.benchmark_group("isPentagon/Hexagon");

    group.bench_function("h3o", |b| {
        let index = CellIndex::try_from(HEXAGON).expect("cell index");
        b.iter(|| black_box(index).is_pentagon())
    });
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::isPentagon(black_box(HEXAGON)) })
    });

    group.finish();
}

pub fn bench_pentagons(c: &mut Criterion) {
    let mut group = c.benchmark_group("isPentagon/Pentagon");

    for (resolution, index) in PENTAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            index,
            |b, &index| {
                let index = CellIndex::try_from(index).expect("cell index");
                b.iter(|| black_box(index).is_pentagon())
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", resolution),
            index,
            |b, &index| {
                b.iter(|| unsafe { h3ron_h3_sys::isPentagon(black_box(index)) })
            },
        );
    }
    group.finish();
}
