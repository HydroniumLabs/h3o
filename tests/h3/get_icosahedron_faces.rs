use super::h3api;
use h3o::{CellIndex, Resolution};

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.icosahedron_faces().iter().collect::<Vec<_>>();
            let reference = h3api::get_icosahedron_faces(index);

            assert_eq!(result, reference);
        }
    };
}

// Class II pentagon neighbor - one face, two adjacent vertices on edge.
test!(hexagon_with_edge_vertices, 0x821c37fffffffff);
// Class III pentagon neighbor, distortion across faces.
test!(hexagon_with_distortion, 0x831c06fffffffff);
// Class II hex with two vertices on edge.
test!(hexagon_crossing_faces, 0x821ce7fffffffff);

test!(class3_pentagon, 0x81083ffffffffff);
test!(class2_pentagon, 0x820807fffffffff);
test!(resolution15_pentagon, 0x8f0800000000000);

#[test]
fn base_cells() {
    for base_cell in CellIndex::base_cells() {
        let result = base_cell.icosahedron_faces().iter().collect::<Vec<_>>();
        let reference = h3api::get_icosahedron_faces(base_cell);

        assert_eq!(result, reference, "base cell {base_cell}");
    }
}

macro_rules! test_cells_at_res {
    ($name:ident, $base_cell:literal, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let base_cell =
                CellIndex::try_from($base_cell).expect("cell index");

            for index in base_cell.children(resolution) {
                let result =
                    index.icosahedron_faces().iter().collect::<Vec<_>>();
                let reference = h3api::get_icosahedron_faces(index);

                assert_eq!(result, reference, "cell {index}");
            }
        }
    };
}

// Base cell 16 is at the center of an icosahedron face, so all children should
// have the same face.
test_cells_at_res!(single_face_hexes_res2, 0x8021fffffffffff, 2);
test_cells_at_res!(single_face_hexes_res3, 0x8021fffffffffff, 3);
