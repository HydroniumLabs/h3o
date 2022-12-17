use criterion::{black_box, Criterion};
use h3o::{CellIndex, Resolution};

const RESOLUTION: Resolution = Resolution::Seven;

pub fn bench(c: &mut Criterion) {
    let compacted = vec![
        CellIndex::try_from(0x802bfffffffffff).unwrap(), // hexagon  res 0.
        CellIndex::try_from(0x820807fffffffff).unwrap(), // pentagon res 1.
        CellIndex::try_from(0x83734efffffffff).unwrap(), // hexagon  res 3.
    ];

    let mut group = c.benchmark_group("uncompactCellsSize");

    group.bench_function("h3o", |b| {
        b.iter(|| {
            let iter = compacted.iter().copied();
            CellIndex::uncompact_size(black_box(iter), black_box(RESOLUTION))
        })
    });
    group.bench_function("h3", |b| {
        let cells =
            compacted.iter().copied().map(u64::from).collect::<Vec<_>>();
        let mut out: i64 = 0;
        b.iter(|| unsafe {
            h3ron_h3_sys::uncompactCellsSize(
                black_box(cells.as_ptr()),
                black_box(cells.len() as i64),
                black_box(u8::from(RESOLUTION).into()),
                &mut out,
            )
        })
    });

    group.finish();
}
