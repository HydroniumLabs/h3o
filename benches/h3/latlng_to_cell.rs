use criterion::{Bencher, BenchmarkId, Criterion};
use h3o::{LatLng, LatLngIndexer, Resolution};
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("latLngToCell");

    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("hex");
    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            &resolution,
            |b, &resolution| bench_h3o(b, ll, resolution),
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            &resolution,
            |b, &resolution| bench_h3(b, ll, resolution),
        );
    }

    let ll = LatLng::new(64.70000012793489, 10.53619907546772).expect("pent");
    for resolution in 0..=15 {
        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            &resolution,
            |b, &resolution| bench_h3o(b, ll, resolution),
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            &resolution,
            |b, &resolution| bench_h3(b, ll, resolution),
        );
    }

    group.finish();

    // Several resolutions per point: repeated `to_cell` vs a fresh indexer,
    // which only shares work between resolutions.
    let mut group = c.benchmark_group("latLngToCells");
    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("hex");
    for resolutions in [&[9][..], &[6, 8, 10], &[3, 5, 7, 9, 11, 13, 15]] {
        let resolutions = resolutions
            .iter()
            .map(|&res| Resolution::try_from(res).expect("resolution"))
            .collect::<Vec<_>>();
        let id = resolutions.len();
        group.bench_with_input(
            BenchmarkId::new("h3o/to_cell", id),
            &resolutions,
            |b, resolutions| {
                b.iter(|| {
                    let ll = black_box(ll);
                    for &res in resolutions {
                        black_box(ll.to_cell(res));
                    }
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3o/LatLngIndexer", id),
            &resolutions,
            |b, resolutions| {
                b.iter(|| {
                    let mut indexer = LatLngIndexer::new();
                    for cell in indexer
                        .to_cells(black_box(ll), resolutions.iter().copied())
                    {
                        black_box(cell);
                    }
                });
            },
        );
    }
    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, ll: LatLng, resolution: u8) {
    let resolution = Resolution::try_from(resolution).expect("resolution");
    b.iter(|| black_box(ll).to_cell(black_box(resolution)))
}

fn bench_h3(b: &mut Bencher<'_>, ll: LatLng, resolution: u8) {
    let mut out: u64 = 0;
    let ll = h3ron_h3_sys::LatLng {
        lat: ll.lat_radians(),
        lng: ll.lng_radians(),
    };
    b.iter(|| unsafe {
        h3ron_h3_sys::latLngToCell(
            black_box(&ll),
            black_box(resolution.into()),
            &mut out,
        )
    })
}

pub fn bench_scanline(c: &mut Criterion) {
    // One 30 m raster scanline of 1000 points, indexed with one indexer.
    let row = (0..1000)
        .map(|col| {
            LatLng::new(39.7, -105.0 + f64::from(col) * 3.5e-4).expect("ll")
        })
        .collect::<Vec<_>>();
    let mut group = c.benchmark_group("latLngScanline");
    for resolutions in [&[9][..], &[6, 8, 10]] {
        let resolutions = resolutions
            .iter()
            .map(|&res| Resolution::try_from(res).expect("resolution"))
            .collect::<Vec<_>>();
        let id = format!("{resolutions:?}");
        group.bench_with_input(
            BenchmarkId::new("h3o/to_cell", &id),
            &resolutions,
            |b, resolutions| {
                b.iter(|| {
                    for &ll in &row {
                        for &res in resolutions {
                            black_box(black_box(ll).to_cell(res));
                        }
                    }
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3o/LatLngIndexer", &id),
            &resolutions,
            |b, resolutions| {
                b.iter(|| {
                    let mut indexer = LatLngIndexer::new();
                    for &ll in &row {
                        for cell in indexer.to_cells(
                            black_box(ll),
                            resolutions.iter().copied(),
                        ) {
                            black_box(cell);
                        }
                    }
                });
            },
        );
    }
    group.finish();
}
