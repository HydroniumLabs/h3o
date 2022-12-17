use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("directedEdgeToBoundary");

    for (resolution, index) in HEXAGONS.iter().enumerate() {
        let index = CellIndex::try_from(*index).expect("hexagon index");

        bench_h3o(&mut group, "h3o/Hexagon", index, resolution);
        bench_h3(&mut group, "h3/Hexagon", index, resolution);
    }

    for (resolution, index) in PENTAGONS.iter().enumerate() {
        let index = CellIndex::try_from(*index).expect("pentagon index");

        bench_h3o(&mut group, "h3o/Pentagon", index, resolution);
        bench_h3(&mut group, "h3/Pentagon", index, resolution);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

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
            b.iter(|| black_box(edge).boundary().len())
        },
    );
}

fn bench_h3<T>(
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
            let mut result = h3ron_h3_sys::CellBoundary {
                numVerts: 0,
                verts: [h3ron_h3_sys::LatLng { lat: 0., lng: 0. }; 10],
            };
            b.iter(|| unsafe {
                h3ron_h3_sys::directedEdgeToBoundary(
                    black_box(edge.into()),
                    &mut result,
                )
            })
        },
    );
}
