use super::utils::load_cells;
use ahash::HashSet;
use criterion::{
    black_box, measurement::Measurement, BenchmarkGroup, BenchmarkId, Criterion,
};
use h3o::{geom::ToGeo, CellIndex};
use std::os::raw::c_int;

pub fn bench_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("h3SetToLinkedGeo");

    for resolution in 5..=11u32 {
        let cells = load_cells(resolution);

        bench_h3o(&mut group, "h3o/Full", cells.clone(), resolution);
        bench_h3(&mut group, "h3/Full", cells, resolution);
    }

    group.finish();
}

pub fn bench_holes(c: &mut Criterion) {
    let mut group = c.benchmark_group("h3SetToLinkedGeo");
    let cells = load_cells(8);

    for n in [0_u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
        let to_remove = cells
            .iter()
            .step_by(10)
            .take(n as usize)
            .flat_map(|origin| origin.grid_disk::<Vec<_>>(1).into_iter())
            .collect::<HashSet<_>>();
        let mut cells = cells.clone();
        cells.retain(|cell| !to_remove.contains(cell));

        bench_h3o(&mut group, "h3o/Holes", cells.clone(), n);
        bench_h3(&mut group, "h3/Holes", cells, n);
    }

    group.finish();
}

pub fn bench_rings(c: &mut Criterion) {
    let mut group = c.benchmark_group("h3SetToLinkedGeo");
    let hexagon = 0x08b1_fb46_622d_efff;

    for k in 0..=5 {
        let index = CellIndex::try_from(hexagon).expect("hex index");
        let mut cells = Vec::new();
        for i in 0..=k {
            cells.extend(
                index
                    .grid_ring_fast(i * 2)
                    .map(|res| res.expect("cell index")),
            );
        }
        cells.sort_unstable();
        cells.dedup();

        bench_h3o(&mut group, "h3o/Rings", cells.clone(), k);
        bench_h3(&mut group, "h3/Rings", cells, k);
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    indexes: Vec<CellIndex>,
    k: u32,
) where
    T: Measurement,
{
    group.bench_with_input(BenchmarkId::new(name, k), &k, |b, _k| {
        b.iter(|| black_box(indexes.iter().copied().to_geom(false)))
    });
}

fn bench_h3<T>(
    group: &mut BenchmarkGroup<T>,
    name: &'static str,
    indexes: Vec<CellIndex>,
    k: u32,
) where
    T: Measurement,
{
    let indexes = indexes.into_iter().map(Into::into).collect::<Vec<_>>();

    group.bench_with_input(BenchmarkId::new(name, k), &k, |b, _k| {
        b.iter(|| unsafe {
            let mut out = h3ron_h3_sys::LinkedGeoPolygon {
                first: std::ptr::null_mut(),
                last: std::ptr::null_mut(),
                next: std::ptr::null_mut(),
            };
            h3ron_h3_sys::cellsToLinkedMultiPolygon(
                black_box(indexes.as_ptr()),
                black_box(indexes.len() as c_int),
                &mut out,
            );
            h3ron_h3_sys::destroyLinkedMultiPolygon(&mut out);
        })
    });
}
