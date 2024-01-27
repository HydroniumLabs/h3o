use super::HexGridError;
use core::fmt;

/// Errors occurring during [`LocalIJ`](crate::LocalIJ) coordinate system
/// conversions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalIjError {
    /// Local origin and cell index have incompatible resolutions.
    ResolutionMismatch,
    /// Pentagon distortion was encountered and could not be handled.
    Pentagon,
    /// Error related to the `IJK` coordinate system.
    HexGrid(HexGridError),
}

impl fmt::Display for LocalIjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ResolutionMismatch => {
                write!(f, "resolution mismatch")
            }
            Self::Pentagon => write!(f, "pentagon distortion"),
            Self::HexGrid(err) => write!(f, "hex grid error: {err}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LocalIjError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ResolutionMismatch | Self::Pentagon => None,
            Self::HexGrid(ref err) => Some(err),
        }
    }
}

impl From<HexGridError> for LocalIjError {
    fn from(value: HexGridError) -> Self {
        Self::HexGrid(value)
    }
}
