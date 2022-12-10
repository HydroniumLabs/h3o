//! H3O error types.

mod invalid_value;
#[cfg(test)]
mod tests;

pub use invalid_value::{
    InvalidBaseCell, InvalidCellIndex, InvalidDirectedEdgeIndex,
    InvalidDirection, InvalidEdge, InvalidResolution, InvalidVertex,
};
