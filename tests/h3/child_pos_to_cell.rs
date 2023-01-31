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
                    for pos in 0..index.children_count(children_res) {
                        let result = index.child_at(pos, children_res);
                        let expected =
                            h3api::child_pos_to_cell(index, pos, children_res);

                        assert_eq!(
                            result, expected,
                            "parent:{index:?}, pos:{pos}, res:{children_res}"
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
