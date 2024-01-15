use super::line;
use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex,
};
use std::boxed::Box;

/// An ordered collection of two or more [`geo::Coord`]s, representing a
/// path between locations.
///
/// Note that the `ToCells` implementation suffers from the same limitation
/// that [`grid_path_cells`](CellIndex::grid_path_cells), which means that on
/// error `max_cells_count` returns 0 and `to_cells` an empty iterator.
#[derive(Clone, Debug, PartialEq)]
pub struct LineString(geo::LineString<f64>);

impl LineString {
    /// Initialize a new line from a line whose coordinates are in radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the line is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::LineString;
    ///
    /// let line_string = geo::LineString::new(vec![
    ///     geo::coord! { x: -0.009526982062241713, y: 0.8285232894553574 },
    ///     geo::coord! { x: 0.04142734140306332, y: 0.8525145186317127 },
    /// ]);
    /// let line = LineString::from_radians(line_string)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        line: geo::LineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Self::check_coords(&line).map(|()| Self(line))
    }

    /// Initialize a new line from a line whose coordinates are in degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the line is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::LineString;
    ///
    /// let line_string = geo::LineString::new(vec![
    ///     geo::coord! { x: -0.5458558636632915, y: 47.47088771408784 },
    ///     geo::coord! { x: 2.373611818843102,   y: 48.84548389122412 },
    /// ]);
    /// let line = LineString::from_degrees(line_string)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        mut line: geo::LineString<f64>,
    ) -> Result<Self, InvalidGeometry> {
        for coord in line.coords_mut() {
            coord.x = coord.x.to_radians();
            coord.y = coord.y.to_radians();
        }
        Self::check_coords(&line).map(|()| Self(line))
    }

    // Check that the line's coordinates are finite.
    fn check_coords(
        line: &geo::LineString<f64>,
    ) -> Result<(), InvalidGeometry> {
        if !line.coords().all(|coord| super::coord_is_valid(*coord)) {
            return Err(InvalidGeometry::new(
                "every coordinate of the line must be valid",
            ));
        }
        Ok(())
    }
}

impl From<LineString> for geo::LineString<f64> {
    fn from(value: LineString) -> Self {
        value.0
    }
}

impl ToCells for LineString {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        self.0
            .lines()
            .map(|line| line::cells_count(line, config.resolution))
            .sum()
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(
            self.0
                .lines()
                .flat_map(move |line| line::to_cells(line, config.resolution)),
        )
    }
}
