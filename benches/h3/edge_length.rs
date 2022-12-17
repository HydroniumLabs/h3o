use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::{CellIndex, DirectedEdgeIndex};

pub fn bench_rads(c: &mut Criterion) {
    bench_edge_length(
        c,
        "edgeLengthRads",
        |b, &edge| b.iter(|| black_box(edge).length_rads()),
        |b, &edge| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::edgeLengthRads(
                    black_box(edge.into()),
                    &mut result,
                );
                result
            })
        },
    );
}

pub fn bench_km(c: &mut Criterion) {
    bench_edge_length(
        c,
        "edgeLengthKm",
        |b, &edge| b.iter(|| black_box(edge).length_km()),
        |b, &edge| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::edgeLengthKm(black_box(edge.into()), &mut result);
                result
            })
        },
    );
}

pub fn bench_m(c: &mut Criterion) {
    bench_edge_length(
        c,
        "edgeLengthM",
        |b, &edge| b.iter(|| black_box(edge).length_m()),
        |b, &edge| {
            let mut result: f64 = 0.;
            b.iter(|| unsafe {
                h3ron_h3_sys::edgeLengthM(black_box(edge.into()), &mut result);
                result
            })
        },
    );
}

// -----------------------------------------------------------------------------

fn bench_edge_length<F, G>(
    c: &mut Criterion,
    name: &'static str,
    bench_h3o: F,
    bench_h3: G,
) where
    F: FnMut(&mut Bencher<'_>, &DirectedEdgeIndex) + Copy,
    G: FnMut(&mut Bencher<'_>, &DirectedEdgeIndex) + Copy,
{
    let mut group = c.benchmark_group(name);

    for (resolution, &index) in HEXAGONS.iter().enumerate() {
        let cell = CellIndex::try_from(index).expect("cell index");
        let edge = cell.edges().next().expect("edge index");

        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            &edge,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            &edge,
            bench_h3,
        );
    }

    for (resolution, &index) in PENTAGONS.iter().enumerate() {
        let cell = CellIndex::try_from(index).expect("cell index");
        let edge = cell.edges().next().expect("edge index");

        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            &edge,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            &edge,
            bench_h3,
        );
    }

    group.finish();
}
