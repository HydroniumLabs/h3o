use super::h3api;
use h3o::Resolution;

macro_rules! test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result = resolution.is_class3();
            let reference = h3api::is_res_class3(resolution);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, 0);
test!(res1, 1);
test!(res2, 2);
test!(res3, 3);
test!(res4, 4);
test!(res5, 5);
test!(res6, 6);
test!(res7, 7);
test!(res8, 8);
test!(res9, 9);
test!(res10, 10);
test!(res11, 11);
test!(res12, 12);
test!(res13, 13);
test!(res14, 14);
test!(res15, 15);
