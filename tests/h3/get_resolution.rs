use super::h3api;
use h3o::CellIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.resolution();
            let reference = h3api::get_resolution(index);

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
