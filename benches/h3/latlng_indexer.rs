use criterion::{BenchmarkId, Criterion};
use h3o::{LatLng, LatLngIndexer, Resolution};
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("LatLngIndexer");

    let ll = LatLng::new(48.85458622023985, 2.373012457671282).expect("hex");
    let coords = build_grid();
    for max_res in 0..=15 {
        let resolutions = Resolution::range(
            Resolution::Zero,
            Resolution::try_from(max_res).unwrap(),
        )
        .collect::<Vec<_>>();

        // Indexing a single location at multiple resolutions.
        group.bench_with_input(
            BenchmarkId::new("LatLng/MultiResolution", max_res),
            &max_res,
            |b, _| {
                b.iter(|| {
                    resolutions
                        .iter()
                        .copied()
                        .map(|res| black_box(ll).to_cell(black_box(res)))
                        .for_each(drop)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Indexed/MultiResolution", max_res),
            &max_res,
            |b, _| {
                b.iter(|| {
                    let mut indexer = LatLngIndexer::default();
                    indexer
                        .to_cells(black_box(ll), resolutions.iter().copied())
                        .for_each(drop)
                })
            },
        );

        // Scanline at a single resolution.
        let current_res = Resolution::try_from(max_res).unwrap();
        group.bench_with_input(
            BenchmarkId::new("LatLng/Scanline", max_res),
            &current_res,
            |b, &res| {
                b.iter(|| {
                    coords
                        .iter()
                        .copied()
                        .map(|coord| black_box(coord).to_cell(black_box(res)))
                        .for_each(drop)
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Indexed/Scanline", max_res),
            &current_res,
            |b, &res| {
                b.iter(|| {
                    let mut indexer = LatLngIndexer::default();
                    coords
                        .iter()
                        .copied()
                        .map(|coord| {
                            indexer.to_cell(black_box(coord), black_box(res))
                        })
                        .for_each(drop)
                })
            },
        );

        // Scanline at multiple resolutions in a single pass.
        group.bench_with_input(
            BenchmarkId::new("LatLng/Combined", max_res),
            &max_res,
            |b, _| {
                b.iter(|| {
                    for coord in coords.iter().copied() {
                        resolutions
                            .iter()
                            .copied()
                            .map(move |res| {
                                black_box(coord).to_cell(black_box(res))
                            })
                            .for_each(drop)
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Indexed/Combined", max_res),
            &max_res,
            |b, _| {
                b.iter(|| {
                    let mut indexer = LatLngIndexer::default();
                    for coord in coords.iter().copied() {
                        indexer
                            .to_cells(
                                black_box(coord),
                                resolutions.iter().copied(),
                            )
                            .for_each(drop)
                    }
                })
            },
        );
    }

    group.finish();
}

fn build_grid() -> Vec<LatLng> {
    let min = LatLng::new(39.7341989, -104.9990547).unwrap();
    let max = LatLng::new(39.7441989, -104.9790547).unwrap();

    let step_lat = (max.lat() - min.lat()) / 100.;
    let step_lng = (max.lng() - min.lng()) / 100.;

    let mut coords = Vec::with_capacity(100 * 100);
    let mut lat = min.lat();
    while lat < max.lat() {
        let mut lng = min.lng();
        while lng < max.lng() {
            coords.push(LatLng::new(lat, lng).unwrap());
            lng += step_lng;
        }
        lat += step_lat;
    }

    coords
}
