use super::h3api;
use h3o::CellIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.vertexes().collect::<Vec<_>>();
            let reference = h3api::cell_to_vertexes(index);

            assert_eq!(result, reference);
        }
    };
}

test!(hexagon, 0x8f0800000000000);
test!(pentagon, 0x8f734e64992d6d8);
