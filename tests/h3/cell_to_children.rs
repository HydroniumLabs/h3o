use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal, $resolution:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result = index.children(resolution).collect::<Vec<_>>();
            let reference = h3api::cell_to_children(index, resolution);

            assert_eq!(result, reference);
        }
    };
}

test!(coarser_hexagon, 0x8b754e649929fff, 10);
test!(coarser_pentagon, 0x8b0800000000fff, 10);
test!(same_res11_hexagon, 0x8b754e649929fff, 11);
test!(same_res11_pentagon, 0x8b0800000000fff, 11);
test!(same_res0_hexagon, 0x8077fffffffffff, 0);
test!(same_res0_pentagon, 0x8009fffffffffff, 0);
test!(children_res12_hexagon, 0x8b754e649929fff, 12);
test!(children_res12_pentagon, 0x8b0800000000fff, 12);
test!(children_res1_hexagon, 0x8077fffffffffff, 1);
test!(children_res1_pentagon, 0x8075fffffffffff, 1);
test!(grand_children_res13_hexagon, 0x8b754e649929fff, 13);
test!(grand_children_res13_pentagon, 0x8b0800000000fff, 13);
test!(grand_children_res2_hexagon, 0x8077fffffffffff, 2);
test!(grand_children_res2_pentagon, 0x8075fffffffffff, 2);
test!(five_generation_res15_hexagon, 0x8b754e649929fff, 15);
test!(five_generation_res15_pentagon, 0x8b0800000000fff, 15);
test!(five_generation_res5_hexagon, 0x8077fffffffffff, 5);
test!(five_generation_res5_pentagon, 0x8075fffffffffff, 5);
