use std::{error::Error, fmt};

// Macro to declare type-specific InvalidValue error type.
macro_rules! invalid_value_error {
    ($name:literal, $error:ident, $value_type:ty) => {
        #[doc = concat!("Invalid ", $name, ".")]
        #[derive(Clone, Copy, Debug, PartialEq)]
        // Value type may not be `Eq` (e.g. f64).
        #[allow(clippy::derive_partial_eq_without_eq)]
        pub struct $error {
            /// The invalid value.
            pub value: $value_type,
            /// The reason why it's invalid.
            pub reason: &'static str,
        }

        impl $error {
            pub(crate) const fn new(
                value: $value_type,
                reason: &'static str,
            ) -> Self {
                Self { value, reason }
            }
        }

        impl fmt::Display for $error {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "invalid {} (got {:?}): {}",
                    $name, self.value, self.reason
                )
            }
        }

        impl Error for $error {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }
        }
    };
}

invalid_value_error!("resolution", InvalidResolution, Option<u8>);
invalid_value_error!("cell index", InvalidCellIndex, Option<u64>);
invalid_value_error!("vertex index", InvalidVertexIndex, Option<u64>);
invalid_value_error!(
    "directed edge index",
    InvalidDirectedEdgeIndex,
    Option<u64>
);
invalid_value_error!("latitude/longitude", InvalidLatLng, f64);
invalid_value_error!("cell edge", InvalidEdge, u8);
invalid_value_error!("cell vertex", InvalidVertex, u8);
invalid_value_error!("icosahedron face", InvalidFace, u8);
invalid_value_error!("base cell", InvalidBaseCell, u8);
invalid_value_error!("direction", InvalidDirection, u8);
