use core::fmt;

/// Resolution mismatch between two cell indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolutionMismatch;

impl fmt::Display for ResolutionMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolution mismatch")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ResolutionMismatch {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
