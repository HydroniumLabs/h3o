use super::h3api;
use h3o::CellIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let result = CellIndex::try_from($index).is_ok();
            let reference = h3api::is_valid_cell($index);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, 0x8075fffffffffff);
test!(res1, 0x81757ffffffffff);
test!(res2, 0x82754ffffffffff);
test!(res3, 0x83754efffffffff);
test!(res4, 0x84754a9ffffffff);
test!(res5, 0x85754e67fffffff);
test!(res6, 0x86754e64fffffff);
test!(res7, 0x87754e64dffffff);
test!(res8, 0x88754e6499fffff);
test!(res9, 0x89754e64993ffff);
test!(res10, 0x8a754e64992ffff);
test!(res11, 0x8b754e649929fff);
test!(res12, 0x8c754e649929dff);
test!(res13, 0x8d754e64992d6ff);
test!(res14, 0x8e754e64992d6df);
test!(res15, 0x8f754e64992d6d8);

test!(high_bit_set, 0x88c2bae305336bff);
test!(invalid_mode, 0x28c2bae305336bff);
test!(tainted_reserved_bits, 0xac2bae305336bff);
test!(invalid_base_cell, 0x80fffffffffffff);

test!(unexpected_unused_first, 0x8c2bee305336bff);
test!(unexpected_unused_middle, 0x8c2bae33d336bff);
test!(unexpected_unused_last, 0x8c2bae305336fff);

test!(missing_unused_first, 0x8c0fae305336aff);
test!(missing_unused_middle, 0x8c0fae305336fef);
test!(missing_unused_last, 0x81757fffffffffe);

test!(deleted_subsequence_hexagon1, 0x81887ffffffffff);
test!(deleted_subsequence_pentagon1, 0x81087ffffffffff);
test!(deleted_subsequence_hexagon2, 0x8804000011fffff);
test!(deleted_subsequence_pentagon2, 0x8808000011fffff);
