mod bbox;
mod polygon;
mod ring;

use ring::{CellBoundary, Ring};

pub use polygon::Polygon;

// ----------------------------------------------------------------------------

// Check that the coordinate are finite and in a legit range.
fn coord_is_valid(coord: geo::Coord) -> bool {
    use crate::TWO_PI;
    use core::f64::consts::PI;

    coord.x.is_finite()
        && coord.y.is_finite()
        && coord.x >= -TWO_PI
        && coord.x <= TWO_PI
        && coord.y >= -PI
        && coord.y <= PI
}
