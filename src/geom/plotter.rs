use crate::{
    error::{InvalidGeometry, PlotterError},
    index::GridPathCells,
    CellIndex, LatLng, Resolution,
};
use geo::Line;

/// A plotter that produces H3 cell indexes along given lines.
///
/// ```rust
/// use geo::line_string;
/// use h3o::{geom::PlotterBuilder, Resolution};
///
/// let mut plotter = PlotterBuilder::new(Resolution::Ten).build();
/// plotter.add_batch(line_string![
///     (x: 2.363503198417334,  y: 48.8203086545891),
///     (x: 2.3730684893043588, y: 48.85398407690437),
///     (x: 2.334964762310932,  y: 48.870861968772914),
/// ].lines())?;
///
/// let cells = plotter.plot().collect::<Result<Vec<_>, _>>()?;
///
/// # Ok::<(), h3o::error::PlotterError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Plotter {
    resolution: Resolution,
    convert_to_rads: bool,
    paths: Vec<GridPathCells>,
}

impl Plotter {
    /// Adds a `Line` to plot.
    ///
    /// # Errors
    ///
    /// [`PlotterError`] if the line is invalid or cannot be handled (cf.
    /// [`grid_path_cells`](CellIndex::grid_path_cells) limitations).
    pub fn add(&mut self, mut line: Line) -> Result<(), PlotterError> {
        if self.convert_to_rads {
            line.start.x = line.start.x.to_radians();
            line.start.y = line.start.y.to_radians();
            line.end.x = line.end.x.to_radians();
            line.end.y = line.end.y.to_radians();
        }
        Self::check_coords(&line)?;

        // Expect valid coordinates, checked by `check_coords` above.
        let start = LatLng::from_radians(line.start.y, line.start.x)
            .expect("valid start")
            .to_cell(self.resolution);
        let end = LatLng::from_radians(line.end.y, line.end.x)
            .expect("valid end")
            .to_cell(self.resolution);

        self.paths.push(GridPathCells::new(start, end)?);

        Ok(())
    }

    /// Adds a batch of `Line` to plot.
    ///
    /// # Errors
    ///
    /// [`PlotterError`] if the line is invalid or cannot be handled (cf.
    /// [`grid_path_cells`](CellIndex::grid_path_cells) limitations).
    pub fn add_batch(
        &mut self,
        lines: impl IntoIterator<Item = Line>,
    ) -> Result<(), PlotterError> {
        for line in lines {
            self.add(line)?;
        }

        Ok(())
    }

    /// Plot the hexagons along the lines.
    ///
    /// Note that this functions suffers from the same limitation as
    /// [`grid_path_cells`](CellIndex::grid_path_cells).
    pub fn plot(self) -> impl Iterator<Item = Result<CellIndex, PlotterError>> {
        self.paths
            .into_iter()
            .flatten()
            .map(|res| res.map_err(Into::into))
    }

    // Check that the line's coordinates are valid.
    fn check_coords(line: &Line) -> Result<(), PlotterError> {
        if !super::coord_is_valid(line.start)
            || !super::coord_is_valid(line.end)
        {
            return Err(InvalidGeometry::new(
                "every coordinate of the line must be valid",
            )
            .into());
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------

/// A builder to configure a plotter.
#[derive(Debug, Clone, Copy)]
pub struct PlotterBuilder {
    resolution: Resolution,
    convert_to_rads: bool,
}

impl PlotterBuilder {
    /// Initializes a new plotter builder with default settings.
    #[must_use]
    pub const fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            convert_to_rads: true,
        }
    }

    /// Disable the degress-to-radians conversion pre-processing.
    #[must_use]
    pub const fn disable_radians_conversion(mut self) -> Self {
        self.convert_to_rads = false;
        self
    }

    /// Builds the plotter.
    #[must_use]
    pub const fn build(self) -> Plotter {
        Plotter {
            resolution: self.resolution,
            convert_to_rads: self.convert_to_rads,
            paths: Vec::new(),
        }
    }
}
