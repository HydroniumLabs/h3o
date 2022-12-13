//! H3O error types.

mod hex_grid;
mod invalid_value;
#[cfg(test)]
mod tests;

pub use hex_grid::HexGridError;
pub use invalid_value::{
    InvalidBaseCell, InvalidCellIndex, InvalidDirectedEdgeIndex,
    InvalidDirection, InvalidEdge, InvalidFace, InvalidLatLng,
    InvalidResolution, InvalidVertex,
};
