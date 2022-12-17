use super::h3api;
use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};

macro_rules! test {
    ($name:ident, $index:expr) => {
        #[test]
        fn $name() {
            let index = $index.expect("H3 index");
            let result = index.to_string();
            let reference = h3api::h3_to_string(index);

            assert_eq!(result, reference);
        }
    };
}

test!(cell_index_res0, CellIndex::try_from(0x802bfffffffffff));
test!(cell_index_res12, CellIndex::try_from(0x8c2bae305336bff));
test!(cell_index_res15, CellIndex::try_from(0x8f2834782b9c2ab));
test!(
    edge_index_res0,
    DirectedEdgeIndex::try_from(0x1302bfffffffffff)
);
test!(
    edge_index_res12,
    DirectedEdgeIndex::try_from(0x13c2bae305336bff)
);
test!(
    edge_index_res15,
    DirectedEdgeIndex::try_from(0x15f2834782b9c2ab)
);
test!(vertex_index_res0, VertexIndex::try_from(0x2302bfffffffffff));
test!(
    vertex_index_res12,
    VertexIndex::try_from(0x23f2834782b9c2a8)
);
test!(
    vertex_index_res15,
    VertexIndex::try_from(0x21c2bae305330dff)
);
