use criterion::{black_box, Criterion};
use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};
use std::ffi::CString;

const CELL_INDEX: &str = "08f734e64992d6d8";
const EDGE_INDEX: &str = "15f2834782b9c2ab";
const VERT_INDEX: &str = "23b734e649928fff";

pub fn bench_cell(c: &mut Criterion) {
    let mut group = c.benchmark_group("stringToH3");

    group.bench_function("h3o/Cell", |b| {
        let s = CELL_INDEX.to_owned();
        b.iter(|| black_box(&s).parse::<CellIndex>())
    });
    group.bench_function("h3/Cell", |b| {
        let ptr = CString::new(CELL_INDEX).expect("CString").into_raw();
        b.iter(|| {
            let mut out: h3ron_h3_sys::H3Index = 0;
            unsafe { h3ron_h3_sys::stringToH3(black_box(ptr), &mut out) }
        })
    });

    group.finish();
}

pub fn bench_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("stringToH3");

    group.bench_function("h3o/Edge", |b| {
        let s = EDGE_INDEX.to_owned();
        b.iter(|| black_box(&s).parse::<DirectedEdgeIndex>())
    });
    group.bench_function("h3/Edge", |b| {
        let ptr = CString::new(EDGE_INDEX).expect("CString").into_raw();
        b.iter(|| {
            let mut out: h3ron_h3_sys::H3Index = 0;
            unsafe { h3ron_h3_sys::stringToH3(black_box(ptr), &mut out) }
        })
    });

    group.finish();
}

pub fn bench_vertex(c: &mut Criterion) {
    let mut group = c.benchmark_group("stringToH3");

    group.bench_function("h3o/Vertex", |b| {
        let s = VERT_INDEX.to_owned();
        b.iter(|| black_box(&s).parse::<VertexIndex>())
    });
    group.bench_function("h3/Vertex", |b| {
        let ptr = CString::new(VERT_INDEX).expect("CString").into_raw();
        b.iter(|| {
            let mut out: h3ron_h3_sys::H3Index = 0;
            unsafe { h3ron_h3_sys::stringToH3(black_box(ptr), &mut out) }
        })
    });

    group.finish();
}
