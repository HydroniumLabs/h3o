#![no_main]

use h3o::{CellIndex, CoordIJ, LocalIJ};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    anchor: CellIndex,
    i: i32,
    j: i32,
}

fuzz_target!(|args: Args| {
    let local_ij = LocalIJ {
        anchor: args.anchor,
        coord: CoordIJ::new(args.i, args.j),
    };

    assert_eq!(
        CellIndex::try_from(local_ij).ok(),
        local_ij_to_cell(local_ij),
        "localIjToCell"
    );
});

// H3 wrappers {{{

pub fn local_ij_to_cell(local_ij: LocalIJ) -> Option<CellIndex> {
    let mut out: u64 = 0;
    let ij = h3ron_h3_sys::CoordIJ {
        i: local_ij.coord.i,
        j: local_ij.coord.j,
    };
    let res = unsafe {
        h3ron_h3_sys::localIjToCell(local_ij.anchor.into(), &ij, 0, &mut out)
    };

    (res == 0).then(|| CellIndex::try_from(out).expect("H3 index"))
}

// }}}
