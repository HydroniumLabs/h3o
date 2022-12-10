use criterion::{black_box, Criterion};
use h3o::CellIndex;
use std::{ffi::CString, fmt::Write};

const INPUT: u64 = 0x8f734e64992d6d8;
const SIZE: usize = 16; // u64 is at most a 16-char hexstring.

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("h3ToString");

    group.bench_function("h3o", |b| {
        let index = CellIndex::try_from(INPUT).expect("cell index");
        let mut s = String::with_capacity(SIZE);
        b.iter(|| write!(&mut s, "{}", black_box(index)))
    });
    group.bench_function("h3", |b| {
        let buf = CString::new(vec![1u8; SIZE]).expect("CString");
        let ptr = buf.into_raw();
        b.iter(|| unsafe {
            // +1 for the nul byte.
            h3ron_h3_sys::h3ToString(black_box(INPUT), ptr, SIZE + 1)
        })
    });

    group.finish();
}
