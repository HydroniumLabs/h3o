use super::h3api;
use h3o::{LatLng, Resolution};

macro_rules! test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let ll = LatLng::new(56.66170660104207, 20.46973734604441)
                .expect("coordinate");
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let index = ll.to_cell(resolution);

            for res in 0..=15 {
                let child_res =
                    Resolution::try_from(res).expect("child resolution");
                let result = index.center_child(child_res);
                let reference = h3api::cell_to_center_child(index, child_res);

                assert_eq!(result, reference, "center child at {child_res:?}");
            }
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
