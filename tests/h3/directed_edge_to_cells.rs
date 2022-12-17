use super::h3api;
use h3o::DirectedEdgeIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index =
                DirectedEdgeIndex::try_from($index).expect("edge index");
            let result = index.cells();
            let reference = h3api::directed_edge_to_cells(index);

            assert_eq!(result, reference);
        }
    };
}

test!(hexagon, 0x13f2834782b9c2ab);
test!(pentagon, 0x1370800000ffffff);
