use std::{error::Error, fmt};

/// Errors related to the `IJK` coordinate system and its variants (e.g.
/// [`LocalIJ`](crate::LocalIJ)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HexGridError {
    reason: &'static str,
}

impl HexGridError {
    /// Initializes a new [`HexGridError`] with the given error message.
    pub(crate) const fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

impl fmt::Display for HexGridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for HexGridError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
