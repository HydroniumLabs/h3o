use criterion::{black_box, Bencher, BenchmarkId, Criterion};
use h3o::{LatLng, VertexIndex};

const HEXAGONS: [u64; 16] = [
    0x2204bfffffffffff,
    0x251723ffffffffff,
    0x2027267fffffffff,
    0x2137348fffffffff,
    0x22473485ffffffff,
    0x215734e2bfffffff,
    0x246734e647ffffff,
    0x257734e648ffffff,
    0x208734e6491fffff,
    0x259734e64983ffff,
    0x24a734e64992ffff,
    0x25b734e649929fff,
    0x24c734e649928bff,
    0x25d734e649929d3f,
    0x24e734e649929d2f,
    0x23f734e64992d6d8,
];

const PENTAGONS: [u64; 16] = [
    0x25007fffffffffff,
    0x211083ffffffffff,
    0x2220807fffffffff,
    0x2330800fffffffff,
    0x24408001ffffffff,
    0x205080003fffffff,
    0x2160800007ffffff,
    0x2270800000ffffff,
    0x23808000001fffff,
    0x249080000003ffff,
    0x20a0800000007fff,
    0x21b0800000000fff,
    0x22c08000000001ff,
    0x23d080000000003f,
    0x24e0800000000007,
    0x20f0800000000000,
];

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertexToLatLng");

    for (resolution, index) in HEXAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Hexagon", resolution),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Hexagon", resolution),
            index,
            bench_h3,
        );
    }

    for (resolution, index) in PENTAGONS.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("h3o/Pentagon", resolution),
            index,
            bench_h3o,
        );
        group.bench_with_input(
            BenchmarkId::new("h3/Pentagon", resolution),
            index,
            bench_h3,
        );
    }
    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3o(b: &mut Bencher<'_>, index: &u64) {
    let index = VertexIndex::try_from(*index).expect("cell index");
    b.iter(|| LatLng::from(black_box(index)))
}

fn bench_h3(b: &mut Bencher<'_>, index: &u64) {
    let mut result = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    b.iter(|| unsafe {
        h3ron_h3_sys::vertexToLatLng(black_box(*index), &mut result);
    })
}
