#![no_main]

use h3o::{CellIndex, Resolution};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    resolution: Resolution,
    cells: Vec<u64>,
}

fuzz_target!(|args: Args| {
    let cells = args
        .cells
        .into_iter()
        .filter_map(|bits| CellIndex::try_from(bits).ok())
        .filter(|cell| cell.resolution() <= args.resolution)
        .collect::<Vec<_>>();

    // Skip inputs that would produce too many indexes.
    if CellIndex::uncompact_size(cells.iter().copied(), args.resolution)
        > 4_000_000
    {
        return;
    }

    let uncompacted =
        CellIndex::uncompact(cells.iter().copied(), args.resolution)
            .collect::<Vec<_>>();

    // Check that every ouput cell is present in the input either as itself
    // or as an ancestor.
    let mut i = 0;
    for cell in uncompacted {
        loop {
            let candidate = cells[i];

            if cell.parent(candidate.resolution()) == Some(candidate) {
                break;
            }
            i += 1;
            if i >= cells.len() {
                panic!("{cell} missing");
            }
        }
    }
});
