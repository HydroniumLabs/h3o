#![no_main]

use float_eq::assert_float_eq;
use h3o::{
    BaseCell, Boundary, CellIndex, DirectedEdgeIndex, Face, LatLng, Resolution,
    VertexIndex,
};
use libfuzzer_sys::fuzz_target;

const EPSILON: f64 = 1.7453292519943295e-11;

fuzz_target!(|index: CellIndex| {
    assert_eq!(index.resolution(), get_resolution(index), "getResolution");
    assert_eq!(
        index.base_cell(),
        get_base_cell_number(index),
        "getBaseCellNumber"
    );
    assert_eq!(index.is_pentagon(), is_pentagon(index), "isPentagon");
    assert_eq!(
        index.resolution().is_class3(),
        is_res_class3(index.resolution()),
        "isResClassIII"
    );
    assert_eq!(
        index.icosahedron_faces().iter().collect::<Vec<_>>(),
        get_icosahedron_faces(index),
        "getIcosahedronFaces"
    );

    assert_float_eq!(
        index.area_rads2(),
        cell_area_rads2(index),
        abs <= EPSILON,
        "cellAreaRads2"
    );
    assert_float_eq!(
        index.area_km2(),
        cell_area_km2(index),
        r2nd <= f64::from(f32::EPSILON),
        "cellAreaKm2"
    );
    assert_float_eq!(
        index.area_m2(),
        cell_area_m2(index),
        r2nd <= f64::from(f32::EPSILON),
        "cellAreaM2"
    );

    assert_eq!(LatLng::from(index), cell_to_latlng(index), "cellToLatLng");
    assert!(
        index
            .boundary()
            .iter()
            .zip(cell_to_boundary(index).iter())
            .all(|(v1, v2)| v1 == v2),
        "cellToBoundary"
    );

    assert_eq!(
        index.edges().collect::<Vec<_>>(),
        origin_to_directed_edges(index),
        "originToDirectedEdges"
    );

    assert_eq!(
        index.vertexes().collect::<Vec<_>>(),
        cell_to_vertexes(index),
        "cellToVertexes"
    );
});

// H3 wrappers {{{

fn get_resolution(index: CellIndex) -> Resolution {
    unsafe {
        Resolution::try_from(h3ron_h3_sys::getResolution(index.into()) as u8)
            .expect("index resolution")
    }
}

fn get_base_cell_number(index: CellIndex) -> BaseCell {
    unsafe {
        BaseCell::try_from(h3ron_h3_sys::getBaseCellNumber(index.into()) as u8)
            .expect("base cell")
    }
}

fn is_pentagon(index: CellIndex) -> bool {
    unsafe { h3ron_h3_sys::isPentagon(index.into()) == 1 }
}

fn is_res_class3(resolution: Resolution) -> bool {
    // XXX: invalid index but we don't care here, only resolution matter.
    let index = 0x0802_5fff_ffff_ffff | (u64::from(resolution) << 52);
    unsafe { h3ron_h3_sys::isResClassIII(index) == 1 }
}

fn get_icosahedron_faces(index: CellIndex) -> Vec<Face> {
    let max_count = index.max_face_count();

    let mut out = vec![0; max_count];
    unsafe {
        h3ron_h3_sys::getIcosahedronFaces(index.into(), out.as_mut_ptr());
    }

    let mut res = out
        .into_iter()
        .filter(|&value| value != -1)
        .map(|value| Face::try_from(value as u8).expect("icosahedron face"))
        .collect::<Vec<_>>();
    res.sort();
    res
}

fn cell_area_km2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaKm2(index.into(), &mut area);
    }
    area
}

fn cell_area_m2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaM2(index.into(), &mut area);
    }
    area
}

fn cell_area_rads2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaRads2(index.into(), &mut area);
    }
    area
}

fn cell_to_latlng(index: CellIndex) -> LatLng {
    let mut ll = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    unsafe {
        h3ron_h3_sys::cellToLatLng(index.into(), &mut ll);
    }
    LatLng::from_radians(ll.lat, ll.lng).expect("coordinate")
}

fn cell_to_boundary(index: CellIndex) -> Boundary {
    let mut result = h3ron_h3_sys::CellBoundary {
        numVerts: 0,
        verts: [h3ron_h3_sys::LatLng { lat: 0., lng: 0. }; 10],
    };
    unsafe {
        h3ron_h3_sys::cellToBoundary(index.into(), &mut result);
    }

    let mut boundary = Boundary::new();
    for i in 0..(result.numVerts as usize) {
        boundary.push(
            LatLng::from_radians(result.verts[i].lat, result.verts[i].lng)
                .expect("vertex coordinate"),
        );
    }

    boundary
}

fn origin_to_directed_edges(index: CellIndex) -> Vec<DirectedEdgeIndex> {
    let mut out = [0; 6];

    unsafe {
        h3ron_h3_sys::originToDirectedEdges(index.into(), out.as_mut_ptr());
    }

    out.into_iter()
        .filter(|&index| index != 0)
        .map(|index| DirectedEdgeIndex::try_from(index).expect("edge index"))
        .collect()
}

fn cell_to_vertexes(cell: CellIndex) -> Vec<VertexIndex> {
    let mut out = [0; 6];

    unsafe {
        h3ron_h3_sys::cellToVertexes(cell.into(), out.as_mut_ptr());
    }

    out.into_iter()
        .filter(|&index| index != 0)
        .map(|index| VertexIndex::try_from(index).expect("vertex index"))
        .collect()
}

// }}}
