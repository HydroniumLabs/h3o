use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::VertexIndex;

const HEXAGONS: [u64; 6] = [
    0x24b734e649928fff,
    0x23b734e649928fff,
    0x22b734e649929fff,
    0x23b734e649929fff,
    0x24b734e649929fff,
    0x25b734e649929fff,
];

const PENTAGONS: [u64; 5] = [
    0x20b0800000000fff,
    0x21b0800000000fff,
    0x22b0800000000fff,
    0x23b0800000000fff,
    0x24b0800000000fff,
];

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("isValidVertex");

    for (vertex, index) in HEXAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", vertex),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", vertex),
            index,
            bench_h3,
        );
    }

    for (vertex, index) in PENTAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", vertex),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", vertex),
            index,
            bench_h3,
        );
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: &u64) {
    b.iter(|| VertexIndex::try_from(black_box(*index)))
}

fn bench_h3(b: &mut Bencher<'_>, index: &u64) {
    b.iter(|| unsafe { h3ron_h3_sys::isValidVertex(black_box(*index)) })
}
