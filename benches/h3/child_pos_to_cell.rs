use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::{CellIndex, Resolution};

pub fn bench(c: &mut Criterion) {
    const HEXAGON: u64 = 0x8073fffffffffff;
    const PENTAGON: u64 = 0x8009fffffffffff;

    let mut group = c.benchmark_group("childPosToCell");

    for (idx, &position) in [
        4272695453621,
        203357016527,
        9578995713,
        9578995713,
        1669688741,
        257312496,
        15190854,
        3661252,
        367080,
        14133,
        14133,
        2128,
        70,
        21,
        0,
        0u64,
    ]
    .iter()
    .rev()
    .enumerate()
    {
        let resolution = idx as u8;
        bench_h3o(&mut group, "h3o/Hexagon", HEXAGON, position, resolution);
        bench_h3(&mut group, "h3/Hexagon", HEXAGON, position, resolution);
    }

    for (idx, &position) in [
        121844509921,
        121844509921,
        41103667915,
        13421093513,
        1557133055,
        144756810,
        23695989,
        636785,
        636785,
        48540,
        14926,
        520,
        177,
        30,
        2,
        0u64,
    ]
    .iter()
    .rev()
    .enumerate()
    {
        let resolution = idx as u8;
        bench_h3o(&mut group, "h3o/Pentagon", PENTAGON, position, resolution);
        bench_h3(&mut group, "h3/Pentagon", PENTAGON, position, resolution);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    position: u64,
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
            b.iter(|| {
                black_box(index)
                    .child_at(black_box(position), black_box(resolution))
            })
        },
    );
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    position: u64,
    resolution: u8,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, resolution),
        &resolution,
        |b, &resolution| {
            b.iter(|| {
                let mut out: u64 = 0;
                unsafe {
                    h3ron_h3_sys::childPosToCell(
                        black_box(position as i64),
                        black_box(index),
                        black_box(resolution.into()),
                        &mut out,
                    )
                }
            })
        },
    );
}
