#![no_main]

use h3o::{CellIndex, Resolution};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    index: CellIndex,
    res: Resolution,
}

fuzz_target!(|args: Args| {
    assert_eq!(
        args.index.parent(args.res),
        cell_to_parent(args.index, args.res),
        "cellToParent"
    );
    assert_eq!(
        args.index.center_child(args.res),
        cell_to_center_child(args.index, args.res),
        "cellToCenterChild"
    );

    assert_eq!(
        args.index.child_position(args.res),
        cell_to_child_pos(args.index, args.res),
        "cellToParent"
    );

    // Do not generate children when the generation gap is too large (OOM risk).
    if u8::from(args.res).saturating_sub(u8::from(args.index.resolution())) < 10
    {
        assert_eq!(
            args.index.children(args.res).collect::<Vec<_>>(),
            cell_to_children(args.index, args.res),
            "cellToChildren"
        );
    }
});

// H3 wrappers {{{

fn cell_to_parent(
    cell: CellIndex,
    resolution: Resolution,
) -> Option<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToParent(cell.into(), resolution.into(), &mut out)
    };
    (res == 0).then(|| CellIndex::try_from(out).expect("cell index"))
}

fn cell_to_center_child(
    cell: CellIndex,
    resolution: Resolution,
) -> Option<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToCenterChild(
            cell.into(),
            resolution.into(),
            &mut out,
        )
    };
    (res == 0).then(|| CellIndex::try_from(out).expect("cell index"))
}

fn cell_to_children(cell: CellIndex, resolution: Resolution) -> Vec<CellIndex> {
    let size = cell.children_count(resolution);
    let mut out = vec![0; usize::try_from(size).expect("too many children")];
    let res = unsafe {
        h3ron_h3_sys::cellToChildren(
            cell.into(),
            u8::from(resolution).into(),
            out.as_mut_ptr(),
        )
    };
    if res != 0 {
        return Vec::new();
    }
    out.into_iter()
        .map(|index| CellIndex::try_from(index).expect("cell index"))
        .collect()
}

pub fn cell_to_child_pos(index: CellIndex, resolution: Resolution) -> Option<u64> {
    let resolution = u8::from(resolution);
    let mut out: i64 = 0;
    unsafe {
        let res = h3ron_h3_sys::cellToChildPos(
            index.into(),
            resolution.into(),
            &mut out,
        );
        match res {
            0 => Some(out as u64),
            // E_RES_DOMAIN: when index res == res, H3 error while we return 0.
            4 => Some(0),
            _ => None,
        }
    }
}

// }}}
