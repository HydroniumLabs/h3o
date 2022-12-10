use crate::error;
use std::fmt;

/// Maximum value for a base cell.
pub const MAX: u8 = 121;

// Bitmap where a bit's position represents a base cell value.
const BASE_PENTAGONS: u128 = 0x0020_0802_0008_0100_8402_0040_0100_4010;

// -----------------------------------------------------------------------------

/// One of the 122 base cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct BaseCell(u8);

impl BaseCell {
    /// Initializes a new base cell using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid base cell.
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        debug_assert!(value <= MAX, "base cell out of range");
        Self(value)
    }

    /// Returns true if the base cell is pentagonal.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// assert!(BaseCell::try_from(4)?.is_pentagon());
    /// assert!(!BaseCell::try_from(8)?.is_pentagon());
    /// # Ok::<(), h3o::error::InvalidBaseCell>(())
    /// ```
    #[must_use]
    pub const fn is_pentagon(self) -> bool {
        BASE_PENTAGONS & (1 << self.0) != 0
    }

    /// Returns the total number of base cells.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// assert_eq!(BaseCell::count(), 122);
    /// ```
    #[must_use]
    pub const fn count() -> u8 {
        MAX + 1
    }

    /// Returns all the base cell.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// let cells = BaseCell::iter().collect::<Vec<_>>();
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..Self::count()).map(Self::new_unchecked)
    }
}

impl TryFrom<u8> for BaseCell {
    type Error = error::InvalidBaseCell;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX {
            return Err(Self::Error::new(value, "out of range"));
        }
        Ok(Self(value))
    }
}

impl From<BaseCell> for u8 {
    fn from(value: BaseCell) -> Self {
        value.0
    }
}

impl From<BaseCell> for usize {
    fn from(value: BaseCell) -> Self {
        Self::from(value.0)
    }
}

impl fmt::Display for BaseCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
