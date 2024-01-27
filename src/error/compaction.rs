use core::fmt;

/// Errors occurring while compacting a set of cell indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompactionError {
    /// Input contains indices of heterogeneous resolutions.
    HeterogeneousResolution,
    /// Input set contains duplicate indices.
    DuplicateInput,
}

impl fmt::Display for CompactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::HeterogeneousResolution => {
                write!(f, "heterogeneous resolution")
            }
            Self::DuplicateInput => write!(f, "duplicate indices"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CompactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
