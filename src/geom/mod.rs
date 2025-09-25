//! Bridge between H3 entities and geometrical shapes.

mod plotter;
mod ring_hierarchy;
mod solvent;
mod tiler;
mod vertex_graph;

use ring_hierarchy::RingHierarchy;
use vertex_graph::VertexGraph;

pub use plotter::{Plotter, PlotterBuilder};
pub use solvent::{Solvent, SolventBuilder};
pub use tiler::{AnnotatedCell, ContainmentMode, Tiler, TilerBuilder};

// Required for the From<CellIndex> for MultiPolygon implementation.
pub(crate) use tiler::cell_boundary;

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

// Return the immediate neighbors, no memory allocations.
fn neighbors(cell: crate::CellIndex, scratchpad: &mut [u64]) -> usize {
    let mut count = 0;

    // Don't use `grid_disk` to avoid the allocation,
    // use the pre-allocated scratchpad memory instead.
    for candidate in cell.grid_disk_fast(1) {
        if let Some(neighbor) = candidate {
            scratchpad[count] = neighbor.into();
            count += 1;
        } else {
            count = 0;
            break;
        }
    }

    // Unsafe version failed, fallback on the safe version.
    if count == 0 {
        for candidate in cell.grid_disk_safe(1) {
            scratchpad[count] = candidate.into();
            count += 1;
        }
    }

    count
}
