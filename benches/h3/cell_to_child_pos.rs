use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::{CellIndex, Resolution};

pub fn bench(c: &mut Criterion) {
    const HEXAGON: u64 = 0x8f73909728c5c58;
    const PENTAGON: u64 = 0x8f0812d5c1562e2;

    let mut group = c.benchmark_group("cellToChildPos");

    for resolution in 0..=15 {
        bench_h3o(&mut group, "h3o/Hexagon", HEXAGON, resolution);
        bench_h3(&mut group, "h3/Hexagon", HEXAGON, resolution);

        bench_h3o(&mut group, "h3o/Pentagon", PENTAGON, resolution);
        bench_h3(&mut group, "h3/Pentagon", PENTAGON, resolution);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    resolution: u8,
) where
    T: Measurement,
{
    let index = CellIndex::try_from(index).expect("cell index");
    let resolution = Resolution::try_from(resolution).expect("resolution");
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &resolution,
        |b, &resolution| {
            b.iter(|| black_box(index).child_position(black_box(resolution)))
        },
    );
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    resolution: u8,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &resolution,
        |b, &resolution| {
            b.iter(|| {
                let mut out: i64 = 0;
                unsafe {
                    h3ron_h3_sys::cellToChildPos(
                        black_box(index),
                        black_box(resolution.into()),
                        &mut out,
                    )
                }
            })
        },
    );
}
