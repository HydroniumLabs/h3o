use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::DirectedEdgeIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("getDirectedEdgeDestination");

    for (resolution, (hexagon, pentagon)) in [
        (0x150f_3fff_ffff_ffff, 0x1401_ffff_ffff_ffff),
        (0x111f_27ff_ffff_ffff, 0x161d_b3ff_ffff_ffff),
        (0x142f_367f_ffff_ffff, 0x162d_b37f_ffff_ffff),
        (0x143f_365f_ffff_ffff, 0x1531_f65f_ffff_ffff),
        (0x114e_c69b_ffff_ffff, 0x1441_f265_ffff_ffff),
    ]
    .iter()
    .enumerate()
    {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            hexagon,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            hexagon,
            bench_h3,
        );

        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            pentagon,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            pentagon,
            bench_h3,
        );
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: &u64) {
    let index = DirectedEdgeIndex::try_from(*index).expect("edge index");
    b.iter(|| black_box(index).destination())
}

fn bench_h3(b: &mut Bencher<'_>, index: &u64) {
    let mut out: u64 = 0;
    b.iter(|| unsafe {
        h3ron_h3_sys::getDirectedEdgeDestination(black_box(*index), &mut out)
    })
}
