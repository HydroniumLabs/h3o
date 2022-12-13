use super::h3api;
use h3o::{CellIndex, LatLng, Resolution};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

const LAT_LNG: (f64, f64) = (48.854501508844095_f64, 2.3729695423293613_f64);

macro_rules! test {
    ($name:ident, $ll:expr, $resolution:literal) => {
        #[test]
        fn $name() {
            let ll = LatLng::new($ll.0.to_radians(), $ll.1.to_radians())
                .expect("coordinate");
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let result = ll.to_cell(resolution);
            let reference = h3api::latlng_to_cell(&ll, resolution);

            assert_eq!(result, reference);
        }
    };
}

test!(res0, LAT_LNG, 0);
test!(res1, LAT_LNG, 1);
test!(res2, LAT_LNG, 2);
test!(res3, LAT_LNG, 3);
test!(res4, LAT_LNG, 4);
test!(res5, LAT_LNG, 5);
test!(res6, LAT_LNG, 6);
test!(res7, LAT_LNG, 7);
test!(res8, LAT_LNG, 8);
test!(res9, LAT_LNG, 9);
test!(res10, LAT_LNG, 10);
test!(res11, LAT_LNG, 11);
test!(res12, LAT_LNG, 12);
test!(res13, LAT_LNG, 13);
test!(res14, LAT_LNG, 14);
test!(res15, LAT_LNG, 15);

// This one triggered a bug where negative latitude was mishandled.
test!(
    negative_latitude,
    (-79.704099298_f64, 209.043753147_f64),
    11
);

// This one triggered a bug in index rotations.
test!(invalid_rotation, (-60.693672001_f64, 187.742078304_f64), 11);

// # How to generate/update `latLngToCell.txt`
//
// First copy test data from the H3 repository
//
// ```
// mkdir data
// cp /path/to/h3/repo/tests/inputfiles/*centers.txt data
// ```
//
// Then, gather the data with `cat data/* > latLngToCell.tmp`
//
// Finally, sort and remove dups with `sort -u latLngToCell.tmp > latLngToCell.txt`
//
// And voilà!
#[test]
fn h3_testinput() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("dataset/latLngToCell.txt");

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("test input");
        let parts = line.split(' ').collect::<Vec<&str>>();
        let index = parts[0].parse::<CellIndex>().expect("cell index");
        let lat = parts[1].parse::<f64>().expect("latitude");
        let lng = parts[2].parse::<f64>().expect("longitude");
        let ll = LatLng::new(lat.to_radians(), lng.to_radians())
            .expect("coordinate");
        let result = ll.to_cell(index.resolution());
        let reference = h3api::latlng_to_cell(&ll, index.resolution());

        assert_eq!(result, reference, "cell {index}");
    }
}
