#![no_main]

use geo_types::{Coord, LineString, Polygon};
use h3o::{geom::TilerBuilder, Resolution};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    resolution: Resolution,
    values: Vec<f64>,
}

fuzz_target!(|args: Args| {
    if args.values.len() < 6 {
        // Not enough point for a polygon.
        return;
    }

    let mut ring = LineString::new(
        args.values
            .chunks_exact(2)
            .map(|chunk| Coord {
                x: chunk[0],
                y: chunk[1],
            })
            .collect(),
    );
    ring.close();
    // Can still return false if the first point contains NaN.
    if !ring.is_closed() {
        return;
    }
    let mut tiler = TilerBuilder::new(args.resolution).build();
    if tiler.add(Polygon::new(ring, Vec::new())).is_ok() {
        if tiler.coverage_size_hint() > 4_000_000 {
            return;
        }
        tiler.into_coverage().for_each(drop);
    }
});
