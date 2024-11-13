#![no_main]

use geo_types::{Coord, LineString, Polygon};
use h3o::{geom::TilerBuilder, Resolution};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    resolution: Resolution,
    values: Vec<Vec<f64>>,
}

fuzz_target!(|args: Args| {
    let mut rings = args
        .values
        .into_iter()
        .filter_map(|coords| {
            let mut ring = LineString::new(
                coords
                    .chunks_exact(2)
                    .map(|chunk| Coord {
                        x: chunk[0],
                        y: chunk[1],
                    })
                    .collect(),
            );
            ring.close();
            ring.is_closed().then_some(ring)
        })
        .collect::<Vec<_>>();

    if rings.len() < 2 {
        // Not enough loop for 1 ring and 1 hole.
        return;
    }
    let outer = rings.pop().expect("checked above");
    rings.truncate(100); // Avoid too many holes.

    let mut tiler = TilerBuilder::new(args.resolution).build();
    if tiler.add(Polygon::new(outer, rings)).is_ok() {
        if tiler.coverage_size_hint() > 4_000_000 {
            return;
        }
        tiler.into_coverage().for_each(drop);
    }
});
