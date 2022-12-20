#![no_main]

use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: String| {
    if let Ok(index) = input.parse::<CellIndex>() {
        assert_eq!(
            index.to_string().parse::<CellIndex>(),
            Ok(index),
            "CellIndex"
        );
    }
    if let Ok(index) = input.parse::<DirectedEdgeIndex>() {
        assert_eq!(
            index.to_string().parse::<DirectedEdgeIndex>(),
            Ok(index),
            "DirectedEdgeIndex"
        );
    }
    if let Ok(index) = input.parse::<VertexIndex>() {
        assert_eq!(
            index.to_string().parse::<VertexIndex>(),
            Ok(index),
            "VertexIndex"
        );
    }
});
