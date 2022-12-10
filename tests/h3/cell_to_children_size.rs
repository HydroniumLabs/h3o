use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal, $resolution:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result = index.children_count(resolution);
            let reference = h3api::cell_to_children_size(index, resolution);

            assert_eq!(result, reference);
        }
    };
}

test!(coarser_hexagon, 0x87283080dffffff, 3);
test!(same_resolution_hexagon, 0x87283080dffffff, 7);
test!(children_hexagon, 0x87283080dffffff, 8);
test!(grand_children_hexagon, 0x87283080dffffff, 9);
test!(highest_resolution_hexagon, 0x806dfffffffffff, 15);
test!(coarser_pentagon, 0x870800000ffffff, 3);
test!(same_resolution_pentagon, 0x870800000ffffff, 7);
test!(children_pentagon, 0x870800000ffffff, 8);
test!(grand_children_pentagon, 0x870800000ffffff, 9);
test!(highest_resolution_pentagon, 0x8009fffffffffff, 15);
