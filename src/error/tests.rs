use crate::error::{
    HexGridError, InvalidBaseCell, InvalidCellIndex, InvalidDirectedEdgeIndex,
    InvalidDirection, InvalidEdge, InvalidFace, InvalidLatLng,
    InvalidResolution, InvalidVertex,
};
use std::error::Error;

// All error must have a non-empty display.
#[test]
fn display() {
    assert!(!HexGridError::new("error").to_string().is_empty());

    assert!(!InvalidResolution::new(32, "error").to_string().is_empty());
    assert!(!InvalidCellIndex::new(Some(0), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidDirectedEdgeIndex::new(Some(0), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidLatLng::new(f64::NAN, "error").to_string().is_empty());
    assert!(!InvalidEdge::new(7, "error").to_string().is_empty());
    assert!(!InvalidVertex::new(8, "error").to_string().is_empty());
    assert!(!InvalidFace::new(33, "error").to_string().is_empty());
    assert!(!InvalidBaseCell::new(128, "error").to_string().is_empty());
    assert!(!InvalidDirection::new(9, "error").to_string().is_empty());
}

// All errors are root errors.
#[test]
fn source() {
    assert!(HexGridError::new("error").source().is_none());

    assert!(InvalidResolution::new(32, "error").source().is_none());
    assert!(InvalidCellIndex::new(Some(0), "error").source().is_none());
    assert!(InvalidDirectedEdgeIndex::new(Some(0), "error")
        .source()
        .is_none());
    assert!(InvalidLatLng::new(f64::NAN, "error").source().is_none());
    assert!(InvalidEdge::new(7, "error").source().is_none());
    assert!(InvalidVertex::new(8, "error").source().is_none());
    assert!(InvalidFace::new(33, "error").source().is_none());
    assert!(InvalidBaseCell::new(128, "error").source().is_none());
    assert!(InvalidDirection::new(9, "error").source().is_none());
}
