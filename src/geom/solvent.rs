use super::VertexGraph;
use crate::{error::DissolutionError, CellIndex, Resolution};
use geo::MultiPolygon;

/// A solvent that dissolves a set of H3 cell indexes into a `MultiPolygon`
/// representing the outlines of the set.
#[derive(Debug, Clone, Copy)]
pub struct Solvent {
    input_mode: InputMode,
    check_duplicate: bool,
}

impl Solvent {
    /// Creates a [`MultiPolygon`] describing the outline(s) of a set of cells.
    ///
    /// # Errors
    ///
    /// All cell indexes must be unique and have the expected resolution,
    /// otherwise [`DissolutionError`] is returned.
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
        cells: impl IntoIterator<Item = CellIndex>,
    ) -> Result<MultiPolygon, DissolutionError> {
        let graph = match self.input_mode {
            InputMode::Homogeneous => {
                VertexGraph::from_homogeneous(cells, self.check_duplicate)
            }
            InputMode::Heterogeneous(resolution) => {
                VertexGraph::from_heterogeneous(
                    cells,
                    resolution,
                    self.check_duplicate,
                )
            }
        }?;

        Ok(graph.into())
    }
}

// -----------------------------------------------------------------------------

/// A builder to configure a solvent.
#[derive(Debug, Clone, Copy)]
pub struct SolventBuilder {
    input_mode: InputMode,
    check_duplicate: bool,
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
            check_duplicate: true,
        }
    }

    /// Disable duplicate detection.
    ///
    /// If the input set contains duplicate cells, the resulting geometry will
    /// be incorrect.
    ///
    /// By default the solvent will ensure that the input set doesn't contains
    /// any duplicate but this may be costly (especially for heterogeneous
    /// input) and implies a memory overhead.
    ///
    /// Thus, in case where unicity is already guaranteed (e.g. coming from a
    /// hashset, already checked beforehand, ...) it is possible to disable this
    /// check.
    #[must_use]
    pub const fn disable_duplicate_detection(mut self) -> Self {
        self.check_duplicate = false;
        self
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
        resolution: Resolution,
    ) -> Self {
        self.input_mode = InputMode::Heterogeneous(resolution);
        self
    }

    /// Builds the plotter.
    #[must_use]
    pub const fn build(self) -> Solvent {
        Solvent {
            input_mode: self.input_mode,
            check_duplicate: self.check_duplicate,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InputMode {
    /// An homogeneous set of cells.
    Homogeneous,
    /// An heterogeneous set of cells (e.g. compacted) with a max resolution.
    Heterogeneous(Resolution),
}
