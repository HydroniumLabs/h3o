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
                for offset in 0..=3 {
                    let children_res =
                        Resolution::try_from(u8::from(resolution) + offset)
                            .expect("valid resolution");
                    for child in index.children(children_res) {
                        let result = child.child_position(resolution);
                        let expected =
                            h3api::cell_to_child_pos(child, resolution);

                        assert_eq!(result, expected, "index:{child:?}");
                    }
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
