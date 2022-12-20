use crate::LatLng;
use std::{fmt, ops::Deref};

/// Maximum number of cell boundary vertices.
///
/// Worst case is pentagon: 5 original verts + 5 edge crossings.
const MAX_BNDRY_VERTS: usize = 10;

/// Boundary in latitude/longitude.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct Boundary {
    /// Vertices in CCW order.
    points: [LatLng; MAX_BNDRY_VERTS],
    /// Number of vertices.
    count: u8,
}

impl Boundary {
    /// Initializes a new empty cell boundary (test only)
    #[must_use]
    #[doc(hidden)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertices to the boundary (test only).
    #[doc(hidden)]
    pub fn push(&mut self, ll: LatLng) {
        self.points[usize::from(self.count)] = ll;
        self.count += 1;
    }
}

impl Deref for Boundary {
    type Target = [LatLng];

    /// Dereference to the slice of filled elements.
    fn deref(&self) -> &Self::Target {
        &self.points[..self.count.into()]
    }
}

impl fmt::Display for Boundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("-")
        )
    }
}

#[cfg(feature = "geo")]
impl From<Boundary> for geo::LineString {
    fn from(value: Boundary) -> Self {
        Self::new(value.iter().copied().map(geo::Coord::from).collect())
    }
}
