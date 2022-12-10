//! Expose some H3 functions to allow testing against the reference
//! implementation.

use h3o::{BaseCell, CellIndex, DirectedEdgeIndex, Resolution};
use std::{ffi::CString, fmt::Debug, os::raw::c_int};

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

/// Expose `degsToRads`.
pub fn degs_to_rads(angle: f64) -> f64 {
    unsafe { h3ron_h3_sys::degsToRads(angle) }
}

/// Expose `getBaseCellNumber`.
pub fn get_base_cell_number(index: CellIndex) -> BaseCell {
    unsafe {
        BaseCell::try_from(h3ron_h3_sys::getBaseCellNumber(index.into()) as u8)
            .expect("base cell")
    }
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

/// Expose `getHexagonEdgeLengthAvgKm`.
pub fn get_hexagon_edge_length_avg_km(resolution: Resolution) -> f64 {
    let resolution = u8::from(resolution);
    let mut out: f64 = 0.;
    unsafe {
        let res = h3ron_h3_sys::getHexagonEdgeLengthAvgKm(
            resolution.into(),
            &mut out,
        );
        assert_eq!(res, 0, "getHexagonEdgeLengthAvgKm");
    }
    out
}

/// Expose `getHexagonEdgeLengthAvgM`.
pub fn get_hexagon_edge_length_avg_m(resolution: Resolution) -> f64 {
    let resolution = u8::from(resolution);
    let mut out: f64 = 0.;
    unsafe {
        let res =
            h3ron_h3_sys::getHexagonEdgeLengthAvgM(resolution.into(), &mut out);
        assert_eq!(res, 0, "getHexagonEdgeLengthAvgM");
    }
    out
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

/// Expose `maxFaceCount`.
pub fn max_face_count(index: CellIndex) -> usize {
    let mut out: c_int = 0;
    unsafe {
        let res = h3ron_h3_sys::maxFaceCount(index.into(), &mut out);
        assert_eq!(res, 0, "maxFaceCount");
    }
    out as usize
}

/// Expose `pentagonCount`.
pub fn pentagon_count() -> u8 {
    unsafe { h3ron_h3_sys::pentagonCount() as u8 }
}

/// Expose `radsToDegs`.
pub fn rads_to_degs(angle: f64) -> f64 {
    unsafe { h3ron_h3_sys::radsToDegs(angle) }
}

/// Expose `res0CellCount​`.
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
