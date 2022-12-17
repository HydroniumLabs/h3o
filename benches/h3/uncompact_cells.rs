use criterion::{black_box, Criterion};
use h3o::{CellIndex, Resolution};

const RESOLUTION: Resolution = Resolution::Seven;

pub fn bench(c: &mut Criterion) {
    let compacted = vec![
        CellIndex::try_from(0x802bfffffffffff).unwrap(), // hexagon  res 0.
        CellIndex::try_from(0x820807fffffffff).unwrap(), // pentagon res 1.
        CellIndex::try_from(0x83734efffffffff).unwrap(), // hexagon  res 3.
    ];
    let size = CellIndex::uncompact_size(compacted.iter().copied(), RESOLUTION);

    let mut group = c.benchmark_group("uncompactCells");

    group.bench_function("h3o", |b| {
        b.iter(|| {
            let iter = compacted.iter().copied();
            CellIndex::uncompact(black_box(iter), black_box(RESOLUTION))
                .for_each(drop)
        })
    });
    group.bench_function("h3", |b| {
        let cells =
            compacted.iter().copied().map(u64::from).collect::<Vec<_>>();
        let mut out = vec![0; size as usize];
        b.iter(|| unsafe {
            h3ron_h3_sys::uncompactCells(
                black_box(cells.as_ptr()),
                black_box(cells.len() as i64),
                out.as_mut_ptr(),
                black_box(size as i64),
                black_box(u8::from(RESOLUTION).into()),
            )
        })
    });

    group.finish();
}
