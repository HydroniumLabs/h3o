use super::h3api;
use h3o::DirectedEdgeIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let result = DirectedEdgeIndex::try_from($index).is_ok();
            let reference = h3api::is_valid_directed_edge($index);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, 0x15075fffffffffff);
test!(res1, 0x151757ffffffffff);
test!(res2, 0x152754ffffffffff);
test!(res3, 0x153754efffffffff);
test!(res4, 0x154754a9ffffffff);
test!(res5, 0x155754e67fffffff);
test!(res6, 0x156754e64fffffff);
test!(res7, 0x157754e64dffffff);
test!(res8, 0x158754e6499fffff);
test!(res9, 0x159754e64993ffff);
test!(res10, 0x15a754e64992ffff);
test!(res11, 0x15b754e649929fff);
test!(res12, 0x15c754e649929dff);
test!(res13, 0x15d754e64992d6ff);
test!(res14, 0x15e754e64992d6df);
test!(res15, 0x15f754e64992d6d8);

test!(high_bit_set, 0x95c2bae305336bff);
test!(invalid_mode, 0x1dc2bae305336bff);
test!(invalid_edge, 0x10c2bae305336bff);
test!(deleted_pentagon_edge, 0x111083ffffffffff);
test!(invalid_base_cell, 0x150fffffffffffff);

test!(unexpected_unused_first, 0x15c2bee305336bff);
test!(unexpected_unused_middle, 0x15c2bae33d336bff);
test!(unexpected_unused_last, 0x15c2bae305336fff);

test!(missing_unused_first, 0x15c0fae305336aff);
test!(missing_unused_middle, 0x15c0fae305336fef);
test!(missing_unused_last, 0x151757fffffffffe);

test!(deleted_subsequence_hexagon1, 0x151887ffffffffff);
test!(deleted_subsequence_pentagon1, 0x151087ffffffffff);
test!(deleted_subsequence_hexagon2, 0x15804000011fffff);
test!(deleted_subsequence_pentagon2, 0x15808000011fffff);
