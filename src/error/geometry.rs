use std::{error::Error, fmt};

/// Errors related to the geometries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidGeometry {
    reason: &'static str,
}

impl InvalidGeometry {
    /// Initializes a new [`InvalidGeometry`] with the given error message.
    pub(crate) const fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

impl fmt::Display for InvalidGeometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl Error for InvalidGeometry {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
