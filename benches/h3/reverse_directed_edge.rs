use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, measurement::Measurement,
};
use h3o::CellIndex;
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("reverseDirectedEdge");

    for (resolution, &index) in HEXAGONS.iter().enumerate() {
        let cell = CellIndex::try_from(index).expect("cell index");

        bench_h3o(&mut group, "h3o/Hexagon", cell, resolution);
    }

    for (resolution, &index) in PENTAGONS.iter().enumerate() {
        let cell = CellIndex::try_from(index).expect("cell index");

        bench_h3o(&mut group, "h3o/Pentagon", cell, resolution);
    }

    group.finish();
}

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: CellIndex,
    resolution: usize,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &index,
        |b, &index| {
            let edge = index.edges().next().expect("edge index");
            b.iter(|| black_box(edge).reverse())
        },
    );
}
