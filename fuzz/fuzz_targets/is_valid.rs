#![no_main]

use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bits: u64| {
    assert_eq!(
        CellIndex::try_from(bits).is_ok(),
        is_valid_cell(bits),
        "isValidCell"
    );
    assert_eq!(
        DirectedEdgeIndex::try_from(bits).is_ok(),
        is_valid_directed_edge(bits),
        "isValidDirectedEdge"
    );
    assert_eq!(
        VertexIndex::try_from(bits).is_ok(),
        is_valid_vertex(bits),
        "isValidVertex"
    );
});

// H3 wrappers {{{

fn is_valid_cell(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidCell(index) == 1 }
}

fn is_valid_directed_edge(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidDirectedEdge(index) == 1 }
}

fn is_valid_vertex(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidVertex(index) == 1 }
}

// }}}
