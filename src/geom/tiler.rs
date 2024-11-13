use crate::{error::InvalidGeometry, CellIndex, Resolution};
use geo::Polygon;

/// A tiler that produces an H3 coverage of the given shapes.
#[derive(Debug, Clone)]
pub struct Tiler {
    resolution: Resolution,
    containment_mode: ContainmentMode,
    convert_to_rads: bool,
    geom: Vec<crate::geom::Polygon>,
}

impl Tiler {
    /// Adds a `Polygon` to tile.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the polygon is invalid.
    pub fn add(&mut self, polygon: Polygon) -> Result<(), InvalidGeometry> {
        let polygon = if self.convert_to_rads {
            crate::geom::Polygon::from_degrees(polygon)?
        } else {
            crate::geom::Polygon::from_radians(polygon)?
        };

        self.geom.push(polygon);

        Ok(())
    }

    /// Adds a batch of `Polygon` to tile.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the polygon is invalid.
    pub fn add_batch(
        &mut self,
        geoms: impl IntoIterator<Item = Polygon>,
    ) -> Result<(), InvalidGeometry> {
        for polygon in geoms {
            self.add(polygon)?;
        }
        Ok(())
    }

    /// Returns an upper bound to the number of cells returned by `into_coverage`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use geo::{LineString, Polygon};
    /// use h3o::{geom::{ContainmentMode, TilerBuilder}, Resolution};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    /// let mut tiler = TilerBuilder::new(Resolution::Ten)
    ///     .containment_mode(ContainmentMode::Covers)
    ///     .build();
    /// tiler.add(polygon)?;
    ///
    /// let size_hint = tiler.coverage_size_hint();
    ///
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    #[must_use]
    pub fn coverage_size_hint(&self) -> usize {
        self.geom
            .iter()
            .map(|polygon| polygon.max_cells_count(self.resolution))
            .sum()
    }

    /// Computes the cell coverage of the geometries.
    ///
    /// The output may contain duplicate indexes in case of overlapping input
    /// geometries/depending on the selected containment mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use geo::{LineString, Polygon};
    /// use h3o::{geom::{ContainmentMode, TilerBuilder}, Resolution};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    /// let mut tiler = TilerBuilder::new(Resolution::Ten)
    ///     .containment_mode(ContainmentMode::Covers)
    ///     .build();
    /// tiler.add(polygon)?;
    ///
    /// let cells = tiler.into_coverage().collect::<Vec<_>>();
    ///
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn into_coverage(self) -> impl Iterator<Item = CellIndex> {
        self.geom.into_iter().flat_map(move |polygon| {
            polygon.into_cells(self.resolution, self.containment_mode)
        })
    }
}

// -----------------------------------------------------------------------------

/// A builder to configure a tiler.
pub struct TilerBuilder {
    resolution: Resolution,
    containment_mode: ContainmentMode,
    convert_to_rads: bool,
}

impl TilerBuilder {
    /// Initializes a new plotter builder with default settings.
    #[must_use]
    pub const fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            containment_mode: ContainmentMode::ContainsCentroid,
            convert_to_rads: true,
        }
    }

    /// Disable the degrees-to-radians conversion pre-processing.
    #[must_use]
    pub const fn disable_radians_conversion(mut self) -> Self {
        self.convert_to_rads = false;
        self
    }

    /// Set the containment mode defining if a cell is in a polygon or not.
    #[must_use]
    pub const fn containment_mode(mut self, mode: ContainmentMode) -> Self {
        self.containment_mode = mode;
        self
    }

    /// Builds the plotter.
    #[must_use]
    pub const fn build(self) -> Tiler {
        Tiler {
            resolution: self.resolution,
            containment_mode: self.containment_mode,
            convert_to_rads: self.convert_to_rads,
            geom: Vec::new(),
        }
    }
}

// -----------------------------------------------------------------------------

/// Containment mode used to decide if a cell is contained in a polygon or not.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainmentMode {
    /// This mode will select every cells whose centroid are contained inside
    /// the polygon.
    ///
    /// This is the fasted option and ensures that every cell is uniquely
    /// assigned (e.g. two adjacent polygon with zero overlap also have zero
    /// overlapping cells).
    ///
    /// On the other hand, some cells may cover area outside of the polygon
    /// (overshooting) and some parts of the polygon may be left uncovered.
    ContainsCentroid,

    /// This mode will select every cells whose boundaries are entirely within
    /// the polygon.
    ///
    /// This ensures that every cell is uniquely assigned  (e.g. two adjacent
    /// polygon with zero overlap also have zero overlapping cells) and avoids
    /// any coverage overshooting.
    ///
    /// Some parts of the polygon may be left uncovered (more than with
    /// `ContainsCentroid`).
    ContainsBoundary,

    /// This mode will select every cells whose boundaries are within the
    /// polygon, even partially.
    ///
    /// This guarantees a complete coverage of the polygon, but some cells may
    /// belong to two different polygons if they are adjacent/close enough. Some
    /// cells may cover area outside of the polygon.
    ///
    /// Note that if the geometry is fully contained within a cell, this mode
    /// returns nothing (because there are no boundaries intersection).
    IntersectsBoundary,

    /// This mode behaves the same as `IntersectsBoundary`, but also handles the
    /// case where the geometry is being covered by a cell without intersecting
    /// with its boundaries. In such cases, the covering cell is returned.
    Covers,
}
