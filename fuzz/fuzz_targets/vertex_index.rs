#![no_main]

use h3o::{LatLng, VertexIndex};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|index: VertexIndex| {
    assert_eq!(
        LatLng::from(index),
        vertex_to_latlng(index),
        "vertexToLatLng"
    );
});

// H3 wrappers {{{

fn vertex_to_latlng(index: VertexIndex) -> LatLng {
    let mut ll = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    unsafe {
        h3ron_h3_sys::vertexToLatLng(index.into(), &mut ll);
    }
    LatLng::new(ll.lat, ll.lng).expect("coordinate")
}

// }}}
