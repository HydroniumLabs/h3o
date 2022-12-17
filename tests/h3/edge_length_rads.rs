use super::h3api;
use float_eq::assert_float_eq;
use h3o::CellIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("edge index");
            for edge in index.edges() {
                let result = edge.length_rads();
                let reference = h3api::edge_length_rads(edge);

                assert_float_eq!(
                    result,
                    reference,
                    r2nd <= f64::from(f32::EPSILON),
                    "edge {edge}",
                );
            }
        }
    };
}

test!(hexagon_res0, 0x8019fffffffffff);
test!(hexagon_res1, 0x811fbffffffffff);
test!(hexagon_res2, 0x821fb7fffffffff);
test!(hexagon_res3, 0x831fb4fffffffff);
test!(hexagon_res4, 0x841fb47ffffffff);
test!(hexagon_res5, 0x851fb467fffffff);
test!(hexagon_res6, 0x861fb4667ffffff);
test!(hexagon_res7, 0x871fb4662ffffff);
test!(hexagon_res8, 0x881fb46623fffff);
test!(hexagon_res9, 0x891fb46622fffff);
test!(hexagon_res10, 0x8a1fb46622dffff);
test!(hexagon_res11, 0x8b1fb46622dcfff);
test!(hexagon_res12, 0x8c1fb46622dc9ff);
test!(hexagon_res13, 0x8d1fb46622dc83f);
test!(hexagon_res14, 0x8e1fb46622dc81f);
test!(hexagon_res15, 0x8f1fb46622dc81b);

test!(pentagon_res0, 0x8031fffffffffff);
test!(pentagon_res1, 0x81303ffffffffff);
test!(pentagon_res2, 0x823007fffffffff);
test!(pentagon_res3, 0x833000fffffffff);
test!(pentagon_res4, 0x8430001ffffffff);
test!(pentagon_res5, 0x85300003fffffff);
test!(pentagon_res6, 0x863000007ffffff);
test!(pentagon_res7, 0x873000000ffffff);
test!(pentagon_res8, 0x8830000001fffff);
test!(pentagon_res9, 0x89300000003ffff);
test!(pentagon_res10, 0x8a3000000007fff);
test!(pentagon_res11, 0x8b3000000000fff);
test!(pentagon_res12, 0x8c30000000001ff);
test!(pentagon_res13, 0x8d300000000003f);
test!(pentagon_res14, 0x8e3000000000007);
test!(pentagon_res15, 0x8f3000000000000);
