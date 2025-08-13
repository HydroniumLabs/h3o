//! Bridge between H3 entities and geometrical shapes.

mod plotter;
mod ring_hierarchy;
mod solvent;
mod tiler;
mod vertex_graph;

use ring_hierarchy::RingHierarchy;
use tiler::cell_boundary;
use vertex_graph::VertexGraph;

pub use plotter::{Plotter, PlotterBuilder};
pub use solvent::{Solvent, SolventBuilder};
pub use tiler::{ContainmentMode, Tiler, TilerBuilder};

use crate::LatLng;

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

/// Return the geometry of this cell, if it crosses the trans-meridian two polygons are returned.
///
/// # Example
///
/// ```
/// let cell = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
/// let geom = cell_to_multi_polygon(cell);
/// # Ok::<(), h3o::error::InvalidCellIndex>(())
/// ```
#[must_use]
pub fn cell_to_multi_polygon(cell: crate::CellIndex) -> geo::MultiPolygon {
    let mut polygons = cell_boundary(cell);
    // converts back everything to degrees
    polygons.iter_mut().for_each(|polygon| {
        polygon.exterior_mut(|line| {
            line.coords_mut().for_each(|coord| {
                let ll = LatLng::new_unchecked(coord.y, coord.x);
                *coord = geo::coord! {
                    x: ll.lng(),
                    y: ll.lat(),
                };
            });
        });
    });
    polygons
}
