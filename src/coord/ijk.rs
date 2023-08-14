//! IJK Coordinates
//!
//! Discrete hexagon planar grid systems naturally have 3 coordinate axes spaced
//! 120° apart. We refer to such a system as an `IJK` coordinate system, for the
//! three coordinate axes `i`, `j`, and `k`.
//!
//! Using an `IJK` coordinate system to address hexagon grid cells provides
//! multiple valid addresses for each cell. Normalizing an `IJK` address creates
//! a unique address consisting of the minimal positive `IJK` components; this
//! always results in at most two non-zero components.

#![allow(clippy::use_self)] // False positive with `auto_ops::impl_op_ex`

use super::{CoordCube, Vec2d, SQRT3_2};
use crate::{error::HexGridError, Direction};
use auto_ops::impl_op_ex;
use std::{cmp, fmt};

// -----------------------------------------------------------------------------

/// IJ hexagon coordinates.
///
/// Each axis is spaced 120 degrees apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoordIJ {
    /// `i` component.
    pub i: i32,
    /// `j` component.
    pub j: i32,
}

impl CoordIJ {
    /// Initializes a new `IJ` coordinate with the specified component values.
    #[must_use]
    pub const fn new(i: i32, j: i32) -> Self {
        Self { i, j }
    }
}

impl TryFrom<CoordIJ> for CoordIJK {
    type Error = HexGridError;

    // Returns the `IJK` coordinates corresponding to the `IJ` one.
    fn try_from(value: CoordIJ) -> Result<Self, Self::Error> {
        Self::new(value.i, value.j, 0)
            .checked_normalize()
            .ok_or_else(|| HexGridError::new("IJ coordinates overflow"))
    }
}

impl fmt::Display for CoordIJ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.i, self.j)
    }
}

// -----------------------------------------------------------------------------

/// IJK hexagon coordinates.
///
/// Each axis is spaced 120 degrees apart.
#[derive(Debug, Clone, Default, Copy, Eq, PartialEq)]
pub struct CoordIJK {
    /// `i` component.
    i: i32,
    /// `j` component.
    j: i32,
    /// `k` component.
    k: i32,
}

impl CoordIJK {
    /// Initializes a new IJK coordinate with the specified component values.
    pub const fn new(i: i32, j: i32, k: i32) -> Self {
        Self { i, j, k }
    }

    /// Returns the `i` component.
    pub const fn i(&self) -> i32 {
        self.i
    }

    /// Returns the `j` component.
    pub const fn j(&self) -> i32 {
        self.j
    }

    /// Returns the `k` component.
    pub const fn k(&self) -> i32 {
        self.k
    }

    /// Normalizes by setting the components to the smallest possible values.
    pub fn normalize(mut self) -> Self {
        let min = cmp::min(self.i, cmp::min(self.j, self.k));

        self.i -= min;
        self.j -= min;
        self.k -= min;

        self
    }

    /// Normalizes by setting the components to the smallest possible values.
    ///
    /// Guard against overflow (to be used when input comes from user).
    fn checked_normalize(mut self) -> Option<Self> {
        let min = cmp::min(self.i, cmp::min(self.j, self.k));

        self.i = self.i.checked_sub(min)?;
        self.j = self.j.checked_sub(min)?;
        self.k = self.k.checked_sub(min)?;

        Some(self)
    }

    pub fn distance(&self, other: &Self) -> i32 {
        let diff = (self - other).normalize();

        cmp::max(diff.i.abs(), cmp::max(diff.j.abs(), diff.k.abs()))
    }

    /// Returns the normalized `IJK` coordinates of the indexing parent of a
    /// cell in an aperture 7 grid.
    #[allow(clippy::cast_possible_truncation)] // On purpose.
    pub fn up_aperture7<const CCW: bool>(&self) -> Self {
        let CoordIJ { i, j } = self.into();

        let (i, j) = if CCW {
            (f64::from(3 * i - j) / 7., f64::from(i + 2 * j) / 7.)
        } else {
            (f64::from(2 * i + j) / 7., f64::from(3 * j - i) / 7.)
        };

        Self::new(i.round() as i32, j.round() as i32, 0).normalize()
    }

    /// Returns the normalized `IJK` coordinates of the indexing parent of a
    /// cell in an aperture 7 grid.
    #[allow(clippy::cast_possible_truncation)] // On purpose.
    pub fn checked_up_aperture7<const CCW: bool>(&self) -> Option<Self> {
        let CoordIJ { i, j } = self.into();

        let (i, j) = if CCW {
            (
                f64::from(i.checked_mul(3)?.checked_sub(j)?) / 7.,
                f64::from(j.checked_mul(2)?.checked_add(i)?) / 7.,
            )
        } else {
            (
                f64::from(i.checked_mul(2)?.checked_add(j)?) / 7.,
                f64::from(j.checked_mul(3)?.checked_sub(i)?) / 7.,
            )
        };

        Self::new(i.round() as i32, j.round() as i32, 0).checked_normalize()
    }

    /// Returns the normalized `IJK` coordinates of the hex centered on the
    /// indicated hex at the next finer aperture 7 resolution.
    pub fn down_aperture7<const CCW: bool>(&self) -> Self {
        // Resolution `r` unit vectors in resolution `r+1`.
        let (mut i_vec, mut j_vec, mut k_vec) = if CCW {
            (Self::new(3, 0, 1), Self::new(1, 3, 0), Self::new(0, 1, 3))
        } else {
            (Self::new(3, 1, 0), Self::new(0, 3, 1), Self::new(1, 0, 3))
        };

        i_vec *= self.i;
        j_vec *= self.j;
        k_vec *= self.k;

        (i_vec + j_vec + k_vec).normalize()
    }

    /// Returns the normalized `IJK` coordinates of the hex centered on the
    /// indicated hex at the next finer aperture 3 resolution.
    pub fn down_aperture3<const CCW: bool>(&self) -> Self {
        // Resolution `r` unit vectors in resolution `r+1`.
        let (mut i_vec, mut j_vec, mut k_vec) = if CCW {
            (Self::new(2, 0, 1), Self::new(1, 2, 0), Self::new(0, 1, 2))
        } else {
            (Self::new(2, 1, 0), Self::new(0, 2, 1), Self::new(1, 0, 2))
        };

        i_vec *= self.i;
        j_vec *= self.j;
        k_vec *= self.k;

        (i_vec + j_vec + k_vec).normalize()
    }

    /// Returns the normalized `IJK` coordinates of the hex in the specified
    /// direction from the current position.
    pub fn neighbor(&self, direction: Direction) -> Self {
        (self + direction.coordinate()).normalize()
    }

    /// Returns the `IJK` coordinates after a 60 degrees rotation.
    pub fn rotate60<const CCW: bool>(&self) -> Self {
        // Unit vector rotations.
        let (mut i_vec, mut j_vec, mut k_vec) = if CCW {
            (Self::new(1, 1, 0), Self::new(0, 1, 1), Self::new(1, 0, 1))
        } else {
            (Self::new(1, 0, 1), Self::new(1, 1, 0), Self::new(0, 1, 1))
        };
        i_vec *= self.i;
        j_vec *= self.j;
        k_vec *= self.k;

        (i_vec + j_vec + k_vec).normalize()
    }
}

impl_op_ex!(+ |lhs: &CoordIJK, rhs: &CoordIJK| -> CoordIJK {
    CoordIJK{
        i: lhs.i + rhs.i,
        j: lhs.j + rhs.j,
        k: lhs.k + rhs.k,
    }
});

impl_op_ex!(-|lhs: &CoordIJK, rhs: &CoordIJK| -> CoordIJK {
    CoordIJK {
        i: lhs.i - rhs.i,
        j: lhs.j - rhs.j,
        k: lhs.k - rhs.k,
    }
});

impl_op_ex!(*= |lhs: &mut CoordIJK, rhs: i32| {
    lhs.i *= rhs;
    lhs.j *= rhs;
    lhs.k *= rhs;
});

impl From<CoordIJK> for Vec2d {
    // Returns the center point in 2D cartesian coordinates of a hex.
    fn from(value: CoordIJK) -> Self {
        let i = f64::from(value.i - value.k);
        let j = f64::from(value.j - value.k);

        Self::new(0.5_f64.mul_add(-j, i), j * SQRT3_2)
    }
}

impl From<&CoordIJK> for CoordIJ {
    // Returns the `IJ` coordinate corresponding to the the `IJK` one.
    fn from(value: &CoordIJK) -> Self {
        Self::new(value.i - value.k, value.j - value.k)
    }
}

impl From<CoordIJK> for CoordCube {
    fn from(value: CoordIJK) -> Self {
        let i = -value.i + value.k;
        let j = value.j - value.k;
        let k = -i - j;

        Self::new(i, j, k)
    }
}

impl TryFrom<CoordIJK> for Direction {
    type Error = HexGridError;

    // Returns the direction corresponding to a given unit vector in `IJK`
    // coordinates.
    fn try_from(value: CoordIJK) -> Result<Self, Self::Error> {
        let value = value.normalize();

        // First, make sure we have a unit vector in `ijk`.
        if (value.i | value.j | value.k) & !1 == 0 {
            // Cannot truncate thx to check above (unit vector).
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let bits = (value.i << 2 | value.j << 1 | value.k) as u8;

            // SAFETY: thx to `normalize` we are guaranteed to have at most two
            // non-zero components, hence max value is 6 (thus valid).
            Ok(Self::new_unchecked(bits))
        } else {
            Err(HexGridError::new("non-unit vector in IJK coordinate"))
        }
    }
}

#[cfg(test)]
#[path = "./ijk_tests.rs"]
mod tests;
