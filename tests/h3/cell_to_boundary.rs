use super::h3api;
use h3o::CellIndex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

macro_rules! test {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            let index = CellIndex::try_from($index).expect("cell index");
            let result = index.boundary();
            let reference = h3api::cell_to_boundary(index);

            for (result, reference) in result.iter().zip(reference.iter()) {
                assert_eq!(result, reference);
            }
        }
    };
}

test!(hexagon_res0, 0x8009fffffffffff);
test!(hexagon_res1, 0x81083ffffffffff);
test!(hexagon_res2, 0x820807fffffffff);
test!(hexagon_res3, 0x830800fffffffff);
test!(hexagon_res4, 0x8408001ffffffff);
test!(hexagon_res5, 0x85080003fffffff);
test!(hexagon_res6, 0x860800007ffffff);
test!(hexagon_res7, 0x870800000ffffff);
test!(hexagon_res8, 0x8808000001fffff);
test!(hexagon_res9, 0x89080000003ffff);
test!(hexagon_res10, 0x8a0800000007fff);
test!(hexagon_res11, 0x8b0800000000fff);
test!(hexagon_res12, 0x8c08000000001ff);
test!(hexagon_res13, 0x8d080000000003f);
test!(hexagon_res14, 0x8e0800000000007);
test!(hexagon_res15, 0x8f0800000000000);

test!(pentagon_res0, 0x8073fffffffffff);
test!(pentagon_res1, 0x81737ffffffffff);
test!(pentagon_res2, 0x82734ffffffffff);
test!(pentagon_res3, 0x83734efffffffff);
test!(pentagon_res4, 0x84734a9ffffffff);
test!(pentagon_res5, 0x85734e67fffffff);
test!(pentagon_res6, 0x86734e64fffffff);
test!(pentagon_res7, 0x87734e64dffffff);
test!(pentagon_res8, 0x88734e6499fffff);
test!(pentagon_res9, 0x89734e64993ffff);
test!(pentagon_res10, 0x8a734e64992ffff);
test!(pentagon_res11, 0x8b734e649929fff);
test!(pentagon_res12, 0x8c734e649929dff);
test!(pentagon_res13, 0x8d734e64992d6ff);
test!(pentagon_res14, 0x8e734e64992d6df);
test!(pentagon_res15, 0x8f734e64992d6d8);

// Bug test for https://github.com/uber/h3/issues/45
test!(class3_edge_vertex_1, 0x894cc5349b7ffff);
test!(class3_edge_vertex_2, 0x894cc534d97ffff);
test!(class3_edge_vertex_3, 0x894cc53682bffff);
test!(class3_edge_vertex_4, 0x894cc536b17ffff);
test!(class3_edge_vertex_5, 0x894cc53688bffff);
test!(class3_edge_vertex_6, 0x894cead92cbffff);
test!(class3_edge_vertex_7, 0x894cc536537ffff);
test!(class3_edge_vertex_8, 0x894cc5acbabffff);
test!(class3_edge_vertex_9, 0x894cc536597ffff);
test!(class3_edge_vertex_exact, 0x894cc536537ffff);

// Bug test for https://github.com/uber/h3/issues/212
test!(cos_lng_constrain, 0x87dc6d364ffffff);

// # How to generate/update `cellToBoundary.txt`
//
// First copy test data from the H3 repository
//
// ```
// mkdir data
// cp /path/to/h3/repo/tests/inputfiles/*cells.txt data
// ```
//
// Then, extract H3 index only with `grep -h '^8' data/* > cellToBoundary.tmp`
//
// Finally, sort and remove dups with `sort -u cellToBoundary.tmp > cellToBoundary.txt`
//
// And voilà!
#[test]
fn h3_testinput() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("dataset/cellToBoundary.txt");

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("test input");
        let index = line.parse::<CellIndex>().expect("cell index");
        let result = index.boundary();
        let reference = h3api::cell_to_boundary(index);

        for (i, (result, reference)) in
            result.iter().zip(reference.iter()).enumerate()
        {
            assert_eq!(
                result, reference,
                "latitude (cell {index}, vertex {i})"
            );
        }
    }
}
