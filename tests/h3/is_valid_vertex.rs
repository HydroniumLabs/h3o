use super::h3api;
use h3o::VertexIndex;

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let result = VertexIndex::try_from($index).is_ok();
            let reference = h3api::is_valid_vertex($index);

            assert_eq!(result, reference);
        }
    };
}

test!(valid, 0x2222597fffffffff);
test!(hexagon_vertex0, 0x2222597fffffffff);
test!(hexagon_vertex1, 0x2523d47fffffffff);
test!(hexagon_vertex2, 0x2423d47fffffffff);
test!(hexagon_vertex3, 0x21224b7fffffffff);
test!(hexagon_vertex4, 0x20224b7fffffffff);
test!(hexagon_vertex5, 0x2322597fffffffff);

test!(high_bit_set, 0xa322597fffffffff);
test!(invalid_owner, 0x2222597ffffffffe);
// Vertex 0 belong to 0x80000016f57b1f0, not 0x823d6ffffffffff.
test!(wrong_owner, 0x2023d6ffffffffff);
test!(cell_index, 0x80000016f57b1f0);
test!(invalid_hexagon_vertex, 0x2622597fffffffff);
test!(invalid_pentagon_vertex, 0x2523007fffffffff);
