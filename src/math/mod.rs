mod area;
mod fp_adder;

#[cfg(not(feature = "std"))]
#[path = "functions-libm.rs"]
mod functions;
#[cfg(feature = "std")]
#[path = "functions-std.rs"]
mod functions;

pub use area::{Coord2d, linear_ring_area};
pub use fp_adder::FloatAdder;
pub use functions::{
    abs, acos, asin, atan, atan2, cos, hypot, mul_add, round, sin, sqrt, tan,
};
