//! Expose some H3 functions to allow testing against the reference
//! implementation.

use h3o::{
    BaseCell, Boundary, CellIndex, CoordIJ, DirectedEdgeIndex, Face, LatLng,
    LocalIJ, Resolution, Vertex, VertexIndex,
};
use std::{ffi::CString, fmt::Debug, os::raw::c_int};

/// Expose `areNeighborCells`.
pub fn are_neighbor_cells(origin: CellIndex, index: CellIndex) -> Option<bool> {
    let mut out: c_int = 0;
    let res = unsafe {
        h3ron_h3_sys::areNeighborCells(origin.into(), index.into(), &mut out)
    };
    (res == 0).then_some(out == 1)
}

/// Expose `cellAreaKm2`.
pub fn cell_area_km2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaKm2(index.into(), &mut area);
    }
    area
}

/// Expose `cellAreaM2`.
pub fn cell_area_m2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaM2(index.into(), &mut area);
    }
    area
}

/// Expose `cellAreaRads2`.
pub fn cell_area_rads2(index: CellIndex) -> f64 {
    let mut area: f64 = 0.;
    unsafe {
        h3ron_h3_sys::cellAreaRads2(index.into(), &mut area);
    }
    area
}

/// Expose `cellToBoundary`.
pub fn cell_to_boundary(index: CellIndex) -> Boundary {
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

/// Expose `cellToCenterChild`.
pub fn cell_to_center_child(
    cell: CellIndex,
    resolution: Resolution,
) -> Option<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToCenterChild(
            cell.into(),
            resolution.into(),
            &mut out,
        )
    };
    (res == 0).then(|| CellIndex::try_from(out).expect("cell index"))
}

/// Expose `cellToChildrenSize`.
pub fn cell_to_children_size(index: CellIndex, resolution: Resolution) -> u64 {
    let resolution = u8::from(resolution);
    let mut out: i64 = 0;
    unsafe {
        let res = h3ron_h3_sys::cellToChildrenSize(
            index.into(),
            resolution.into(),
            &mut out,
        );
        match res {
            0 => out as u64,
            // E_RES_DOMAIN: when index res == res, H3 error while we return 0.
            4 => 0,
            _ => panic!("cellToChildrenSize"),
        }
    }
}

/// Expose `cellToChildPos`.
pub fn cell_to_child_pos(
    index: CellIndex,
    resolution: Resolution,
) -> Option<u64> {
    let resolution = u8::from(resolution);
    let mut out: i64 = 0;
    unsafe {
        let res = h3ron_h3_sys::cellToChildPos(
            index.into(),
            resolution.into(),
            &mut out,
        );
        match res {
            0 => Some(out as u64),
            // E_RES_DOMAIN: when index res == res, H3 error while we return 0.
            4 => Some(0),
            _ => None,
        }
    }
}

/// Expose `cellToChildren`.
pub fn cell_to_children(
    cell: CellIndex,
    resolution: Resolution,
) -> Vec<CellIndex> {
    let size = cell_to_children_size(cell, resolution);
    let mut out = vec![0; usize::try_from(size).expect("too many children")];
    unsafe {
        let res = h3ron_h3_sys::cellToChildren(
            cell.into(),
            u8::from(resolution).into(),
            out.as_mut_ptr(),
        );
        assert_eq!(res, 0, "cellToChildren");
    }
    out.into_iter()
        .map(|index| CellIndex::try_from(index).expect("cell index"))
        .collect()
}

/// Expose `cellToLatLng`.
pub fn cell_to_latlng(index: CellIndex) -> LatLng {
    let mut ll = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    unsafe {
        h3ron_h3_sys::cellToLatLng(index.into(), &mut ll);
    }
    LatLng::from_radians(ll.lat, ll.lng).expect("coordinate")
}

/// Expose `cellToLocalIj`.
pub fn cell_to_local_ij(
    origin: CellIndex,
    index: CellIndex,
) -> Option<LocalIJ> {
    let mut out = h3ron_h3_sys::CoordIJ { i: 0, j: 0 };
    let res = unsafe {
        h3ron_h3_sys::cellToLocalIj(origin.into(), index.into(), 0, &mut out)
    };
    (res == 0).then(|| LocalIJ::new(origin, CoordIJ::new(out.i, out.j)))
}

/// Expose `cellToParent`.
pub fn cell_to_parent(
    cell: CellIndex,
    resolution: Resolution,
) -> Option<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToParent(cell.into(), resolution.into(), &mut out)
    };
    (res == 0).then(|| CellIndex::try_from(out).expect("cell index"))
}

/// Expose `cellToVertex`.
pub fn cell_to_vertex(cell: CellIndex, vertex: Vertex) -> Option<VertexIndex> {
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellToVertex(
            cell.into(),
            u8::from(vertex).into(),
            &mut out,
        )
    };
    (res == 0).then(|| VertexIndex::try_from(out).expect("vertex index"))
}

/// Expose `cellToVertexes`
pub fn cell_to_vertexes(cell: CellIndex) -> Vec<VertexIndex> {
    let mut out = [0; 6];

    unsafe {
        h3ron_h3_sys::cellToVertexes(cell.into(), out.as_mut_ptr());
    }

    out.into_iter()
        .filter_map(|index| {
            (index != 0)
                .then(|| VertexIndex::try_from(index).expect("vertex index"))
        })
        .collect()
}

/// Expose `cellsToDirectedEdge`
pub fn cells_to_directed_edge(
    origin: CellIndex,
    destination: CellIndex,
) -> Option<DirectedEdgeIndex> {
    let mut out: u64 = 0;
    let res = unsafe {
        h3ron_h3_sys::cellsToDirectedEdge(
            origin.into(),
            destination.into(),
            &mut out,
        )
    };
    (res == 0).then(|| DirectedEdgeIndex::try_from(out).expect("edge index"))
}

/// Expose `childPosToCell`.
pub fn child_pos_to_cell(
    parent: CellIndex,
    position: u64,
    resolution: Resolution,
) -> Option<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out: u64 = 0;
    unsafe {
        let res = h3ron_h3_sys::childPosToCell(
            position as i64,
            parent.into(),
            resolution.into(),
            &mut out,
        );
        match res {
            0 => Some(CellIndex::try_from(out).expect("cell index")),
            _ => None,
        }
    }
}

/// Expose `compactCells`.
pub fn compact_cells(cells: &[CellIndex]) -> Option<Vec<CellIndex>> {
    let mut out = vec![0; cells.len()];
    let res = unsafe {
        // SAFETY: `CellIndex` is `repr(transparent)`
        let cells = &*(cells as *const [CellIndex] as *const [u64]);
        h3ron_h3_sys::compactCells(
            cells.as_ptr(),
            out.as_mut_ptr(),
            cells.len() as i64,
        )
    };
    (res == 0).then(|| {
        out.into_iter()
            .filter_map(|index| {
                (index != 0)
                    .then(|| CellIndex::try_from(index).expect("cell index"))
            })
            .collect()
    })
}

/// Expose `degsToRads`.
pub fn degs_to_rads(angle: f64) -> f64 {
    unsafe { h3ron_h3_sys::degsToRads(angle) }
}

/// Expose `directedEdgeToBoundary`.
pub fn directed_edge_to_boundary(index: DirectedEdgeIndex) -> Boundary {
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
            LatLng::from_radians(result.verts[i].lat, result.verts[i].lng)
                .expect("vertex coordinate"),
        );
    }

    boundary
}

/// Expose `directedEdgeToCells`.
pub fn directed_edge_to_cells(
    index: DirectedEdgeIndex,
) -> (CellIndex, CellIndex) {
    let mut out = [0; 2];
    unsafe {
        h3ron_h3_sys::directedEdgeToCells(index.into(), out.as_mut_ptr());
    }

    (
        CellIndex::try_from(out[0]).expect("edge origin"),
        CellIndex::try_from(out[1]).expect("edge destination"),
    )
}

/// Expose `edgeLengthKm`.
pub fn edge_length_km(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthKm(index.into(), &mut length);
    }
    length
}

/// Expose `edgeLengthM`.
pub fn edge_length_m(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthM(index.into(), &mut length);
    }
    length
}

/// Expose `edgeLengthRads`.
pub fn edge_length_rads(index: DirectedEdgeIndex) -> f64 {
    let mut length: f64 = 0.;
    unsafe {
        h3ron_h3_sys::edgeLengthRads(index.into(), &mut length);
    }
    length
}

/// Expose `getBaseCellNumber`.
pub fn get_base_cell_number(index: CellIndex) -> BaseCell {
    unsafe {
        BaseCell::try_from(h3ron_h3_sys::getBaseCellNumber(index.into()) as u8)
            .expect("base cell")
    }
}

/// Expose `getDirectedEdgeDestination`.
pub fn get_directed_edge_destination(index: DirectedEdgeIndex) -> CellIndex {
    let mut out: u64 = 0;
    unsafe {
        h3ron_h3_sys::getDirectedEdgeDestination(index.into(), &mut out);
    }
    CellIndex::try_from(out).expect("cell index")
}

/// Expose `getDirectedEdgeOrigin`.
pub fn get_directed_edge_origin(index: DirectedEdgeIndex) -> CellIndex {
    let mut out: u64 = 0;
    unsafe {
        h3ron_h3_sys::getDirectedEdgeOrigin(index.into(), &mut out);
    }
    CellIndex::try_from(out).expect("cell index")
}

/// Expose `getHexagonAreaAvgKm2`.
pub fn get_hexagon_area_avg_km2(resolution: Resolution) -> f64 {
    let resolution = u8::from(resolution);
    let mut out: f64 = 0.;
    unsafe {
        let res =
            h3ron_h3_sys::getHexagonAreaAvgKm2(resolution.into(), &mut out);
        assert_eq!(res, 0, "getHexagonAreaAvgKm2");
    }
    out
}

/// Expose `getHexagonAreaAvgM2`.
pub fn get_hexagon_area_avg_m2(resolution: Resolution) -> f64 {
    let resolution = u8::from(resolution);
    let mut out: f64 = 0.;
    unsafe {
        let res =
            h3ron_h3_sys::getHexagonAreaAvgM2(resolution.into(), &mut out);
        assert_eq!(res, 0, "getHexagonAreaAvgM2");
    }
    out
}

/// Expose `getIcosahedronFaces`.
pub fn get_icosahedron_faces(index: CellIndex) -> Vec<Face> {
    let max_count = max_face_count(index);

    let mut out = vec![0; max_count];
    unsafe {
        h3ron_h3_sys::getIcosahedronFaces(index.into(), out.as_mut_ptr());
    }

    let mut res = out
        .into_iter()
        .filter_map(|value| {
            (value != -1)
                .then(|| Face::try_from(value as u8).expect("icosahedron face"))
        })
        .collect::<Vec<_>>();
    res.sort();
    res
}

/// Expose `getNumCells`.
pub fn get_num_cells(resolution: Resolution) -> u64 {
    let resolution = u8::from(resolution);
    let mut out: i64 = 0;
    unsafe {
        let res = h3ron_h3_sys::getNumCells(resolution.into(), &mut out);
        assert_eq!(res, 0, "getNumCells");
    }
    out as u64
}

/// Expose `getPentagons`.
pub fn get_pentagons(resolution: Resolution) -> Vec<CellIndex> {
    let resolution = u8::from(resolution);
    let mut out = [0u64; 12];
    unsafe {
        h3ron_h3_sys::getPentagons(resolution.into(), out.as_mut_ptr());
    }
    out.into_iter()
        .map(|index| CellIndex::try_from(index).expect("cell index"))
        .collect()
}

/// Expose `getRes0Cells`.
pub fn get_res0_cells() -> Vec<CellIndex> {
    let mut out = [0u64; 122];
    unsafe {
        h3ron_h3_sys::getRes0Cells(out.as_mut_ptr());
    }
    out.into_iter()
        .map(|index| CellIndex::try_from(index).expect("cell index"))
        .collect()
}

/// Expose `getResolution`.
pub fn get_resolution(index: CellIndex) -> Resolution {
    unsafe {
        Resolution::try_from(h3ron_h3_sys::getResolution(index.into()) as u8)
            .expect("index resolution")
    }
}

/// Expose `greatCircleDistanceKm`.
pub fn great_circle_distance_km(src: &LatLng, dst: &LatLng) -> f64 {
    let src = h3ron_h3_sys::LatLng {
        lat: src.lat_radians(),
        lng: src.lng_radians(),
    };
    let dst = h3ron_h3_sys::LatLng {
        lat: dst.lat_radians(),
        lng: dst.lng_radians(),
    };
    unsafe { h3ron_h3_sys::greatCircleDistanceKm(&src, &dst) }
}

/// Expose `greatCircleDistanceM`.
pub fn great_circle_distance_m(src: &LatLng, dst: &LatLng) -> f64 {
    let src = h3ron_h3_sys::LatLng {
        lat: src.lat_radians(),
        lng: src.lng_radians(),
    };
    let dst = h3ron_h3_sys::LatLng {
        lat: dst.lat_radians(),
        lng: dst.lng_radians(),
    };
    unsafe { h3ron_h3_sys::greatCircleDistanceM(&src, &dst) }
}

/// Expose `greatCircleDistanceRads`.
pub fn great_circle_distance_rads(src: &LatLng, dst: &LatLng) -> f64 {
    let src = h3ron_h3_sys::LatLng {
        lat: src.lat_radians(),
        lng: src.lng_radians(),
    };
    let dst = h3ron_h3_sys::LatLng {
        lat: dst.lat_radians(),
        lng: dst.lng_radians(),
    };
    unsafe { h3ron_h3_sys::greatCircleDistanceRads(&src, &dst) }
}

/// Expose `gridDisk`.
pub fn grid_disk(origin: CellIndex, k: u32) -> Vec<CellIndex> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDisk(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistances");
    }

    cells
        .into_iter()
        .filter_map(|cell| {
            (cell != 0).then(|| CellIndex::try_from(cell).expect("cell index"))
        })
        .collect()
}

/// Expose `gridDiskDistances`.
pub fn grid_disk_distances(origin: CellIndex, k: u32) -> Vec<(CellIndex, u32)> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDiskDistances(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistances");
    }

    cells
        .into_iter()
        .zip(distances.into_iter())
        .filter_map(|(cell, distance)| {
            (cell != 0).then(|| {
                (
                    CellIndex::try_from(cell).expect("cell index"),
                    distance as u32,
                )
            })
        })
        .collect()
}

/// Expose `gridDiskDistancesSafe`.
pub fn grid_disk_distances_safe(
    origin: CellIndex,
    k: u32,
) -> Vec<(CellIndex, u32)> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    unsafe {
        let res = h3ron_h3_sys::gridDiskDistancesSafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        );
        assert_eq!(res, 0, "gridDiskDistancesSafe");
    }

    cells
        .into_iter()
        .zip(distances.into_iter())
        .filter_map(|(cell, distance)| {
            (cell != 0).then(|| {
                (
                    CellIndex::try_from(cell).expect("cell index"),
                    distance as u32,
                )
            })
        })
        .collect()
}

/// Expose `gridDiskDistancesUnsafe`.
pub fn grid_disk_distances_unsafe(
    origin: CellIndex,
    k: u32,
) -> Option<Vec<(CellIndex, u32)>> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];
    let mut distances: Vec<c_int> = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDiskDistancesUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
            distances.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .zip(distances.into_iter())
            .filter_map(|(cell, distance)| {
                (cell != 0).then(|| {
                    (
                        CellIndex::try_from(cell).expect("cell index"),
                        distance as u32,
                    )
                })
            })
            .collect()
    })
}

/// Expose `gridDiskUnsafe`.
pub fn grid_disk_unsafe(origin: CellIndex, k: u32) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDiskUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

/// Expose `gridDisksUnsafe`.
pub fn grid_disks_unsafe(
    origins: impl IntoIterator<Item = CellIndex>,
    k: u32,
) -> Option<Vec<CellIndex>> {
    let mut origins = origins.into_iter().map(u64::from).collect::<Vec<_>>();
    let size = origins.len()
        * usize::try_from(max_grid_disk_size(k)).expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridDisksUnsafe(
            origins.as_mut_ptr(),
            origins.len() as c_int,
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

/// Expose `gridDistance`.
pub fn grid_distance(src: CellIndex, dst: CellIndex) -> Option<i32> {
    let mut out = 0;
    let res =
        unsafe { h3ron_h3_sys::gridDistance(src.into(), dst.into(), &mut out) };
    (res == 0).then(|| i32::try_from(out).expect("distance overflow"))
}

/// Expose `gridPathCells`.
pub fn grid_path_cells(
    src: CellIndex,
    dst: CellIndex,
) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(grid_path_cells_size(src, dst)?)
        .expect("path too long");
    let mut out = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridPathCells(src.into(), dst.into(), out.as_mut_ptr())
    };

    (res == 0).then(|| {
        out.into_iter()
            .map(|cell| CellIndex::try_from(cell).expect("cell index"))
            .collect()
    })
}

/// Expose `gridPathCellsSize`.
pub fn grid_path_cells_size(src: CellIndex, dst: CellIndex) -> Option<i32> {
    let mut out = 0;
    let res = unsafe {
        h3ron_h3_sys::gridPathCellsSize(src.into(), dst.into(), &mut out)
    };
    (res == 0).then(|| i32::try_from(out).expect("distance overflow"))
}

/// Expose `gridRingUnsafe`.
pub fn grid_ring_unsafe(origin: CellIndex, k: u32) -> Option<Vec<CellIndex>> {
    let size = usize::try_from(if k == 0 { 1 } else { 6 * k })
        .expect("grid too large");
    let mut cells = vec![0; size];

    let res = unsafe {
        h3ron_h3_sys::gridRingUnsafe(
            origin.into(),
            k as c_int,
            cells.as_mut_ptr(),
        )
    };

    (res == 0).then(|| {
        cells
            .into_iter()
            .filter_map(|cell| {
                (cell != 0)
                    .then(|| CellIndex::try_from(cell).expect("cell index"))
            })
            .collect()
    })
}

/// Expose `h3ToString`.
pub fn h3_to_string(index: impl Into<u64>) -> String {
    let buf = CString::new(vec![1u8; 16]).expect("valid CString");
    let ptr = buf.into_raw();
    unsafe {
        let res = h3ron_h3_sys::h3ToString(index.into(), ptr, 17);
        assert_eq!(res, 0, "h3ToString");
        CString::from_raw(ptr)
            .into_string()
            .expect("valid hexstring")
    }
}

/// Expose `isPentagon`.
pub fn is_pentagon(index: CellIndex) -> bool {
    unsafe { h3ron_h3_sys::isPentagon(index.into()) == 1 }
}

/// Expose `isResClassIII`.
pub fn is_res_class3(resolution: Resolution) -> bool {
    // XXX: invalid index but we don't care here, only resolution matter.
    let index = 0x0802_5fff_ffff_ffff | (u64::from(resolution) << 52);
    unsafe { h3ron_h3_sys::isResClassIII(index) == 1 }
}

/// Expose `isValidCell`.
pub fn is_valid_cell(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidCell(index) == 1 }
}

/// Expose `isValidDirectedEdge`.
pub fn is_valid_directed_edge(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidDirectedEdge(index) == 1 }
}

/// Expose `isValidVertex`.
pub fn is_valid_vertex(index: u64) -> bool {
    unsafe { h3ron_h3_sys::isValidVertex(index) == 1 }
}

/// Expose `latLngToCell`.
pub fn latlng_to_cell(ll: &LatLng, resolution: Resolution) -> CellIndex {
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

/// Expose `localIjToCell`.
pub fn local_ij_to_cell(local_ij: LocalIJ) -> Option<CellIndex> {
    let mut out: u64 = 0;
    let ij = h3ron_h3_sys::CoordIJ {
        i: local_ij.coord.i,
        j: local_ij.coord.j,
    };
    let res = unsafe {
        h3ron_h3_sys::localIjToCell(local_ij.anchor.into(), &ij, 0, &mut out)
    };

    (res == 0).then(|| CellIndex::try_from(out).expect("H3 index"))
}

/// Expose `maxFaceCount`.
pub fn max_face_count(index: CellIndex) -> usize {
    let mut out: c_int = 0;
    unsafe {
        let res = h3ron_h3_sys::maxFaceCount(index.into(), &mut out);
        assert_eq!(res, 0, "maxFaceCount");
    }
    out as usize
}

/// Expose `maxGridDiskSize`.
pub fn max_grid_disk_size(k: u32) -> u64 {
    let mut out: i64 = 0;
    unsafe {
        let res = h3ron_h3_sys::maxGridDiskSize(k as c_int, &mut out);
        assert_eq!(res, 0, "maxGridDiskSize");
    }
    out as u64
}

/// Expose `originToDirectedEdges`.
pub fn origin_to_directed_edges(index: CellIndex) -> Vec<DirectedEdgeIndex> {
    let mut out = [0; 6];

    unsafe {
        h3ron_h3_sys::originToDirectedEdges(index.into(), out.as_mut_ptr());
    }

    out.into_iter()
        .filter_map(|index| {
            (index != 0).then(|| {
                DirectedEdgeIndex::try_from(index).expect("edge index")
            })
        })
        .collect()
}

/// Expose `pentagonCount`.
pub fn pentagon_count() -> u8 {
    unsafe { h3ron_h3_sys::pentagonCount() as u8 }
}

/// Expose `radsToDegs`.
pub fn rads_to_degs(angle: f64) -> f64 {
    unsafe { h3ron_h3_sys::radsToDegs(angle) }
}

/// Expose `res0CellCount`.
pub fn res0_cell_count() -> u8 {
    unsafe { h3ron_h3_sys::res0CellCount() as u8 }
}

/// Expose `stringToH3`.
pub fn string_to_h3<T>(s: &str) -> Option<T>
where
    T: TryFrom<u64>,
    <T as TryFrom<u64>>::Error: Debug,
{
    let ptr = CString::new(s).expect("valid CString").into_raw();
    let mut out: u64 = 0;
    let res = unsafe {
        let res = h3ron_h3_sys::stringToH3(ptr, &mut out);
        let _ = CString::from_raw(ptr); // Don't leak the string!
        res
    };
    (res == 0).then(|| T::try_from(out).expect("H3 index"))
}

/// Expose `uncompactCellsSize`.
pub fn uncompact_cells_size(
    cells: &[CellIndex],
    resolution: Resolution,
) -> u64 {
    let mut out: i64 = 0;

    unsafe {
        // SAFETY: `CellIndex` is `repr(transparent)`
        let cells = &*(cells as *const [CellIndex] as *const [u64]);
        let res = h3ron_h3_sys::uncompactCellsSize(
            cells.as_ptr(),
            cells.len() as i64,
            u8::from(resolution).into(),
            &mut out,
        );
        assert_eq!(res, 0, "uncompactCellsSize");
    }

    u64::try_from(out).expect("too many expanded cells")
}

/// Expose `uncompactCells`.
pub fn uncompact_cells(
    cells: &[CellIndex],
    resolution: Resolution,
) -> Vec<CellIndex> {
    let size = uncompact_cells_size(cells, resolution);
    let mut out =
        vec![0; usize::try_from(size).expect("too many expanded cells")];

    unsafe {
        // SAFETY: `CellIndex` is `repr(transparent)`
        let cells = &*(cells as *const [CellIndex] as *const [u64]);
        let res = h3ron_h3_sys::uncompactCells(
            cells.as_ptr(),
            cells.len() as i64,
            out.as_mut_ptr(),
            size as i64,
            u8::from(resolution).into(),
        );
        assert_eq!(res, 0, "uncompactCells");
    }

    out.into_iter()
        .map(|index| CellIndex::try_from(index).expect("cell index"))
        .collect()
}

/// Expose `vertexToLatLng`.
pub fn vertex_to_latlng(index: VertexIndex) -> LatLng {
    let mut ll = h3ron_h3_sys::LatLng { lat: 0., lng: 0. };
    unsafe {
        h3ron_h3_sys::vertexToLatLng(index.into(), &mut ll);
    }
    LatLng::from_radians(ll.lat, ll.lng).expect("coordinate")
}
