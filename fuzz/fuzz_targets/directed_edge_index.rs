#![no_main]

use float_eq::assert_float_eq;
use h3o::{Boundary, CellIndex, DirectedEdgeIndex, LatLng};
use libfuzzer_sys::fuzz_target;

const EPSILON: f64 = 1.7453292519943295e-11;

fuzz_target!(|index: DirectedEdgeIndex| {
    assert_eq!(
        index.origin(),
        get_directed_edge_origin(index),
        "getDirectedEdgeOrigin"
    );
    assert_eq!(
        index.destination(),
        get_directed_edge_destination(index),
        "getDirectedEdgeDestination"
    );
    assert_eq!(
        index.cells(),
        directed_edge_to_cells(index),
        "directedEdgeToCells"
    );
    assert!(
        index
            .boundary()
            .iter()
            .zip(directed_edge_to_boundary(index).iter())
            .all(|(v1, v2)| v1 == v2),
        "directedEdgeToBoundary"
    );

    // Epsilon of ~0.1mm.
    assert_float_eq!(
        index.length_rads(),
        edge_length_rads(index),
        abs <= EPSILON,
        "edgeLengthRads"
    );
    assert_float_eq!(
        index.length_km(),
        edge_length_km(index),
        abs <= 1e-7,
        "edgeLengthKm"
    );
    assert_float_eq!(
        index.length_m(),
        edge_length_m(index),
        abs <= 1e-4,
        "edgeLengthM"
    );
});

// H3 wrappers {{{

fn get_directed_edge_destination(index: DirectedEdgeIndex) -> CellIndex {
    let mut out: u64 = 0;
    unsafe {
        h3ron_h3_sys::getDirectedEdgeDestination(index.into(), &mut out);
    }
    CellIndex::try_from(out).expect("cell index")
}

fn get_directed_edge_origin(index: DirectedEdgeIndex) -> CellIndex {
    let mut out: u64 = 0;
    unsafe {
        h3ron_h3_sys::getDirectedEdgeOrigin(index.into(), &mut out);
    }
    CellIndex::try_from(out).expect("cell index")
}

fn directed_edge_to_cells(index: DirectedEdgeIndex) -> (CellIndex, CellIndex) {
    let mut out = [0; 2];
    unsafe {
        h3ron_h3_sys::directedEdgeToCells(index.into(), out.as_mut_ptr());
    }

    (
        CellIndex::try_from(out[0]).expect("edge origin"),
        CellIndex::try_from(out[1]).expect("edge destination"),
    )
}

fn directed_edge_to_boundary(index: DirectedEdgeIndex) -> Boundary {
    let mut result = h3ron_h3_sys::CellBoundary {
        numVerts: 0,
        verts: [h3ron_h3_sys::LatLng { lat: 0., lng: 0. }; 10],
    };
    unsafe {
        h3ron_h3_sys::directedEdgeToBoundary(index.into(), &mut result);
    }

    let mut boundary = Boundary::new();
    for i in 0..(result.numVerts as usize) {
        boundary.push(
            LatLng::new(result.verts[i].lat, result.verts[i].lng)
                .expect("vertex coordinate"),
        );
    }

    boundary
}

fn edge_length_km(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthKm(index.into(), &mut length);
    }
    length
}

fn edge_length_m(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthM(index.into(), &mut length);
    }
    length
}

fn edge_length_rads(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthRads(index.into(), &mut length);
    }
    length
}

// }}}
