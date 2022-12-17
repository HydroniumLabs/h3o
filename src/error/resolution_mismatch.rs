use std::{error::Error, fmt};

/// Resolution mismatch between two cell indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolutionMismatch;

impl fmt::Display for ResolutionMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolution mismatch")
    }
}

impl Error for ResolutionMismatch {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
