use crate::error::{
    CompactionError, HexGridError, InvalidBaseCell, InvalidCellIndex,
    InvalidDirectedEdgeIndex, InvalidDirection, InvalidEdge, InvalidFace,
    InvalidLatLng, InvalidResolution, InvalidVertex, InvalidVertexIndex,
    LocalIjError, ResolutionMismatch,
};
#[cfg(feature = "geo")]
use crate::error::{InvalidGeometry, OutlinerError};
use std::error::Error;

// All error must have a non-empty display.
#[test]
fn display() {
    let hex_grid_error = HexGridError::new("error");

    assert!(!CompactionError::HeterogeneousResolution
        .to_string()
        .is_empty());
    assert!(!CompactionError::DuplicateInput.to_string().is_empty());

    assert!(!hex_grid_error.to_string().is_empty());

    assert!(!InvalidResolution::new(Some(32), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidCellIndex::new(Some(0), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidDirectedEdgeIndex::new(Some(0), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidVertexIndex::new(Some(0), "error")
        .to_string()
        .is_empty());
    assert!(!InvalidLatLng::new(f64::NAN, "error").to_string().is_empty());
    assert!(!InvalidEdge::new(7, "error").to_string().is_empty());
    assert!(!InvalidVertex::new(8, "error").to_string().is_empty());
    assert!(!InvalidFace::new(33, "error").to_string().is_empty());
    assert!(!InvalidBaseCell::new(128, "error").to_string().is_empty());
    assert!(!InvalidDirection::new(9, "error").to_string().is_empty());

    assert!(!LocalIjError::ResolutionMismatch.to_string().is_empty());
    assert!(!LocalIjError::Pentagon.to_string().is_empty());
    assert!(!LocalIjError::HexGrid(hex_grid_error).to_string().is_empty());

    assert!(!ResolutionMismatch.to_string().is_empty());

    #[cfg(feature = "geo")]
    assert!(!InvalidGeometry::new("error").to_string().is_empty());

    #[cfg(feature = "geo")]
    assert!(!OutlinerError::HeterogeneousResolution
        .to_string()
        .is_empty());
    #[cfg(feature = "geo")]
    assert!(!OutlinerError::DuplicateInput.to_string().is_empty());
}

// All errors are root errors.
#[test]
fn source() {
    let hex_grid_error = HexGridError::new("error");

    assert!(CompactionError::HeterogeneousResolution.source().is_none());
    assert!(CompactionError::DuplicateInput.source().is_none());

    assert!(hex_grid_error.source().is_none());

    assert!(InvalidResolution::new(Some(32), "error").source().is_none());
    assert!(InvalidCellIndex::new(Some(0), "error").source().is_none());
    assert!(InvalidDirectedEdgeIndex::new(Some(0), "error")
        .source()
        .is_none());
    assert!(InvalidVertexIndex::new(Some(0), "error").source().is_none());
    assert!(InvalidLatLng::new(f64::NAN, "error").source().is_none());
    assert!(InvalidEdge::new(7, "error").source().is_none());
    assert!(InvalidVertex::new(8, "error").source().is_none());
    assert!(InvalidFace::new(33, "error").source().is_none());
    assert!(InvalidBaseCell::new(128, "error").source().is_none());
    assert!(InvalidDirection::new(9, "error").source().is_none());

    assert!(LocalIjError::ResolutionMismatch.source().is_none());
    assert!(LocalIjError::Pentagon.source().is_none());
    assert!(LocalIjError::HexGrid(hex_grid_error).source().is_some());

    assert!(ResolutionMismatch.source().is_none());

    #[cfg(feature = "geo")]
    assert!(InvalidGeometry::new("error").source().is_none());

    #[cfg(feature = "geo")]
    assert!(OutlinerError::HeterogeneousResolution.source().is_none());
    #[cfg(feature = "geo")]
    assert!(OutlinerError::DuplicateInput.source().is_none());
}
