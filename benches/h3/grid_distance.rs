use criterion::{black_box, BenchmarkId, Criterion};
use h3o::{LatLng, Resolution};

pub fn bench(c: &mut Criterion) {
    let src = LatLng::new(30.3157384429565, 104.15339644867949).expect("src");
    let dst = LatLng::new(29.794972232093798, 106.56006993629623).expect("dst");
    let mut group = c.benchmark_group("gridDistance");

    for res in 0..=15 {
        let resolution = Resolution::try_from(res).expect("resolution");
        let src = src.to_cell(resolution);
        let dst = dst.to_cell(resolution);

        group.bench_with_input(
            BenchmarkId::new("h3o", res),
            &(src, dst),
            |b, (src, dst)| {
                b.iter(|| black_box(src).grid_distance(black_box(*dst)))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("h3", res),
            &(src, dst),
            |b, (src, dst)| {
                let mut out: i64 = 0;
                let src = u64::from(*src);
                let dst = u64::from(*dst);
                b.iter(|| unsafe {
                    h3ron_h3_sys::gridDistance(src, dst, &mut out)
                })
            },
        );
    }

    group.finish();
}
