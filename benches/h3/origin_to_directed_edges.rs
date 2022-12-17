use criterion::{black_box, Bencher, Criterion};
use h3o::CellIndex;

const PENTAGON: u64 = 0x8f0800000000000;
const HEXAGON: u64 = 0x8f734e64992d6d8;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("originToDirectedEdges");

    group.bench_function("h3o/Hexagon", |b| bench_h3o(b, HEXAGON));
    group.bench_function("h3/Hexagon", |b| bench_h3(b, HEXAGON));

    group.bench_function("h3o/Pentagon", |b| bench_h3o(b, PENTAGON));
    group.bench_function("h3/Pentagon", |b| bench_h3(b, PENTAGON));
    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: u64) {
    let index = CellIndex::try_from(index).expect("cell index");
    b.iter(|| {
        for edge in black_box(index).edges() {
            black_box(edge);
        }
    });
}

fn bench_h3(b: &mut Bencher<'_>, index: u64) {
    let mut out = [0; 6];
    b.iter(|| unsafe {
        h3ron_h3_sys::originToDirectedEdges(black_box(index), out.as_mut_ptr());
        for edge in out {
            black_box(edge);
        }
    })
}
