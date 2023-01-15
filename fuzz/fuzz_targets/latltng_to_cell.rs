#![no_main]

use h3o::{CellIndex, LatLng, Resolution};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct Args {
    ll: LatLng,
    res: Resolution,
}

fuzz_target!(|args: Args| {
    assert_eq!(args.ll.to_cell(args.res), latlng_to_cell(args.ll, args.res));
});

// H3 wrappers {{{

fn latlng_to_cell(ll: LatLng, resolution: Resolution) -> CellIndex {
    let mut out: u64 = 0;
    let ll = h3ron_h3_sys::LatLng {
        lat: ll.lat_radians(),
        lng: ll.lng_radians(),
    };
    unsafe {
        h3ron_h3_sys::latLngToCell(&ll, u8::from(resolution).into(), &mut out);
    }
    CellIndex::try_from(out).expect("cell index")
}

// }}}
