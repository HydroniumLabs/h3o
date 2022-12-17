use super::h3api;
use h3o::CellIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.edges().collect::<Vec<_>>();
            let reference = h3api::origin_to_directed_edges(index);

            assert_eq!(result, reference);
        }
    };
}

test!(hexagon, 0x8f0800000000000);
test!(pentagon, 0x8f734e64992d6d8);
