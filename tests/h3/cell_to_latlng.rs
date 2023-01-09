use super::h3api;
use h3o::{CellIndex, LatLng};
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
            let result = LatLng::from(index);
            let reference = h3api::cell_to_latlng(index);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, 0x801ffffffffffff);
test!(res1, 0x811fbffffffffff);
test!(res2, 0x821fb7fffffffff);
test!(res3, 0x831fb4fffffffff);
test!(res4, 0x841fb47ffffffff);
test!(res5, 0x851fb467fffffff);
test!(res6, 0x861fb4667ffffff);
test!(res7, 0x871fb4662ffffff);
test!(res8, 0x881fb46623fffff);
test!(res9, 0x891fb46622fffff);
test!(res10, 0x8a1fb46622dffff);
test!(res11, 0x8b1fb46622d8fff);
test!(res12, 0x8c1fb46622d85ff);
test!(res13, 0x8d1fb46622d85bf);
test!(res14, 0x8e1fb46622d8597);
test!(res15, 0x8f1fb46622d8591);

// Those ones triggered a bug where longitude weren't in the right bounds.
test!(bound_bug1, 0x8400481ffffffff);
test!(bound_bug2, 0x8471921ffffffff);

// This one triggered a bug where an invalid CellIndex was used.
test!(index_invariant_bug, 0x840800bffffffff);

// # How to generate/update `cellToLatLng.txt`
//
// First copy test data from the H3 repository
//
// ```
// mkdir data
// cp /path/to/h3/repo/tests/inputfiles/*ic.txt data
// ```
//
// Then, extract H3 index only with `cut -f1 -d' ' data/* > cellToLatLng.tmp`
//
// Finally, sort and remove dups with `sort -u cellToLatLng.tmp > cellToLatLng.txt`
//
// And voilà!
#[test]
fn h3_testinput() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("dataset/cellToLatLng.txt");

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("test input");
        let index = line.parse::<CellIndex>().expect("cell index");
        let result = LatLng::from(index);
        let reference = h3api::cell_to_latlng(index);

        assert_eq!(result, reference, "cell {index}");
    }
}
