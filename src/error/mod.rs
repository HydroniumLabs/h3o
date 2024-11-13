//! H3O error types.

mod compaction;
mod hex_grid;
mod invalid_value;
mod localij;
mod resolution_mismatch;

#[cfg(feature = "geo")]
mod geom;

#[cfg(test)]
mod tests;

pub use compaction::CompactionError;
pub use hex_grid::HexGridError;
pub use invalid_value::{
    InvalidBaseCell, InvalidCellIndex, InvalidDirectedEdgeIndex,
    InvalidDirection, InvalidEdge, InvalidFace, InvalidLatLng,
    InvalidResolution, InvalidVertex, InvalidVertexIndex,
};
pub use localij::LocalIjError;
pub use resolution_mismatch::ResolutionMismatch;

#[cfg(feature = "geo")]
pub use geom::{InvalidGeometry, OutlinerError};
