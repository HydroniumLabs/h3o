//! Bridge between H3 entities and geometrical shapes.

mod plotter;
mod ring_hierarchy;
mod tiler;
mod vertex_graph;

use ring_hierarchy::RingHierarchy;
use vertex_graph::VertexGraph;

pub use plotter::{Plotter, PlotterBuilder};
pub use tiler::{ContainmentMode, Tiler, TilerBuilder};

/// Creates a [`MultiPolygon`](geo::MultiPolygon) describing the outline(s) of a
/// set of cells.
///
/// # Errors
///
/// All cell indexes must be unique and have the same resolution, otherwise
/// [`DissolutionError`](crate::error::DissolutionError) is returned.
///
/// # Example
///
/// ```
/// use h3o::{CellIndex, Resolution};
///
/// let index = CellIndex::try_from(0x089283470803ffff)?;
/// let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();
/// let geom = h3o::geom::dissolve(cells)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn dissolve(
    cells: impl IntoIterator<Item = crate::CellIndex>,
) -> Result<geo::MultiPolygon, crate::error::DissolutionError> {
    VertexGraph::from_cells(cells).map(Into::into)
}

// ----------------------------------------------------------------------------

// Check that the coordinate are finite and in a legit range.
fn coord_is_valid(coord: geo::Coord) -> bool {
    use crate::TWO_PI;
    use std::f64::consts::PI;

    coord.x.is_finite()
        && coord.y.is_finite()
        && coord.x >= -TWO_PI
        && coord.x <= TWO_PI
        && coord.y >= -PI
        && coord.y <= PI
}
