use super::h3api;
use h3o::{CellIndex, DirectedEdgeIndex, VertexIndex};

macro_rules! test {
    ($name:ident, $string:literal, $ty:ty) => {
        #[test]
        fn $name() {
            let result = $string.parse::<$ty>().expect("cell index");
            let reference =
                h3api::string_to_h3::<$ty>($string).expect("cell index");

            assert_eq!(result, reference);
        }
    };
}

test!(cell_index_res0, "802bfffffffffff", CellIndex);
test!(cell_index_res12, "8c2bae305336bff", CellIndex);
test!(cell_index_res15, "8f2834782b9c2ab", CellIndex);
test!(edge_index_res0, "1302bfffffffffff", DirectedEdgeIndex);
test!(edge_index_res12, "13c2bae305336bff", DirectedEdgeIndex);
test!(edge_index_res15, "15f2834782b9c2ab", DirectedEdgeIndex);
test!(vertex_index_res0, "2302bfffffffffff", VertexIndex);
test!(vertex_index_res12, "23f2834782b9c2a8", VertexIndex);
test!(vertex_index_res15, "21c2bae305330dff", VertexIndex);
