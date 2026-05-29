mod arc_set;

use arc_set::ArcSet;

/// A solvent that dissolves a set of H3 cell indexes into a `MultiPolygon`
/// representing the outlines of the set.
#[derive(Debug, Clone, Copy)]
pub struct Solvent {
    input_mode: InputMode,
}

impl Solvent {
    /// Creates a [`MultiPolygon`][geo::MultiPolygon] describing the outline(s)
    /// of a set of cells.
    ///
    /// # Errors
    ///
    /// All cell indexes must be unique and have the expected resolution,
    /// otherwise [`DissolutionError`][crate::error::DissolutionError] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{geom::SolventBuilder, CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x089283470803ffff)?;
    /// let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();
    /// let solvent = SolventBuilder::new().build();
    /// let geom = solvent.dissolve(cells)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn dissolve(
        &self,
        cells: impl IntoIterator<Item = crate::CellIndex>,
    ) -> Result<geo::MultiPolygon, crate::error::DissolutionError> {
        Ok(match self.input_mode {
            InputMode::Homogeneous => ArcSet::new(cells)?.into(),
            InputMode::Heterogeneous(resolution) => {
                ArcSet::new_heterogeneous(cells, resolution)?.into()
            }
        })
    }
}

// -----------------------------------------------------------------------------

/// A builder to configure a solvent.
#[derive(Debug, Clone, Copy)]
pub struct SolventBuilder {
    input_mode: InputMode,
}

impl Default for SolventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SolventBuilder {
    /// Initializes a new plotter builder with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input_mode: InputMode::Homogeneous,
        }
    }

    /// Enable support for heterogeneous (e.g. compacted) cell set.
    ///
    /// This mode is way faster and use less memory for large & dense set of
    /// cells. It may be slower for small or very sparse sets.
    ///
    /// When enabling this mode you must specify the finest resolution you want
    /// to support. Coarser cells will be converted on-the-fly and finer cells
    /// will trigger an error.
    #[must_use]
    pub const fn enable_heterogeneous_support(
        mut self,
        resolution: crate::Resolution,
    ) -> Self {
        self.input_mode = InputMode::Heterogeneous(resolution);
        self
    }

    /// Builds the plotter.
    #[must_use]
    pub const fn build(self) -> Solvent {
        Solvent {
            input_mode: self.input_mode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InputMode {
    /// An homogeneous set of cells.
    Homogeneous,
    /// An heterogeneous set of cells (e.g. compacted) with a max resolution.
    Heterogeneous(crate::Resolution),
}
