use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal, $resolution:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result = index.parent(resolution);
            let reference = h3api::cell_to_parent(index, resolution);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, 0x8f734e64992d6d8, 0);
test!(res1, 0x8f734e64992d6d8, 1);
test!(res2, 0x8f734e64992d6d8, 2);
test!(res3, 0x8f734e64992d6d8, 3);
test!(res4, 0x8f734e64992d6d8, 4);
test!(res5, 0x8f734e64992d6d8, 5);
test!(res6, 0x8f734e64992d6d8, 6);
test!(res7, 0x8f734e64992d6d8, 7);
test!(res8, 0x8f734e64992d6d8, 8);
test!(res9, 0x8f734e64992d6d8, 9);
test!(res10, 0x8f734e64992d6d8, 10);
test!(res11, 0x8f734e64992d6d8, 11);
test!(res12, 0x8f734e64992d6d8, 12);
test!(res13, 0x8f734e64992d6d8, 13);
test!(res14, 0x8f734e64992d6d8, 14);
test!(res15, 0x8f734e64992d6d8, 15);
