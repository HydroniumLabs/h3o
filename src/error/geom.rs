use crate::error::LocalIjError;
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

/// Errors from a plotter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlotterError {
    /// Invalid input geometry.
    InvalidGeometry(InvalidGeometry),
    /// Coordinate system conversion error.
    LocalIj(LocalIjError),
}

impl fmt::Display for PlotterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidGeometry(err) => write!(f, "plotting error: {err}"),
            Self::LocalIj(err) => write!(f, "plotting error: {err}"),
        }
    }
}

impl Error for PlotterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::InvalidGeometry(ref err) => Some(err),
            Self::LocalIj(ref err) => Some(err),
        }
    }
}

impl From<InvalidGeometry> for PlotterError {
    fn from(value: InvalidGeometry) -> Self {
        Self::InvalidGeometry(value)
    }
}

impl From<LocalIjError> for PlotterError {
    fn from(value: LocalIjError) -> Self {
        Self::LocalIj(value)
    }
}

// -----------------------------------------------------------------------------

/// Errors occurring during the dissolution of a set of cell indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DissolutionError {
    /// Input contains cell indexes of unsupported resolutions.
    UnsupportedResolution,
    /// Input set contains duplicate cell indexes.
    DuplicateInput,
}

impl fmt::Display for DissolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnsupportedResolution => {
                write!(f, "unsupported resolution")
            }
            Self::DuplicateInput => write!(f, "duplicate indices"),
        }
    }
}

impl Error for DissolutionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
