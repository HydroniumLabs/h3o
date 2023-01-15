use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::{LatLng, Resolution};

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
