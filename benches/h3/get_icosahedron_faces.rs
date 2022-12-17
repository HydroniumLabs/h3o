use super::constants::{HEXAGONS, PENTAGONS};
use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getIcosahedronFaces");

    for (resolution, index) in HEXAGONS.iter().copied().enumerate() {
        bench_h3o(&mut group, "h3o/Hexagon", index, resolution);
        bench_h3(&mut group, "h3/Hexagon", index, resolution);
    }

    for (resolution, index) in PENTAGONS.iter().copied().enumerate() {
        bench_h3o(&mut group, "h3o/Pentagon", index, resolution);
        bench_h3(&mut group, "h3/Pentagon", index, resolution);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    resolution: usize,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &index,
        |b, &index| {
            let index = CellIndex::try_from(index).expect("cell index");
            b.iter(|| black_box(index).icosahedron_faces())
        },
    );
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    resolution: usize,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &index,
        |b, &index| {
            let mut result = vec![-1; 5];
            b.iter(|| unsafe {
                h3ron_h3_sys::getIcosahedronFaces(
                    black_box(index),
                    result.as_mut_ptr(),
                );
            })
        },
    );
}
