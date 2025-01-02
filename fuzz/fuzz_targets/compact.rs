#![no_main]

use h3o::CellIndex;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|cells: Vec<u64>| {
    // Filter out invalid cell indexes.
    let mut cells = cells
        .into_iter()
        .filter_map(|bits| CellIndex::try_from(bits).ok())
        .collect::<Vec<_>>();

    let mut compacted = cells.clone();
    if CellIndex::compact(&mut compacted).is_ok() {
        // Check that every input cell is present in the output either as itself
        // or as an ancestor.
        let mut i = 0;
        cells.sort_unstable();
        for cell in cells {
            loop {
                let candidate = compacted[i];
                if cell.parent(candidate.resolution()) == Some(candidate) {
                    break;
                }
                i += 1;
                if i >= compacted.len() {
                    panic!("{cell} missing");
                }
            }
        }
    }
});
