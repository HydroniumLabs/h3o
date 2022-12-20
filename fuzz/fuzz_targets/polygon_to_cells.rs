#![no_main]

use geo_types as geo;
use h3o::{
    geom::{Polygon, ToCells},
    Resolution,
};
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

    let mut ring = geo::LineString::new(
        args.values
            .chunks_exact(2)
            .map(|chunk| geo::Coord {
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

    if let Ok(polygon) =
        Polygon::from_degrees(geo::Polygon::new(ring, Vec::new()))
    {
        let upper_bound = polygon.max_cells_count(args.resolution);

        if upper_bound > 4_000_000 {
            return;
        }
        polygon.to_cells(args.resolution).for_each(drop);
    }
});
