use criterion::{black_box, BenchmarkId, Criterion};
use h3o::CellIndex;

pub fn bench_succ(c: &mut Criterion) {
    let hexagons: [u64; 16] = [
        0o40067777777777777777,
        0o40266677777777777777,
        0o40466667777777777777,
        0o40666666777777777777,
        0o41066666677777777777,
        0o41266666667777777777,
        0o41466666666777777777,
        0o41666666666677777777,
        0o42066666666667777777,
        0o42266666666666777777,
        0o42466666666666677777,
        0o42666666666666667777,
        0o43066666666666666777,
        0o43266666666666666677,
        0o43466666666666666667,
        0o43666666666666666666,
    ];
    let mut group = c.benchmark_group("NextCellIndex");

    for (resolution, index) in hexagons.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            index,
            |b, &index| {
                let index = CellIndex::try_from(index).expect("valid index");
                b.iter(|| black_box(index).succ())
            },
        );
    }

    group.finish();
}

pub fn bench_pred(c: &mut Criterion) {
    let hexagons: [u64; 16] = [
        0o40007777777777777777,
        0o40200077777777777777,
        0o40400007777777777777,
        0o40600000777777777777,
        0o41000000077777777777,
        0o41200000007777777777,
        0o41400000000777777777,
        0o41600000000077777777,
        0o42000000000007777777,
        0o42200000000000777777,
        0o42400000000000077777,
        0o42600000000000007777,
        0o43000000000000000777,
        0o43200000000000000077,
        0o43400000000000000007,
        0o43600000000000000000,
    ];
    let mut group = c.benchmark_group("PreviousCellIndex");

    for (resolution, index) in hexagons.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o", resolution),
            index,
            |b, &index| {
                let index = CellIndex::try_from(index).expect("valid index");
                b.iter(|| black_box(index).pred())
            },
        );
    }

    group.finish();
}
