use criterion::{black_box, Bencher, Criterion};
use h3o::LatLng;

const SRC: (f64, f64) = (48.854501508844095, 2.3729695423293613);
const DST: (f64, f64) = (33.988491214456516, -118.47327934764078);

pub fn bench_rads(c: &mut Criterion) {
    bench_distance(
        c,
        "greatCircleDistanceRads",
        |b, src, dst| b.iter(|| black_box(src).distance_rads(dst)),
        |b, src, dst| {
            b.iter(|| unsafe {
                h3ron_h3_sys::greatCircleDistanceRads(
                    &black_box(src),
                    &black_box(dst),
                )
            })
        },
    );
}

pub fn bench_km(c: &mut Criterion) {
    bench_distance(
        c,
        "greatCircleDistanceKm",
        |b, src, dst| b.iter(|| black_box(src).distance_km(dst)),
        |b, src, dst| {
            b.iter(|| unsafe {
                h3ron_h3_sys::greatCircleDistanceKm(
                    &black_box(src),
                    &black_box(dst),
                )
            })
        },
    );
}

pub fn bench_m(c: &mut Criterion) {
    bench_distance(
        c,
        "greatCircleDistanceM",
        |b, src, dst| b.iter(|| black_box(src).distance_m(dst)),
        |b, src, dst| {
            b.iter(|| unsafe {
                h3ron_h3_sys::greatCircleDistanceM(
                    &black_box(src),
                    &black_box(dst),
                )
            })
        },
    );
}

// -----------------------------------------------------------------------------

fn bench_distance<F, G>(
    c: &mut Criterion,
    name: &'static str,
    mut bench_h3o: F,
    mut bench_h3: G,
) where
    F: FnMut(&mut Bencher<'_>, LatLng, LatLng),
    G: FnMut(&mut Bencher<'_>, h3ron_h3_sys::LatLng, h3ron_h3_sys::LatLng),
{
    let mut group = c.benchmark_group(name);

    group.bench_function("h3o", |b| {
        let src = LatLng::new(SRC.0, SRC.1).expect("src");
        let dst = LatLng::new(DST.0, DST.1).expect("dst");
        bench_h3o(b, src, dst)
    });
    group.bench_function("h3", |b| {
        let src = h3ron_h3_sys::LatLng {
            lat: SRC.0,
            lng: SRC.1,
        };
        let dst = h3ron_h3_sys::LatLng {
            lat: DST.0,
            lng: DST.1,
        };
        bench_h3(b, src, dst)
    });

    group.finish();
}
