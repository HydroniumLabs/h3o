use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! exhaustive_test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                let result = index.grid_distance(index).unwrap_or(-1);
                let reference =
                    h3api::grid_distance(index, index).unwrap_or(-1);
                assert_eq!(result, reference, "distance to self for {index}");

                for neighbor in index.grid_disk::<Vec<_>>(5) {
                    let result = index.grid_distance(neighbor).unwrap_or(-1);
                    let reference =
                        h3api::grid_distance(index, neighbor).unwrap_or(-1);
                    assert_eq!(
                        result, reference,
                        "distance from {index} to {neighbor}"
                    );
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
