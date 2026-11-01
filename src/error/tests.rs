use crate::error::{
    CompactionError, HexGridError, InvalidBaseCell, InvalidCellIndex,
    InvalidDirectedEdgeIndex, InvalidDirection, InvalidEdge, InvalidFace,
    InvalidLatLng, InvalidResolution, InvalidVertex, InvalidVertexIndex,
    LocalIjError, ResolutionMismatch,
};
#[cfg(feature = "geo")]
use crate::error::{DissolutionError, InvalidGeometry, PlotterError};
use alloc::string::ToString;
use core::error::Error as _;

// All error must have a non-empty display.
#[test]
fn display() {
    let hex_grid = HexGridError::new("error");

    assert_ne!(CompactionError::HeterogeneousResolution.to_string(), "");
    assert_ne!(CompactionError::DuplicateInput.to_string(), "");

    assert_ne!(hex_grid.to_string(), "");

    assert_ne!(InvalidResolution::new(Some(32), "error").to_string(), "");
    assert_ne!(InvalidCellIndex::new(Some(0), "error").to_string(), "");
    assert_ne!(
        InvalidDirectedEdgeIndex::new(Some(0), "error").to_string(),
        ""
    );
    assert_ne!(InvalidVertexIndex::new(Some(0), "error").to_string(), "");
    assert_ne!(InvalidLatLng::new(f64::NAN, "error").to_string(), "");
    assert_ne!(InvalidEdge::new(7, "error").to_string(), "");
    assert_ne!(InvalidVertex::new(8, "error").to_string(), "");
    assert_ne!(InvalidFace::new(33, "error").to_string(), "");
    assert_ne!(InvalidBaseCell::new(128, "error").to_string(), "");
    assert_ne!(InvalidDirection::new(9, "error").to_string(), "");

    assert_ne!(LocalIjError::ResolutionMismatch.to_string(), "");
    assert_ne!(LocalIjError::Pentagon.to_string(), "");
    assert_ne!(LocalIjError::HexGrid(hex_grid).to_string(), "");

    assert_ne!(ResolutionMismatch.to_string(), "");

    #[cfg(feature = "geo")]
    {
        let invalid_geometry = InvalidGeometry::new("error");
        let local_ij = LocalIjError::Pentagon;

        assert_ne!(invalid_geometry.to_string(), "");
        assert_ne!(DissolutionError::UnsupportedResolution.to_string(), "");
        assert_ne!(DissolutionError::DuplicateInput.to_string(), "");

        assert_ne!(PlotterError::from(invalid_geometry).to_string(), "");
        assert_ne!(PlotterError::from(local_ij).to_string(), "");
    }
}

#[test]
fn source() {
    let hex_grid = HexGridError::new("error");

    assert!(CompactionError::HeterogeneousResolution.source().is_none());
    assert!(CompactionError::DuplicateInput.source().is_none());

    assert!(hex_grid.source().is_none());

    assert!(InvalidResolution::new(Some(32), "error").source().is_none());
    assert!(InvalidCellIndex::new(Some(0), "error").source().is_none());
    assert!(
        InvalidDirectedEdgeIndex::new(Some(0), "error")
            .source()
            .is_none()
    );
    assert!(InvalidVertexIndex::new(Some(0), "error").source().is_none());
    assert!(InvalidLatLng::new(f64::NAN, "error").source().is_none());
    assert!(InvalidEdge::new(7, "error").source().is_none());
    assert!(InvalidVertex::new(8, "error").source().is_none());
    assert!(InvalidFace::new(33, "error").source().is_none());
    assert!(InvalidBaseCell::new(128, "error").source().is_none());
    assert!(InvalidDirection::new(9, "error").source().is_none());

    assert!(LocalIjError::ResolutionMismatch.source().is_none());
    assert!(LocalIjError::Pentagon.source().is_none());
    assert!(LocalIjError::HexGrid(hex_grid).source().is_some());

    assert!(ResolutionMismatch.source().is_none());

    #[cfg(feature = "geo")]
    {
        let invalid_geometry = InvalidGeometry::new("error");
        let local_ij = LocalIjError::Pentagon;

        assert!(invalid_geometry.source().is_none());
        assert!(DissolutionError::UnsupportedResolution.source().is_none());
        assert!(DissolutionError::DuplicateInput.source().is_none());
        assert!(PlotterError::from(invalid_geometry).source().is_some());
        assert!(PlotterError::from(local_ij).source().is_some());
    }
}
