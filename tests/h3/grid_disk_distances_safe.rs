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
                for k in 0..=5 {
                    let mut result =
                        index.grid_disk_distances_safe(k).collect::<Vec<_>>();
                    let mut reference =
                        h3api::grid_disk_distances_safe(index, k);

                    result.sort_unstable_by_key(|(cell, _)| *cell);
                    reference.sort_unstable_by_key(|(cell, _)| *cell);

                    assert_eq!(result, reference, "origin {index}, k {k}");
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
