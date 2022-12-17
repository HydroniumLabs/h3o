use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::{CellIndex, Vertex};

pub fn bench(c: &mut Criterion) {
    const HEXAGON: u64 = 0x084e_c69b_ffff_ffff;
    const PENTAGON: u64 = 0x0841_f265_ffff_ffff;

    let mut group = c.benchmark_group("cellToVertex");

    for vertex_number in 0..6u8 {
        bench_h3o(&mut group, "h3o/Hexagon", HEXAGON, vertex_number);
        bench_h3(&mut group, "h3/Hexagon", HEXAGON, vertex_number);

        if vertex_number < 5 {
            bench_h3o(&mut group, "h3o/Pentagon", PENTAGON, vertex_number);
            bench_h3(&mut group, "h3/Pentagon", PENTAGON, vertex_number);
        }
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    vertex_number: u8,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, vertex_number),
        &index,
        |b, &index| {
            let index = CellIndex::try_from(index).expect("cell index");
            let vertex = Vertex::try_from(vertex_number).expect("vertex");
            b.iter(|| black_box(index).vertex(black_box(vertex)))
        },
    );
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    index: u64,
    vertex_number: u8,
) where
    T: Measurement,
{
    group.bench_with_input(
        BenchmarkId::new(name, vertex_number),
        &index,
        |b, &index| {
            let mut out: u64 = 0;
            b.iter(|| unsafe {
                h3ron_h3_sys::cellToVertex(
                    black_box(index),
                    black_box(vertex_number.into()),
                    &mut out,
                )
            })
        },
    );
}
