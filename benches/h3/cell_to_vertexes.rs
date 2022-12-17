use criterion::{black_box, Bencher, Criterion};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    const PENTAGON: u64 = 0x08f0_8000_0000_0000;
    const HEXAGON: u64 = 0x08f7_34e6_4992_d6d8;

    let mut group = c.benchmark_group("cellToVertexes");

    group.bench_function("h3o/Hexagon", |b| bench_h3o(b, HEXAGON));
    group.bench_function("h3/Hexagon", |b| bench_h3(b, HEXAGON));

    group.bench_function("h3o/Pentagon", |b| bench_h3o(b, PENTAGON));
    group.bench_function("h3/Pentagon", |b| bench_h3(b, PENTAGON));

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: u64) {
    let index = CellIndex::try_from(index).expect("cell index");
    b.iter(|| black_box(index).vertexes().for_each(drop))
}

fn bench_h3(b: &mut Bencher<'_>, index: u64) {
    let mut out = [0; 6];
    b.iter(|| unsafe {
        h3ron_h3_sys::cellToVertexes(black_box(index), out.as_mut_ptr())
    })
}
