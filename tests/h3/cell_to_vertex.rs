use super::h3api;
use h3o::{CellIndex, Resolution, Vertex};

macro_rules! exhaustive_test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                for vertex in 0..6 {
                    let vertex = Vertex::try_from(vertex).expect("cell vertex");
                    let result = index.vertex(vertex);
                    let reference = h3api::cell_to_vertex(index, vertex);

                    assert_eq!(
                        result,
                        reference,
                        "index {index}/vertex {}",
                        u8::from(vertex)
                    );
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
