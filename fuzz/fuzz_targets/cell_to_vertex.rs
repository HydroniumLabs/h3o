#![no_main]

use h3o::{CellIndex, Vertex, VertexIndex};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    cell: CellIndex,
    vertex_num: Vertex,
}

fuzz_target!(|args: Args| {
    assert_eq!(
        args.cell.vertex(args.vertex_num),
        cell_to_vertex(args.cell, args.vertex_num),
        "cellToVertex"
    );
});

// H3 wrappers {{{

fn cell_to_vertex(cell: CellIndex, vertex: Vertex) -> Option<VertexIndex> {
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToVertex(
            cell.into(),
            u8::from(vertex).into(),
            &mut out,
        )
    };
    (res == 0).then(|| VertexIndex::try_from(out).expect("vertex index"))
}

// }}}
