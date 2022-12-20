use std::fmt;

/// H3 index modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
#[non_exhaustive]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
pub enum IndexMode {
    /// An H3 Cell (Hexagon/Pentagon) index.
    Cell = 1,
    /// An H3 directed edge (Cell A -> Cell B) index.
    DirectedEdge = 2,
    /// An H3 undirected edge (Cell A <-> Cell B) index.
    UndirectedEdge = 3,
    /// An H3 Vertex (i.e. a single vertex of an H3 Cell).
    Vertex = 4,
}

impl From<IndexMode> for u8 {
    fn from(value: IndexMode) -> Self {
        value as Self
    }
}

impl fmt::Display for IndexMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Cell => "Cell",
                Self::DirectedEdge => "DirectedEdge",
                Self::UndirectedEdge => "UndirectedEdge",
                Self::Vertex => "Vertex",
            }
        )
    }
}
