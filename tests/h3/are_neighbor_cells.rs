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
                let result = index.is_neighbor_with(index).ok();
                let reference = h3api::are_neighbor_cells(index, index);
                assert_eq!(
                    result, reference,
                    "self-neighbor check with {index}"
                );

                for neighbor in index.grid_disk::<Vec<_>>(3) {
                    let result = index.is_neighbor_with(neighbor).ok();
                    let reference = h3api::are_neighbor_cells(index, neighbor);
                    assert_eq!(
                        result, reference,
                        "areNeighborCells({index}, {neighbor})"
                    );
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
