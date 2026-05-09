//! Implement an accurate floating-point adder.
//!
//! Current implementation is based on the Kahan summation algorithm, but this
//! is subject to change in the future.
//!
//! # References
//! - <https://en.wikipedia.org/wiki/Kahan_summation_algorithm> for the
//!   algorithm description.

use core::ops::AddAssign;

#[derive(Clone, Copy, Default)]
pub struct FloatAdder {
    sum: f64,
    correction: f64,
}

impl From<FloatAdder> for f64 {
    #[inline]
    fn from(value: FloatAdder) -> Self {
        value.sum + value.correction
    }
}

impl AddAssign<f64> for FloatAdder {
    #[inline]
    fn add_assign(&mut self, value: f64) {
        let y = value - self.correction;
        let t = self.sum + y;
        self.correction = (t - self.sum) - y;
        self.sum = t;
    }
}

#[cfg(test)]
#[path = "./fp_adder_tests.rs"]
mod tests;
