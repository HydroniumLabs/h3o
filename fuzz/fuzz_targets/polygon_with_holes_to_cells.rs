#![no_main]

use geo_types as geo;
use h3o::{
    geom::{PolyfillConfig, Polygon, ToCells},
    Resolution,
};
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
            let mut ring = geo::LineString::new(
                coords
                    .chunks_exact(2)
                    .map(|chunk| geo::Coord {
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

    if let Ok(polygon) = Polygon::from_degrees(geo::Polygon::new(outer, rings))
    {
        let config = PolyfillConfig::new(args.resolution);
        let upper_bound = polygon.max_cells_count(config);

        if upper_bound > 4_000_000 {
            return;
        }
        polygon.to_cells(config).for_each(drop);
    }
});
