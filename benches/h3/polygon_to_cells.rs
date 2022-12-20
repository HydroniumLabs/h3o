use super::utils::load_polygon;
use criterion::{black_box, BatchSize, Bencher, BenchmarkId, Criterion};
use h3o::{
    geom::{Polygon, ToCells},
    Resolution,
};
use std::os::raw::c_int;

pub fn bench_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("polygonToCells");
    let polygon = load_polygon("Paris");

    for res in 0..=12 {
        group.bench_with_input(
            BenchmarkId::new("h3o/Full", res),
            &res,
            |b, &res| bench_h3o(b, &polygon, res),
        );

        group.bench_with_input(
            BenchmarkId::new("h3/Full", res),
            &res,
            |b, &res| bench_h3(b, &polygon, res),
        );
    }

    group.finish();
}

pub fn bench_transmeridian(c: &mut Criterion) {
    let mut group = c.benchmark_group("polygonToCells");
    let polygon = load_polygon("Rabi");

    for res in 0..=13 {
        group.bench_with_input(
            BenchmarkId::new("h3o/Transmeridian", res),
            &res,
            |b, &res| bench_h3o(b, &polygon, res),
        );

        group.bench_with_input(
            BenchmarkId::new("h3/Transmeridian", res),
            &res,
            |b, &res| bench_h3(b, &polygon, res),
        );
    }

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, polygon: &Polygon, resolution: u8) {
    let resolution = Resolution::try_from(resolution).expect("resolution");

    b.iter(|| {
        black_box(polygon)
            .to_cells(black_box(resolution))
            .for_each(drop)
    });
}

fn bench_h3(b: &mut Bencher<'_>, polygon: &Polygon, resolution: u8) {
    let mut coords = geo::Polygon::from(polygon.clone())
        .exterior()
        .coords()
        .map(|coord| h3ron_h3_sys::LatLng {
            lat: coord.y,
            lng: coord.x,
        })
        .collect::<Vec<_>>();
    let geoloop = h3ron_h3_sys::GeoLoop {
        numVerts: coords.len() as c_int,
        verts: coords.as_mut_ptr(),
    };
    let polygon = h3ron_h3_sys::GeoPolygon {
        geoloop,
        numHoles: 0,
        holes: std::ptr::null_mut(),
    };
    let mut size = 0;
    unsafe {
        h3ron_h3_sys::maxPolygonToCellsSize(
            black_box(&polygon),
            black_box(resolution.into()),
            0,
            &mut size,
        );
    }
    b.iter_batched_ref(
        || vec![0; size as usize],
        |out| unsafe {
            h3ron_h3_sys::polygonToCells(
                black_box(&polygon),
                black_box(resolution.into()),
                0,
                out.as_mut_ptr(),
            )
        },
        BatchSize::SmallInput,
    )
}
