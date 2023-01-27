use crate::{
    coord::CoordIJK, error, CellIndex, Edge, Vertex, NUM_HEX_VERTS,
    NUM_PENT_VERTS,
};
use std::{fmt, num::NonZeroU8};

/// Maximum value for a direction.
const MAX: u8 = 6;

/// Hexagon direction to vertex relationships (same face).
const TO_VERTEX_HEXAGON: [Vertex; NUM_HEX_VERTS as usize] = [
    Vertex::new_unchecked(3),
    Vertex::new_unchecked(1),
    Vertex::new_unchecked(2),
    Vertex::new_unchecked(5),
    Vertex::new_unchecked(4),
    Vertex::new_unchecked(0),
];

/// Pentagon direction to vertex relationships (same face).
const TO_VERTEX_PENTAGON: [Vertex; NUM_PENT_VERTS as usize] = [
    Vertex::new_unchecked(1),
    Vertex::new_unchecked(2),
    Vertex::new_unchecked(4),
    Vertex::new_unchecked(3),
    Vertex::new_unchecked(0),
];

// -----------------------------------------------------------------------------

/// A direction within an hexagonal grid.
///
/// In H3, each hexagonal cell at level `N-1` is divided into 7 cells at the
/// level `N`, with each sub-cell in one of the 7 possible directions (6 axes +
/// the center).
///
/// ```text
///              J axis
///             ___
///            /   \
///        +--+  2  +--+
///       / 3  \___/  6 \
///       \    /   \    /
///        +--+  0  +--+
///       /    \___/    \
///       \ 1  /   \  4 /
///      K +--+  5  +--+ I
///     axis   \___/    axis
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
#[allow(clippy::exhaustive_enums)] // Not gonna change any time soon.
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
pub enum Direction {
    /// Center.
    Center = 0,
    /// K axe.
    K = 1,
    /// J axe.
    J = 2,
    /// JK axe.
    JK = 3,
    /// I axe.
    I = 4,
    /// IK axe.
    IK = 5,
    /// IJ axe.
    IJ = 6,
}

impl Direction {
    /// Iterates over the valid directions.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::Direction;
    ///
    /// let directions = Direction::iter().collect::<Vec<_>>();
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        // SAFETY: values from 0 to MAX are valid directions.
        (0..=MAX).map(Self::new_unchecked)
    }

    /// Returns the IJK coordinate of the direction.
    pub(crate) fn coordinate(self) -> CoordIJK {
        let value = u8::from(self);

        CoordIJK::new(
            i32::from((value >> 2) & 1),
            i32::from((value >> 1) & 1),
            i32::from(value & 1),
        )
    }

    /// Returns the axe numerical value, if any.
    pub(crate) fn axe(self) -> Option<NonZeroU8> {
        NonZeroU8::new(self.into())
    }

    /// Initializes a new [`Direction`] using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid direction.
    #[allow(unsafe_code)]
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        assert!(value <= MAX, "direction out of range");
        // SAFETY: range checked above.
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }

    /// Returns a direction rotated `count` time, by 60 degrees step.
    #[rustfmt::skip]
    pub(crate) const fn rotate60<const CCW: bool>(self, count: usize) -> Self {
        use Direction::{Center, I, IJ, IK, J, JK, K};

        const CCW_SEQUENCE: [Direction; 6] = [K, IK, I, IJ, J, JK];
        const CW_SEQUENCE:  [Direction; 6] = [K, JK, J, IJ, I, IK];

        // Returns self if there is no rotation.
        if count == 0 {
            return self;
        }

        let offset = match self {
            // The center is not affected by any rotations.
            Center => return self,
            K  => 0,
            J  => if CCW { 4 } else { 2 },
            JK => if CCW { 5 } else { 1 },
            I  => if CCW { 2 } else { 4 },
            IK => if CCW { 1 } else { 5 },
            IJ => 3,
        };
        let index = (count + offset) % 6;

        if CCW { CCW_SEQUENCE[index] } else { CW_SEQUENCE[index] }
    }

    /// Returns a direction rotated once, by 60 degrees step.
    #[rustfmt::skip]
    pub(crate) const fn rotate60_once<const CCW: bool>(self) -> Self {
        use Direction::{Center, I, IJ, IK, J, JK, K};

        // XXX: Lookup table approach is ~20% slower than explicit match
        // statement.
        match self {
            Center => Center,
            K  => if CCW { IK } else { JK },
            J  => if CCW { JK } else { IJ },
            JK => if CCW { K }  else { J },
            I  => if CCW { IJ } else { IK },
            IK => if CCW { I }  else { K },
            IJ => if CCW { J }  else { I },
        }
    }

    /// Returns the first topological vertex.
    ///
    /// Get the first vertex for this direction. The neighbor in this
    /// direction is located between this vertex and the next in sequence.
    pub(crate) fn vertex(self, origin: CellIndex) -> Vertex {
        let is_pentagon = origin.is_pentagon();

        // Check for invalid directions: center and deleted K axis (pentagon
        // only).
        assert!(self != Self::Center && !(is_pentagon && self == Self::K));

        // Determine the vertex rotations for this cell.
        let rotations = origin.vertex_rotations();

        // Find the appropriate vertex, rotating CCW if necessary.
        Vertex::new_unchecked(if is_pentagon {
            // -2 because we don't use directions 0 (center) or 1 (deleted K
            // axis).
            let index = usize::from(self) - 2;
            (u8::from(TO_VERTEX_PENTAGON[index]) + NUM_PENT_VERTS - rotations)
                % NUM_PENT_VERTS
        } else {
            // -1 because we don't use direction 0 (center).
            let index = usize::from(self) - 1;
            (u8::from(TO_VERTEX_HEXAGON[index]) + NUM_HEX_VERTS - rotations)
                % NUM_HEX_VERTS
        })
    }
}

impl TryFrom<u8> for Direction {
    type Error = error::InvalidDirection;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Center),
            1 => Ok(Self::K),
            2 => Ok(Self::J),
            3 => Ok(Self::JK),
            4 => Ok(Self::I),
            5 => Ok(Self::IK),
            6 => Ok(Self::IJ),
            _ => Err(Self::Error::new(value, "out of range")),
        }
    }
}

impl From<Direction> for u8 {
    fn from(value: Direction) -> Self {
        value as Self
    }
}

impl From<Direction> for u64 {
    fn from(value: Direction) -> Self {
        u8::from(value).into()
    }
}

impl From<Direction> for usize {
    fn from(value: Direction) -> Self {
        u8::from(value).into()
    }
}

impl From<Edge> for Direction {
    fn from(value: Edge) -> Self {
        // SAFETY: Edge are numbered from 1 to 6, according to which direction
        // they face.
        Self::new_unchecked(value.into())
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", u8::from(*self))
    }
}
