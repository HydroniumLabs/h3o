use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::DirectedEdgeIndex;

const HEXAGONS: [u64; 16] = [
    0x12073fffffffffff,
    0x121737ffffffffff,
    0x122734ffffffffff,
    0x123734efffffffff,
    0x124734a9ffffffff,
    0x125734e67fffffff,
    0x126734e64fffffff,
    0x127734e64dffffff,
    0x128734e6499fffff,
    0x129734e64993ffff,
    0x12a734e64992ffff,
    0x12b734e649929fff,
    0x12c734e649929dff,
    0x12d734e64992d6ff,
    0x12e734e64992d6df,
    0x12f734e64992d6d8,
];

const PENTAGONS: [u64; 16] = [
    0x12009fffffffffff,
    0x121083ffffffffff,
    0x1220807fffffffff,
    0x1230800fffffffff,
    0x12408001ffffffff,
    0x125080003fffffff,
    0x1260800007ffffff,
    0x1270800000ffffff,
    0x12808000001fffff,
    0x129080000003ffff,
    0x12a0800000007fff,
    0x12b0800000000fff,
    0x12c08000000001ff,
    0x12d080000000003f,
    0x12e0800000000007,
    0x12f0800000000000,
];

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("isValidDirectedEdge");

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

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: &u64) {
    b.iter(|| DirectedEdgeIndex::try_from(black_box(*index)))
}

fn bench_h3(b: &mut Bencher<'_>, index: &u64) {
    b.iter(|| unsafe { h3ron_h3_sys::isValidDirectedEdge(black_box(*index)) })
}
