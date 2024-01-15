use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex, LatLng, Resolution,
};
use geo::Coord;
use std::boxed::Box;

/// A line segment made up of exactly two [`geo::Coord`]s.
///
/// Note that the `ToCells` implementation suffers from the same limitation
/// that [`grid_path_cells`](CellIndex::grid_path_cells), which means that on
/// error `max_cells_count` returns 0 and `to_cells` an empty iterator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line(geo::Line<f64>);

impl Line {
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
    /// use h3o::geom::Line;
    ///
    /// let line = geo::Line::new(
    ///     geo::coord! { x: 0.05470401801197459, y: 0.8005260881667454 },
    ///     geo::coord! { x: 0.0420053741471695, y: 0.8218402563603641 },
    /// );
    /// let line = Line::from_radians(line)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(line: geo::Line<f64>) -> Result<Self, InvalidGeometry> {
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
    /// use h3o::geom::Line;
    ///
    /// let line = geo::Line::new(
    ///     geo::coord! { x: 3.13430935449378,  y: 45.866766242072146 },
    ///     geo::coord! { x: 2.406730655500752, y: 47.08797812339847 },
    /// );
    /// let line = Line::from_degrees(line)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(line: geo::Line<f64>) -> Result<Self, InvalidGeometry> {
        let line = geo::Line::new(
            Coord {
                x: line.start.x.to_radians(),
                y: line.start.y.to_radians(),
            },
            Coord {
                x: line.end.x.to_radians(),
                y: line.end.y.to_radians(),
            },
        );
        Self::from_radians(line)
    }

    // Check that the line's coordinates are finite.
    fn check_coords(line: &geo::Line<f64>) -> Result<(), InvalidGeometry> {
        if !super::coord_is_valid(line.start)
            || !super::coord_is_valid(line.end)
        {
            return Err(InvalidGeometry::new("start and end must be valid"));
        }
        Ok(())
    }
}

impl From<Line> for geo::Line<f64> {
    fn from(value: Line) -> Self {
        value.0
    }
}

impl ToCells for Line {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        cells_count(self.0, config.resolution)
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(to_cells(self.0, config.resolution))
    }
}

// ----------------------------------------------------------------------------

pub fn cells_count(line: geo::Line<f64>, resolution: Resolution) -> usize {
    let (start, end) = start_end_cells(&line, resolution);
    // Ideally, this should return an error but see comments in `to_cells`
    // below.
    usize::try_from(start.grid_path_cells_size(end).unwrap_or_default())
        .expect("positive cells count")
}

pub fn to_cells(
    line: geo::Line<f64>,
    resolution: Resolution,
) -> impl Iterator<Item = CellIndex> {
    let (start, end) = start_end_cells(&line, resolution);

    // TODO: We should avoid `collect` here and returns errors on the fly.
    // BUT this would be make the API less convenient since one would need
    // to handle `Error` for `Geometry` while only `Line` and `LineString`
    // can fail.
    // And for no good reason, since sooner or later we should get an
    // implementation of `grid_disk_path` that correctly handles pentagons and
    // stuff.
    //
    // Until then, let's use the suboptimal collect internally to preserve a
    // nice external API.
    let cells = start.grid_path_cells(end).map_or_else(
        |_| Vec::new(),
        |iter| iter.collect::<Result<Vec<_>, _>>().unwrap_or_default(),
    );

    cells.into_iter()
}

// Returns the cell indexes at the start and end of the line for the given
// resolution.
fn start_end_cells(
    line: &geo::Line<f64>,
    resolution: Resolution,
) -> (CellIndex, CellIndex) {
    // TODO: precompute those at creation time?
    // Expect valid coordinates, checked by `check_coords` before.
    let start = LatLng::from_radians(line.start.y, line.start.x)
        .expect("valid start")
        .to_cell(resolution);
    let end = LatLng::from_radians(line.end.y, line.end.x)
        .expect("valid end")
        .to_cell(resolution);

    (start, end)
}
