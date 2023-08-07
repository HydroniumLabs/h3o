use super::LineString;
use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex,
};
use std::boxed::Box;

/// A collection of [`geo::LineString`].
///
/// Note that the `ToCells` implementation suffers from the same limitation
/// that [`grid_path_cells`](CellIndex::grid_path_cells), which means that on
/// error `max_cells_count` returns 0 and `to_cells` an empty iterator.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiLineString(Vec<LineString>);

impl MultiLineString {
    /// Initialize a collection of lines from lines whose coordinates are in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the line is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::line_string;
    /// use h3o::geom::MultiLineString;
    ///
    /// let line_string: geo::LineString<f64> = line_string![
    ///     (x: 1.996408325715777, y: 0.534292570530397),
    ///     (x: 2.208424012168513, y: 0.7995167582816788),
    ///     (x: 2.1213562369319434, y: 0.5449632604075227),
    /// ];
    /// let lines = geo::MultiLineString::new(vec![line_string]);
    /// let lines = MultiLineString::from_radians(lines)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        lines: geo::MultiLineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            lines
                .into_iter()
                .map(LineString::from_radians)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    /// Initialize a collection of lines from lines whose coordinates are in
    /// degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the line is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::line_string;
    /// use h3o::geom::MultiLineString;
    ///
    /// let line_string: geo::LineString<f64> = line_string![
    ///     (x: 114.385771248293,   y: 30.612709316587612),
    ///     (x: 126.53337527260373, y: 45.8089358995214),
    ///     (x: 121.54475921995464, y: 31.22409481103989),
    /// ];
    /// let lines = geo::MultiLineString::new(vec![line_string]);
    /// let lines = MultiLineString::from_degrees(lines)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        lines: geo::MultiLineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            lines
                .into_iter()
                .map(LineString::from_degrees)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<MultiLineString> for geo::MultiLineString<f64> {
    fn from(value: MultiLineString) -> Self {
        Self(value.0.into_iter().map(Into::into).collect())
    }
}

impl ToCells for MultiLineString {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        self.0.iter().map(|line| line.max_cells_count(config)).sum()
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(self.0.iter().flat_map(move |line| line.to_cells(config)))
    }
}
