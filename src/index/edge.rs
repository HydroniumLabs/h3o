use super::{bits, IndexMode};
use crate::{
    coord::FaceIJK, error, grid, Boundary, CellIndex, Direction,
    EARTH_RADIUS_KM,
};
use std::{cmp::Ordering, fmt, num::NonZeroU64, str::FromStr};

/// Minimum value for a cell edge.
const MIN: u8 = 1;

/// Maximum value for a cell edge.
const MAX: u8 = 6;

// -----------------------------------------------------------------------------

/// Edge of an H3 cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edge(u8);

impl Edge {
    /// Iterates over the valid directions.
    ///
    /// # Example
    ///
    /// ```
    /// let edges = h3o::Edge::iter().collect::<Vec<_>>();
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        // SAFETY: values from 0 to MAX are valid directions.
        (MIN..=MAX).map(Self::new_unchecked)
    }

    /// Initializes a new cell edge using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid cell edge.
    pub(crate) fn new_unchecked(value: u8) -> Self {
        debug_assert!((MIN..=MAX).contains(&value), "cell edge out of range");
        Self(value)
    }
}

impl TryFrom<u8> for Edge {
    type Error = error::InvalidEdge;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(MIN..=MAX).contains(&value) {
            return Err(Self::Error::new(value, "out of range"));
        }
        Ok(Self(value))
    }
}

impl From<Edge> for u8 {
    fn from(value: Edge) -> Self {
        value.0
    }
}

impl From<Edge> for u64 {
    fn from(value: Edge) -> Self {
        Self::from(value.0)
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------

/// Represents a single directed edge between two cells (an "origin" cell and a
/// neighboring "destination" cell).
///
/// The index is encoded on 64-bit with the following bit layout:
///
/// ```text
///  ┏━┳━━━┳━━━┳━━━━━━━━━━━━━━━━━━━━━━┈┈┈┈┈┈┈┈━━━━━━━┓
///  ┃U┃ M ┃ E ┃                O                    ┃
///  ┗━┻━━━┻━━━┻━━━━━━━━━━━━━━━━━━━━━━┈┈┈┈┈┈┈┈━━━━━━━┛
/// 64 63 59   56                                    0
/// ```
///
/// Where:
/// - `U` is an unused reserved bit, always set to 0 (bit 63).
/// - `M` is the index mode, always set to 2, coded on 4 bits (59-62).
/// - `E` is the edge of the origin cell, in [1; 6], coded on 3 bits (56-58).
/// - `O` is the origin cell index, coded on 56 bits (0-55).
///
/// References:
/// - [H3 Index Representations](https://h3geo.org/docs/core-library/h3Indexing)
/// - [H3 Index Bit Layout](https://observablehq.com/@nrabinowitz/h3-index-bit-layout?collection=@nrabinowitz/h3)
/// - [H3 Index Inspector](https://observablehq.com/@nrabinowitz/h3-index-inspector?collection=@nrabinowitz/h3)
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectedEdgeIndex(NonZeroU64);

impl DirectedEdgeIndex {
    /// Returns the cell edge.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// assert_eq!(index.edge(), h3o::Edge::try_from(3)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn edge(self) -> Edge {
        // SAFETY: `EdgeIndex` only contains valid cell edge (invariant).
        Edge::new_unchecked(bits::get_edge(self.0.get()))
    }

    /// Returns the origin hexagon from the directed edge index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// assert_eq!(index.origin(), h3o::CellIndex::try_from(0x8a194e699ab7fff)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn origin(self) -> CellIndex {
        let bits = bits::set_mode(self.0.get(), IndexMode::Cell);
        CellIndex::new_unchecked(bits::clr_edge(bits))
    }

    /// Returns the destination hexagon from the directed edge index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a1_94e6_99ab_7fff)?;
    /// assert_eq!(index.destination(), h3o::CellIndex::try_from(0x8a194e699a97fff)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn destination(self) -> CellIndex {
        let direction = Direction::from(self.edge());
        let origin = self.origin();
        // Every edge has a destination.
        grid::neighbor_rotations(origin, direction, 0)
            .expect("destination cell index")
            .0
    }

    /// Returns the `(origin, destination)` pair of cell index for this edge.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a1_94e6_99ab_7fff)?;
    /// assert_eq!(index.cells(), (
    ///     h3o::CellIndex::try_from(0x8a194e699ab7fff)?,
    ///     h3o::CellIndex::try_from(0x8a194e699a97fff)?,
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn cells(self) -> (CellIndex, CellIndex) {
        (self.origin(), self.destination())
    }

    /// Returns the coordinates defining the directed edge.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// let boundary = index.boundary();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn boundary(self) -> Boundary {
        // Get the origin and neighbor direction from the edge.
        let direction = Direction::from(self.edge());
        let origin = self.origin();

        // Get the start vertex for the edge.
        let start_vertex = direction.vertex(origin);

        // Get the geo boundary for the appropriate vertexes of the origin. Note
        // that while there are always 2 topological vertexes per edge, the
        // resulting edge boundary may have an additional distortion vertex if
        // it crosses an edge of the icosahedron.
        let fijk = FaceIJK::from(origin);
        let resolution = origin.resolution();
        if origin.is_pentagon() {
            fijk.pentagon_boundary(resolution, start_vertex, 2)
        } else {
            fijk.hexagon_boundary(resolution, start_vertex, 2)
        }
    }

    /// Computes the length of this directed edge, in radians.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// assert_eq!(index.length_rads(), 1.1795418098325597e-5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn length_rads(self) -> f64 {
        let boundary = self.boundary();

        (0..boundary.len() - 1)
            .map(|i| boundary[i].distance_rads(boundary[i + 1]))
            .sum()
    }

    /// Computes the length of this directed edge, in kilometers.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// assert_eq!(index.length_km(), 0.07514869340636812);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn length_km(self) -> f64 {
        self.length_rads() * EARTH_RADIUS_KM
    }

    /// Computes the length of this directed edge, in meters.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// assert_eq!(index.length_m(), 75.14869340636812);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn length_m(self) -> f64 {
        self.length_km() * 1000.
    }

    /// Initializes a new edge index a value that may be invalid.
    ///
    /// # Safety
    ///
    /// The value must be a valid edge index.
    pub(crate) fn new_unchecked(value: u64) -> Self {
        debug_assert!(Self::try_from(value).is_ok(), "invalid edge index");
        Self(NonZeroU64::new(value).expect("valid edge index"))
    }
}

impl Ord for DirectedEdgeIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        /// Bitmask to hide the resolution and edge.
        const MASK: u64 = 0xf80f_ffff_ffff_ffff;

        // Order by index first, then by edge.
        (self.0.get() & MASK, self.edge())
            .cmp(&(other.0.get() & MASK, other.edge()))
    }
}

impl PartialOrd for DirectedEdgeIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<DirectedEdgeIndex> for u64 {
    fn from(value: DirectedEdgeIndex) -> Self {
        value.0.get()
    }
}

impl TryFrom<u64> for DirectedEdgeIndex {
    type Error = error::InvalidDirectedEdgeIndex;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if bits::get_mode(value) != u8::from(IndexMode::DirectedEdge) {
            return Err(Self::Error::new(Some(value), "invalid index mode"));
        }

        // Clear the highest byte and validate the index part.
        let bits = bits::set_mode(value, IndexMode::Cell);
        let bits = bits::clr_edge(bits);
        CellIndex::try_from(bits)
            .map_err(|err| Self::Error::new(Some(value), err.reason))?;

        // An hexagon has 6 edges (1-6), while a pentagon only has 5 (2-6).
        let min_edge =
            1 + u8::from(CellIndex::new_unchecked(bits).is_pentagon());
        if !(min_edge..=MAX).contains(&bits::get_edge(value)) {
            return Err(Self::Error::new(Some(value), "invalid cell edge"));
        }

        // XXX: 0 is rejected by the mode check (mode cannot be 0).
        Ok(Self(NonZeroU64::new(value).expect("non-zero edge index")))
    }
}

impl FromStr for DirectedEdgeIndex {
    type Err = error::InvalidDirectedEdgeIndex;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16)
            .map_err(|_| Self::Err {
                value: None,
                reason: "invalid 64-bit hex number",
            })
            .and_then(Self::try_from)
    }
}

impl fmt::Debug for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{:015o}_{} ({})",
            self.origin().base_cell(),
            u64::from(*self) & bits::DIRECTIONS_MASK,
            self.edge(),
            self
        )
    }
}

impl fmt::Display for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:x}")
    }
}

impl fmt::Binary for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::Octal for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for DirectedEdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for DirectedEdgeIndex {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        u64::arbitrary(data).and_then(|byte| {
            Self::try_from(byte).map_err(|_| arbitrary::Error::IncorrectFormat)
        })
    }
}

#[cfg(test)]
#[path = "./edge_tests.rs"]
mod tests;
