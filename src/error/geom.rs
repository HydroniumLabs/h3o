use core::{error::Error, fmt};

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

// -----------------------------------------------------------------------------

/// Errors occurring during the outline computation of a set of cell indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutlinerError {
    /// Input contains indices of heterogeneous resolutions.
    HeterogeneousResolution,
    /// Input set contains duplicate indices.
    DuplicateInput,
}

impl fmt::Display for OutlinerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::HeterogeneousResolution => {
                write!(f, "heterogeneous resolution")
            }
            Self::DuplicateInput => write!(f, "duplicate indices"),
        }
    }
}

impl Error for OutlinerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
