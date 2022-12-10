use super::{bits, IndexMode};
use crate::{error, CellIndex};
use std::{cmp::Ordering, fmt, num::NonZeroU64};

/// Maximum value for a cell vertex.
const MAX: u8 = 5;

// -----------------------------------------------------------------------------

/// Vertex of an H3 cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Vertex(u8);

impl Vertex {
    /// Initializes a new cell vertex using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid cell vertex.
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        debug_assert!(value <= MAX, "cell vertex out of range");
        Self(value)
    }
}

impl TryFrom<u8> for Vertex {
    type Error = error::InvalidVertex;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX {
            return Err(Self::Error::new(value, "out of range"));
        }
        Ok(Self(value))
    }
}

impl From<Vertex> for u8 {
    fn from(value: Vertex) -> Self {
        value.0
    }
}

impl From<Vertex> for u64 {
    fn from(value: Vertex) -> Self {
        Self::from(value.0)
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------

/// Represents a single topological vertex in H3 grid system, shared by three
/// cells.
///
/// Note that this does not include the distortion vertexes occasionally present
/// in a cell's geo boundary. A vertex is arbitrarily assigned one of the three
/// neighboring cells as its "owner", which is used to calculate the canonical
/// index and geo coordinate for the vertex.
///
/// The index is encoded on 64-bit with the following bit layout:
///
/// ```text
///  ┏━┳━━━┳━━━┳━━━━━━━━━━━━━━━━━━━━━━┈┈┈┈┈┈┈┈━━━━━━━┓
///  ┃U┃ M ┃ V ┃                O                    ┃
///  ┗━┻━━━┻━━━┻━━━━━━━━━━━━━━━━━━━━━━┈┈┈┈┈┈┈┈━━━━━━━┛
/// 64 63 59   56                                    0
/// ```
///
/// Where:
/// - `U` is an unused reserved bit, always set to 0 (bit 63).
/// - `M` is the index mode, always set to 4, coded on 4 bits (59-62).
/// - `V` is the vertex number on the owner cell, in [0; 5], coded on 3 bits
///   (56-58).
/// - `O` is the owner cell index, coded on 56 bits (0-55).
///
/// Refrences:
/// - [H3 Index Representations](https://h3geo.org/docs/core-library/h3Indexing)
/// - [H3 Index Bit Layout](https://observablehq.com/@nrabinowitz/h3-index-bit-layout?collection=@nrabinowitz/h3)
/// - [H3 Index Inspector](https://observablehq.com/@nrabinowitz/h3-index-inspector?collection=@nrabinowitz/h3)
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct VertexIndex(NonZeroU64);

impl VertexIndex {
    /// Returns the cell vertex.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::VertexIndex::try_from(0x2222597fffffffff)?;
    /// assert_eq!(index.vertex(), h3o::Vertex::try_from(2)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn vertex(self) -> Vertex {
        // SAFETY: `VertexIndex` only contains valid cell vertex (invariant).
        Vertex::new_unchecked(bits::get_vertex(self.0.get()))
    }

    /// Returns the owner hexagon from the vertex index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::VertexIndex::try_from(0x2222597fffffffff)?;
    /// assert_eq!(index.owner(), h3o::CellIndex::try_from(0x822597fffffffff)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn owner(self) -> CellIndex {
        let bits = bits::set_mode(self.0.get(), IndexMode::Cell);
        CellIndex::new_unchecked(bits::clr_vertex(bits))
    }

    /// Initializes a new vertex index a value that may be invalid.
    ///
    /// # Safety
    ///
    /// The value must be a valid vertex index.
    #[cfg(test)]
    pub(crate) fn new_unchecked(value: u64) -> Self {
        // XXX: cannot `debug_assert!` a `Self::try_from` here.
        // `try_from` relies on `CellIndex::vertex` for canonical check,
        // which itself calls `new_unchecked` => infinite recursion, stack overflow.
        Self(NonZeroU64::new(value).expect("valid vertex index"))
    }
}

impl Ord for VertexIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        // Bitmask to hide the resolution and vertex.
        const MASK: u64 = 0xf80f_ffff_ffff_ffff;

        // Order by index first, then by vertex.
        (self.0.get() & MASK, self.vertex())
            .cmp(&(other.0.get() & MASK, other.vertex()))
    }
}

impl PartialOrd for VertexIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<VertexIndex> for u64 {
    fn from(value: VertexIndex) -> Self {
        value.0.get()
    }
}

impl fmt::Debug for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{:015o}_{} ({})",
            self.owner().base_cell(),
            u64::from(*self) & bits::DIRECTIONS_MASK,
            self.vertex(),
            self
        )
    }
}

impl fmt::Display for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::Binary for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::Octal for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VertexIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

#[cfg(test)]
#[path = "./vertex_tests.rs"]
mod tests;
