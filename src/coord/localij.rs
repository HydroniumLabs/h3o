//! Local IJ Coordinates
//!
//! Algorithms working with hexagons may want to refer to grid coordinates that
//! are not interrupted by base cells or faces. These coordinates have 2
//! coordinate axes spaced 120° apart, with the coordinates anchored by an
//! origin H3 index.
//!
//! - local coordinates are only comparable when they have the same origin
//!   index.
//! - local coordinates are only valid near the origin. Practically, this is
//!   within the same base cell or a neighboring base cell, except for
//!   pentagons.
//! - the coordinate space may have deleted or warped regions due to pentagon
//!   distortion.
//! - there may be multiple coordinates for the same index, with the same
//!   origin.
//! - the origin may not be at (0, 0) in the local coordinate space.

use super::{CoordIJ, CoordIJK};
use crate::{
    error::{HexGridError, LocalIjError},
    index::bits,
    BaseCell, CellIndex, Direction, Resolution, CCW, CW, DEFAULT_CELL_INDEX,
};
use std::{fmt, num::NonZeroU8};

// -----------------------------------------------------------------------------

/// `IJK` coordinates anchored by an origin.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LocalIJK {
    /// Anchor cell.
    pub anchor: CellIndex,
    /// `IJK` coordinates.
    pub coord: CoordIJK,
}

impl LocalIJK {
    /// Return the `IJK` coordinate.
    pub const fn coord(&self) -> &CoordIJK {
        &self.coord
    }
}

impl TryFrom<LocalIJK> for CellIndex {
    type Error = LocalIjError;

    fn try_from(value: LocalIJK) -> Result<Self, Self::Error> {
        let resolution = value.anchor.resolution();
        let origin_base_cell = value.anchor.base_cell();
        let origin_on_pent = origin_base_cell.is_pentagon();

        // Initialize the index.
        let mut bits = bits::set_resolution(DEFAULT_CELL_INDEX, resolution);

        // Check for res 0/base cell.
        if resolution == Resolution::Zero {
            let dir = Direction::try_from(value.coord)?;
            // Bail out if we're moving in an invalid direction off a pentagon.
            let new_base_cell = origin_base_cell
                .neighbor(dir)
                .ok_or(Self::Error::Pentagon)?;
            return Ok(Self::new_unchecked(h3o_bit::set_base_cell(
                bits,
                new_base_cell.into(),
            )));
        }

        // We need to find the correct base cell offset (if any) for this H3
        // index; start with the passed in base cell and resolution res ijk
        // coordinates in that base cell's coordinate system.
        let ijk = checked_directions_bits_from_ijk(
            value.coord,
            &mut bits,
            resolution,
        )
        .ok_or_else(|| HexGridError::new("IJ coordinates overflow"))?;

        // Lookup the correct base cell.
        let mut dir = Direction::try_from(ijk)?;
        let mut base_cell = origin_base_cell.neighbor(dir);
        // If `base_cell` is invalid, it must be because the origin base cell is
        // a pentagon, and because pentagon base cells do not border each other,
        // `base_cell` must not be a pentagon.
        let index_on_pent =
            base_cell.map(BaseCell::is_pentagon).unwrap_or_default();

        if dir != Direction::Center {
            // If the index is in a warped direction, we need to unwarp the base
            // cell direction. There may be further need to rotate the index
            // digits.
            let mut pentagon_rotations = 0;
            if origin_on_pent {
                let leading_direction = bits::first_axe(value.anchor.into())
                    .map_or_else(|| 0, NonZeroU8::get);
                pentagon_rotations = PENTAGON_ROTATIONS_REVERSE
                    [usize::from(leading_direction)][usize::from(dir)];
                assert_ne!(pentagon_rotations, 0xff);
                dir = dir.rotate60::<CCW>(pentagon_rotations.into());

                // The pentagon rotations are being chosen so that dir is not
                // the deleted direction. If it still happens, it means we're
                // moving into a deleted subsequence, so there is no index here.
                let fixed_base_cell = origin_base_cell
                    .neighbor(dir)
                    .ok_or(Self::Error::Pentagon)?;
                base_cell = Some(fixed_base_cell);
                debug_assert!(!fixed_base_cell.is_pentagon());
            }
            let fixed_base_cell = base_cell.expect("fixed base cell");

            // Now we can determine the relation between the origin and target
            // base cell.
            let base_cell_rotations = origin_base_cell.neighbor_rotation(dir);

            // Adjust for pentagon warping within the base cell. The base cell
            // should be in the right location, so now we need to rotate the
            // index back. We might not need to check for errors since we would
            // just be double mapping.
            if index_on_pent {
                let rev_dir = usize::from(
                    fixed_base_cell
                        .direction(origin_base_cell)
                        .expect("reverse direction"),
                );

                // Adjust for the different coordinate space in the two base
                // cells. This is done first because we need to do the pentagon
                // rotations based on the leading digit in the pentagon's
                // coordinate system.
                bits = bits::rotate60::<CCW>(bits, base_cell_rotations.into());

                let leading_direction = usize::from(
                    bits::first_axe(bits).map_or_else(|| 0, NonZeroU8::get),
                );
                let pentagon_rotations = if fixed_base_cell.is_polar_pentagon()
                {
                    PENTAGON_ROTATIONS_REVERSE_POLAR[rev_dir][leading_direction]
                } else {
                    PENTAGON_ROTATIONS_REVERSE_NONPOLAR[rev_dir]
                        [leading_direction]
                };
                // For this to occur, `rev_direction` would need to be 1. Since
                // `rev_direction` is from the index base cell (which is a
                // pentagon) towards the origin, this should never be the case.
                assert_ne!(pentagon_rotations, 0xff);

                bits = (0..pentagon_rotations)
                    .fold(bits, |acc, _| bits::pentagon_rotate60::<CCW>(acc));
            } else {
                assert!(pentagon_rotations != 0xff);
                let count =
                    usize::from(pentagon_rotations + base_cell_rotations);
                bits = bits::rotate60::<CCW>(bits, count);
            }
        } else if origin_on_pent && index_on_pent {
            let origin_leading_dir = usize::from(
                bits::first_axe(value.anchor.into())
                    .map_or_else(|| 0, NonZeroU8::get),
            );
            let index_leading_dir = usize::from(
                bits::first_axe(bits).map_or_else(|| 0, NonZeroU8::get),
            );

            let rotations = PENTAGON_ROTATIONS_REVERSE[origin_leading_dir]
                [index_leading_dir];
            assert!(rotations != 0xff, "invalid K axis digit");
            bits = bits::rotate60::<CCW>(bits, rotations.into());
        }

        if index_on_pent {
            // TODO: There are cases which are failed but not accounted for
            // here, instead just fail if the recovered index is invalid.
            if bits::first_axe(bits) == Direction::K.axe() {
                return Err(Self::Error::Pentagon);
            }
        }

        let base_cell = base_cell
            .ok_or_else(|| HexGridError::new("cannot resolve base cell"))?;
        Ok(Self::new_unchecked(h3o_bit::set_base_cell(
            bits,
            base_cell.into(),
        )))
    }
}

/// Set the directions of a cell index (in-place) from finest resolution up.
///
/// IJK coordinates are adjusted during the traversal so that, at the end, they
/// should match the IJK of the base cell in the coordinate system of the
/// current base cell.
///
/// Returns the adjusted `IJK` coordinates.
#[allow(clippy::inline_always)] // 4-5% boost, up to 13% at resolution 1.
#[inline(always)]
pub fn checked_directions_bits_from_ijk(
    mut ijk: CoordIJK,
    bits: &mut u64,
    resolution: Resolution,
) -> Option<CoordIJK> {
    for res in Resolution::range(Resolution::One, resolution).rev() {
        let last_ijk = ijk;
        let last_center = if res.is_class3() {
            // Rotate CCW.
            ijk = ijk.checked_up_aperture7::<{ CCW }>()?;
            ijk.down_aperture7::<{ CCW }>()
        } else {
            // Rotate CW.
            ijk = ijk.checked_up_aperture7::<{ CW }>()?;
            ijk.down_aperture7::<{ CW }>()
        };

        let diff = (last_ijk - last_center).normalize();
        let direction = Direction::try_from(diff).expect("unit IJK coordinate");
        // SAFETY: `res` is in [resolution; 1], thus valid.
        *bits = bits::set_direction(*bits, direction.into(), res);
    }

    Some(ijk)
}

// -----------------------------------------------------------------------------

/// `IJ` coordinates anchored by an origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalIJ {
    /// Anchor cell.
    pub anchor: CellIndex,
    /// `IJ` coordinates.
    pub coord: CoordIJ,
}

impl LocalIJ {
    /// Initialize a new `LocalIJ` from its components.
    ///
    /// Could be used to build invalid local IJ coordinate, only used for tests.
    #[must_use]
    pub const fn new(anchor: CellIndex, coord: CoordIJ) -> Self {
        Self { anchor, coord }
    }
}

impl TryFrom<LocalIJ> for CellIndex {
    type Error = LocalIjError;

    fn try_from(value: LocalIJ) -> Result<Self, Self::Error> {
        let local_ijk = LocalIJK {
            anchor: value.anchor,
            coord: CoordIJK::try_from(value.coord)?,
        };
        Self::try_from(local_ijk)
    }
}

impl fmt::Display for LocalIJ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.anchor, self.coord)
    }
}

// -----------------------------------------------------------------------------

// In the lookup table below, it would be nice to use `u8` with a custom niche.
// Not supported yet though: https://github.com/rust-lang/rfcs/pull/3334

/// Reverse base cell direction -> leading index digit -> rotations 60 CCW.
///
/// For reversing the rotation introduced in `PENTAGON_ROTATIONS` when the
/// origin is on a pentagon (regardless of the base cell of the index).
#[rustfmt::skip]
const PENTAGON_ROTATIONS_REVERSE: [[u8; 7]; 7] = [
    [ 0,    0,    0,    0,    0,    0,    0],    // 0
    [ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], // 1
    [ 0,    1,    0,    0,    0,    0,    0],    // 2
    [ 0,    1,    0,    0,    0,    1,    0],    // 3
    [ 0,    5,    0,    0,    0,    0,    0],    // 4
    [ 0,    5,    0,    5,    0,    0,    0],    // 5
    [ 0,    0,    0,    0,    0,    0,    0],    // 6
];

/// Reverse base cell direction -> leading index digit -> rotations 60 CCW.
///
/// For reversing the rotation introduced in `PENTAGON_ROTATIONS` when the index
/// is on a pentagon and the origin is not.
#[rustfmt::skip]
const PENTAGON_ROTATIONS_REVERSE_NONPOLAR: [[u8; 7]; 7] = [
    [ 0,    0,    0,    0,    0,    0,    0],    // 0
    [ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], // 1
    [ 0,    1,    0,    0,    0,    0,    0],    // 2
    [ 0,    1,    0,    0,    0,    1,    0],    // 3
    [ 0,    5,    0,    0,    0,    0,    0],    // 4
    [ 0,    1,    0,    5,    1,    1,    0],    // 5
    [ 0,    0,    0,    0,    0,    0,    0],    // 6
];

/// Reverse base cell direction -> leading index digit -> rotations 60 CCW.
///
/// For reversing the rotation introduced in `PENTAGON_ROTATIONS` when the index
/// is on a polar pentagon and the origin is not.
#[rustfmt::skip]
const PENTAGON_ROTATIONS_REVERSE_POLAR: [[u8; 7]; 7] = [
    [ 0,    0,    0,    0,    0,    0,    0],    // 0
    [ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], // 1
    [ 0,    1,    1,    1,    1,    1,    1],    // 2
    [ 0,    1,    0,    0,    0,    1,    0],    // 3
    [ 0,    1,    0,    0,    1,    1,    1],    // 4
    [ 0,    1,    0,    5,    1,    1,    0],    // 5
    [ 0,    1,    1,    0,    1,    1,    1],    // 6
];

#[cfg(test)]
#[path = "./localij_tests.rs"]
mod tests;
