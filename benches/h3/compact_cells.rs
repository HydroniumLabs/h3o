use criterion::{black_box, Bencher, Criterion};
use h3o::{CellIndex, Direction, Resolution};

pub fn bench(c: &mut Criterion) {
    const RESOLUTION: Resolution = Resolution::Three;

    let mut group = c.benchmark_group("compactCells");

    let cells = CellIndex::base_cells()
        .flat_map(|index| index.children(RESOLUTION))
        .collect::<Vec<_>>();
    group.bench_function("h3o/FullCompaction", |b| bench_h3o(b, &cells));
    group.bench_function("h3/FullCompaction", |b| bench_h3(b, &cells));

    let sparse = cells
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(idx, cell)| (idx % 33 != 0).then_some(cell))
        .collect::<Vec<_>>();
    group.bench_function("h3o/PartialCompaction", |b| bench_h3o(b, &sparse));
    group.bench_function("h3/PartialCompaction", |b| bench_h3(b, &sparse));

    let uncompactable = cells
        .iter()
        .copied()
        .filter_map(|cell| {
            (cell.direction_at(RESOLUTION) != Some(Direction::IK))
                .then_some(cell)
        })
        .collect::<Vec<_>>();
    group.bench_function("h3o/NoCompaction", |b| bench_h3o(b, &uncompactable));
    group.bench_function("h3/NoCompaction", |b| bench_h3(b, &uncompactable));

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, indexes: &[CellIndex]) {
    b.iter(|| {
        CellIndex::compact(black_box(indexes.iter().copied()))
            .expect("compacted set")
            .for_each(drop)
    })
}

fn bench_h3(b: &mut Bencher<'_>, indexes: &[CellIndex]) {
    let indexes = indexes.iter().copied().map(u64::from).collect::<Vec<_>>();
    let mut out = vec![0; indexes.len()];
    b.iter(|| unsafe {
        h3ron_h3_sys::compactCells(
            black_box(indexes.as_ptr()),
            out.as_mut_ptr(),
            indexes.len() as i64,
        )
    })
}
