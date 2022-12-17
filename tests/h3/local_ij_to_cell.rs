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
                // Test self.
                let coord =
                    index.to_local_ij(index).expect("self local IJ coord");
                let result = CellIndex::try_from(coord).ok();
                let reference = h3api::local_ij_to_cell(coord);
                assert_eq!(result, reference, "anchor {index}, index {index}");

                // Test neighbors.
                for neighbor in index.grid_disk_safe(5) {
                    if let Ok(coord) = neighbor.to_local_ij(index) {
                        let result = CellIndex::try_from(coord).ok();
                        let reference = h3api::local_ij_to_cell(coord);

                        assert_eq!(
                            result, reference,
                            "anchor {index}, index {neighbor}"
                        );
                    }
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
