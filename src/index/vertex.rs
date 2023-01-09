use super::{bits, IndexMode};
use crate::{
    coord::FaceIJK, error, CellIndex, Direction, LatLng, NUM_HEX_VERTS,
    NUM_PENT_VERTS,
};
use std::{cmp::Ordering, fmt, num::NonZeroU64, str::FromStr};

/// Maximum value for a cell vertex.
const MAX: u8 = 5;

/// Vertex number to hexagon direction relationships (same face).
const TO_DIRECTION_HEXAGON: [Direction; NUM_HEX_VERTS as usize] = [
    Direction::IJ,
    Direction::J,
    Direction::JK,
    Direction::K,
    Direction::IK,
    Direction::I,
];

/// Vertex number to pentagon direction relationships (same face).
const TO_DIRECTION_PENTAGON: [Direction; NUM_PENT_VERTS as usize] = [
    Direction::IJ,
    Direction::J,
    Direction::JK,
    Direction::IK,
    Direction::I,
];

// -----------------------------------------------------------------------------

/// Vertex of an H3 cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub(crate) fn to_direction(self, origin: CellIndex) -> Direction {
        let is_pentagon = origin.is_pentagon();
        let vertex_count = if is_pentagon {
            NUM_PENT_VERTS
        } else {
            NUM_HEX_VERTS
        };

        // Invalid vertex are filtered out by the caller.
        assert!(self.0 < vertex_count);

        // Determine the vertex rotations for this cell.
        let rotations = origin.vertex_rotations();

        // Find the appropriate direction, rotating CW if necessary
        if is_pentagon {
            let index = (self.0 + rotations) % NUM_PENT_VERTS;

            TO_DIRECTION_PENTAGON[usize::from(index)]
        } else {
            let index = (self.0 + rotations) % NUM_HEX_VERTS;

            TO_DIRECTION_HEXAGON[usize::from(index)]
        }
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

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Vertex {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        u8::arbitrary(data).and_then(|byte| {
            Self::try_from(byte).map_err(|_| arbitrary::Error::IncorrectFormat)
        })
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
/// References:
/// - [H3 Index Representations](https://h3geo.org/docs/core-library/h3Indexing)
/// - [H3 Index Bit Layout](https://observablehq.com/@nrabinowitz/h3-index-bit-layout?collection=@nrabinowitz/h3)
/// - [H3 Index Inspector](https://observablehq.com/@nrabinowitz/h3-index-inspector?collection=@nrabinowitz/h3)
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl TryFrom<u64> for VertexIndex {
    type Error = error::InvalidVertexIndex;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if bits::get_mode(value) != u8::from(IndexMode::Vertex) {
            return Err(Self::Error::new(Some(value), "invalid index mode"));
        }

        // Clear the highest byte and validate the owner part.
        let bits = bits::set_mode(value, IndexMode::Cell);
        let bits = bits::clr_vertex(bits);
        let owner = CellIndex::try_from(bits)
            .map_err(|err| Self::Error::new(Some(value), err.reason))?;

        // The easiest way to ensure that the owner + vertex number is valid,
        // and that the vertex is canonical, is to recreate and compare.
        let vertex =
            Vertex::try_from(bits::get_vertex(value)).map_err(|_| {
                Self::Error::new(Some(value), "invalid vertex number")
            })?;
        let canonical = owner.vertex(vertex).map(u64::from);

        if canonical != Some(value) {
            return Err(Self::Error::new(Some(value), "non-canonical vertex"));
        }

        // XXX: 0 is rejected by the mode check (mode cannot be 0).
        Ok(Self(NonZeroU64::new(value).expect("non-zero vertex index")))
    }
}

impl From<VertexIndex> for LatLng {
    // Get the geocoordinates of an H3 vertex.
    fn from(value: VertexIndex) -> Self {
        // SAFETY: VertexIndex always contains a valid vertex value.
        let vertex = Vertex::new_unchecked(bits::get_vertex(value.0.get()));
        let owner = value.owner();

        // Get the single vertex from the boundary.
        let fijk = FaceIJK::from(owner);
        let resolution = owner.resolution();
        let boundary = if owner.is_pentagon() {
            fijk.pentagon_boundary(resolution, vertex, 1)
        } else {
            fijk.hexagon_boundary(resolution, vertex, 1)
        };

        boundary[0]
    }
}

impl FromStr for VertexIndex {
    type Err = error::InvalidVertexIndex;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16)
            .map_err(|_| Self::Err {
                value: None,
                reason: "invalid 64-bit hex number",
            })
            .and_then(Self::try_from)
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
        write!(f, "{self:x}")
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

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for VertexIndex {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        u64::arbitrary(data).and_then(|byte| {
            Self::try_from(byte).map_err(|_| arbitrary::Error::IncorrectFormat)
        })
    }
}

#[cfg(test)]
#[path = "./vertex_tests.rs"]
mod tests;
