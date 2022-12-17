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
                let result = index
                    .grid_path_cells(index)
                    .ok()
                    .and_then(|iter| iter.collect::<Result<Vec<_>, _>>().ok());
                let reference = h3api::grid_path_cells(index, index);
                assert_eq!(result, reference, "path to self for {index}");

                for neighbor in index.grid_disk::<Vec<_>>(5) {
                    let result =
                        index.grid_path_cells(neighbor).ok().and_then(|iter| {
                            iter.collect::<Result<Vec<_>, _>>().ok()
                        });
                    let reference = h3api::grid_path_cells(index, neighbor);
                    assert_eq!(
                        result, reference,
                        "path from {index} to {neighbor}"
                    );
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
