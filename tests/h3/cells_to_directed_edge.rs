use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("edge index");
            for edge in index.edges() {
                let destination = edge.destination();
                let result = index.edge(destination);
                let reference =
                    h3api::cells_to_directed_edge(index, destination);

                assert_eq!(result, reference, "edge {edge}");
            }
        }
    };
}

test!(hexagon, 0x891fb46622fffff);
test!(pentagon, 0x821c07fffffffff);

macro_rules! exhaustive_test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                for edge in index.edges() {
                    let destination = edge.destination();
                    let result = index.edge(destination);
                    let reference =
                        h3api::cells_to_directed_edge(index, destination);

                    assert_eq!(result, reference, "edge {edge}");
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
