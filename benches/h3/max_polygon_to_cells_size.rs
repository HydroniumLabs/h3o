use super::utils::load_polygon;
use criterion::{black_box, Criterion};
use h3o::{
    geom::{PolyfillConfig, ToCells},
    Resolution,
};
use std::os::raw::c_int;

const RESOLUTION: Resolution = Resolution::Nine;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("maxPolygonToCellsSize");
    let polygon = load_polygon("Paris");
    let config = PolyfillConfig::new(RESOLUTION);

    group.bench_function("h3o", |b| {
        b.iter(|| black_box(&polygon).max_cells_count(black_box(config)))
    });
    group.bench_function("h3", |b| {
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
        let mut out = 0;
        b.iter(|| unsafe {
            h3ron_h3_sys::maxPolygonToCellsSize(
                black_box(&polygon),
                black_box(u8::from(RESOLUTION).into()),
                0,
                &mut out,
            );
        })
    });

    group.finish();
}
