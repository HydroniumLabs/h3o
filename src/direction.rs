use crate::{error, Edge};
use std::fmt;

/// Maximum value for a direction.
const MAX: u8 = 6;

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

    /// Initializes a new [`Direction`] using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid direction.
    #[allow(unsafe_code)] // TODO: bench if this is needed!
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        debug_assert!(value <= MAX, "direction out of range");
        // SAFETY: range checked above.
        unsafe { std::mem::transmute::<u8, Self>(value) }
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
