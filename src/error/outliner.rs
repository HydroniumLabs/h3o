use core::fmt;

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

#[cfg(feature = "std")]
impl std::error::Error for OutlinerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
