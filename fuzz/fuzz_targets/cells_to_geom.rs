#![no_main]

use h3o::CellIndex;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<u64>| {
    let cells = data
        .into_iter()
        .filter_map(|bits| CellIndex::try_from(bits).ok())
        .take(1024) // Limit to 1024 cells to avoid looooooooong exec time.
        .collect::<Vec<_>>();
    let cell_count = cells.len();
    let polygons = h3o::geom::dissolve(cells);

    assert!(cell_count >= polygons.map(|mp| mp.0.len()).unwrap_or_default());
});
