use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::{CellIndex, Resolution};

pub fn bench(c: &mut Criterion) {
    const HEXAGON: u64 = 0x83734efffffffff;
    const PENTAGON: u64 = 0x830800fffffffff;

    let hexagon = CellIndex::try_from(HEXAGON).expect("hexagon");
    let pentagon = CellIndex::try_from(PENTAGON).expect("pentagon");

    let mut group = c.benchmark_group("cellToChildren");

    // 7 generations of cell is ~1 millions cells.
    for i in 0..=7u8 {
        let child_res =
            Resolution::try_from(u8::from(hexagon.resolution()) + i)
                .expect("hex resolution");
        bench_h3o(&mut group, "h3o/Hexagon", i, hexagon, child_res);
        bench_h3(&mut group, "h3/Hexagon", i, hexagon, child_res);

        let child_res =
            Resolution::try_from(u8::from(pentagon.resolution()) + i)
                .expect("pent resolution");
        bench_h3o(&mut group, "h3o/Pentagon", i, pentagon, child_res);
        bench_h3(&mut group, "h3/Pentagon", i, pentagon, child_res);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    run: u8,
    index: CellIndex,
    resolution: Resolution,
) where
    T: Measurement,
{
    group.bench_with_input(BenchmarkId::new(name, run), &resolution, |b, _| {
        b.iter(|| {
            black_box(index)
                .children(black_box(resolution))
                .for_each(drop)
        })
    });
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    run: u8,
    index: CellIndex,
    resolution: Resolution,
) where
    T: Measurement,
{
    group.bench_with_input(BenchmarkId::new(name, run), &resolution, |b, _| {
        let size = index.children_count(resolution) as usize;
        let mut out = vec![0; size];
        b.iter(|| unsafe {
            h3ron_h3_sys::cellToChildren(
                black_box(index.into()),
                black_box(u8::from(resolution).into()),
                out.as_mut_ptr(),
            )
        })
    });
}
