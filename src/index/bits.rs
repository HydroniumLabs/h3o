//! Bit twiddling.

use super::IndexMode;
use crate::Resolution;

/// Offset (in bits) of the mode in an H3 index.
const MODE_OFFSET: usize = 59;
const MODE_MASK: u64 = 0b1111 << MODE_OFFSET;

/// Offset (in bits) of the cell edge in an H3 index.
const EDGE_OFFSET: usize = 56;
const EDGE_MASK: u64 = 0b111 << EDGE_OFFSET;

/// Offset (in bits) of the cell vertex in an H3 index.
const VERTEX_OFFSET: usize = 56;
const VERTEX_MASK: u64 = 0b111 << VERTEX_OFFSET;

/// The bit offset of the resolution in an H3 index.
const RESOLUTION_OFFSET: u64 = 52;
// Bitmask to select the resolution bits in an H3 index.
const RESOLUTION_MASK: u64 = 0b1111 << RESOLUTION_OFFSET;

/// Offset (in bits) of the base cell in an H3 index.
const BASE_CELL_OFFSET: u64 = 45;
// Bitmask to select the base cell bits in an H3 index.
const BASE_CELL_MASK: u64 = 0b111_1111 << BASE_CELL_OFFSET;

// Bitmask to select the directions bits in an H3 index.
pub const DIRECTIONS_MASK: u64 = 0x0000_1fff_ffff_ffff;

/// Returns the H3 index mode  bits.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub const fn get_mode(bits: u64) -> u8 {
    ((bits & MODE_MASK) >> MODE_OFFSET) as u8
}

/// Clears the H3 index mode bits.
#[must_use]
pub const fn clr_mode(bits: u64) -> u64 {
    bits & !MODE_MASK
}

/// Sets the H3 index mode bits.
#[must_use]
pub const fn set_mode(bits: u64, mode: IndexMode) -> u64 {
    clr_mode(bits) | ((mode as u64) << MODE_OFFSET)
}

/// Returns the H3 index cell edge bits.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub const fn get_edge(bits: u64) -> u8 {
    ((bits & EDGE_MASK) >> EDGE_OFFSET) as u8
}

/// Clears the H3 index cell edge bits.
#[must_use]
pub const fn clr_edge(bits: u64) -> u64 {
    bits & !EDGE_MASK
}

/// Returns the H3 index cell vertex bits.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub const fn get_vertex(bits: u64) -> u8 {
    ((bits & VERTEX_MASK) >> VERTEX_OFFSET) as u8
}

/// Clears the H3 index cell vertex bits.
#[must_use]
pub const fn clr_vertex(bits: u64) -> u64 {
    bits & !VERTEX_MASK
}

/// Returns the H3 index resolution.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub const fn get_resolution(bits: u64) -> Resolution {
    // SAFETY: the masking restricts the value on 4 bits (thus 0-15).
    Resolution::new_unchecked(
        ((bits & RESOLUTION_MASK) >> RESOLUTION_OFFSET) as u8,
    )
}

/// Clears the H3 index resolution bits.
#[must_use]
pub const fn clr_resolution(bits: u64) -> u64 {
    bits & !RESOLUTION_MASK
}

/// Sets the H3 index resolution bits.
#[must_use]
pub fn set_resolution(bits: u64, resolution: Resolution) -> u64 {
    clr_resolution(bits) | (u64::from(resolution) << RESOLUTION_OFFSET)
}

/// Returns the H3 index base cell bits.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub const fn get_base_cell(bits: u64) -> u8 {
    ((bits & BASE_CELL_MASK) >> BASE_CELL_OFFSET) as u8
}

/// Sets the H3 index base cell bits.
#[must_use]
pub fn set_base_cell(bits: u64, cell: u8) -> u64 {
    (bits & !BASE_CELL_MASK) | (u64::from(cell) << BASE_CELL_OFFSET)
}

/// Returns the H3 index direction bits at the given resolution.
#[allow(clippy::cast_possible_truncation)] // Cast safe thx to masking.
#[must_use]
pub fn get_direction(bits: u64, resolution: Resolution) -> u8 {
    ((bits & resolution.direction_mask()) >> resolution.direction_offset())
        as u8
}

/// Sets unused directions in an H3 index at the given resolution.
#[must_use]
pub fn set_unused(bits: u64, resolution: Resolution) -> u64 {
    let unused_end_offset = resolution.direction_offset();
    let unused_bits = (1 << unused_end_offset) - 1;

    bits | unused_bits
}
