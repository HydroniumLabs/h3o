use crate::{CellIndex, Resolution};
use std::boxed::Box;

/// A trait to convert a geometry (or a collection of geometries) into a list of
/// cell indexes of the specified resolution.
pub trait ToCells {
    /// Returns an upper bound to the number of cells returned by `to_cells`.
    ///
    /// Can be used to preallocate memory for [`Self::to_cells`].
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{Resolution, geom::{Point, PolyfillConfig, ToCells}};
    ///
    /// let p = geo::point!(x: 2.349014, y: 48.864716);
    /// let point = Point::from_degrees(p)?;
    /// let count = point.max_cells_count(PolyfillConfig::new(Resolution::Nine));
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    fn max_cells_count(&self, config: PolyfillConfig) -> usize;

    /// Computes the coverage of the input using cell indexes of the specified
    /// resolution.
    ///
    /// The output may contain duplicate indexes in case of overlapping input
    /// geometries/depending on the selected containment mode.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use geojson::GeoJson;
    /// use h3o::{Resolution, geom::{Geometry, Polygon, ToCells, PolyfillConfig}};
    /// use std::{fs::File, io::BufReader};
    ///
    /// let file = File::open("foo.geojson")?;
    /// let reader = BufReader::new(file);
    /// let geojson = GeoJson::from_reader(reader)?;
    /// let geometry = Geometry::try_from(&geojson)?;
    /// let polygon = Polygon::try_from(geometry)?;
    /// let cells = polygon.to_cells(PolyfillConfig::new(Resolution::Seven)).collect::<Vec<_>>();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    // TODO: use `impl Iterator` when RPITIT are stabilized.
    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_>;
}

// -----------------------------------------------------------------------------

/// Polyfill configuration.
#[derive(Clone, Copy, Debug)]
pub struct PolyfillConfig {
    pub(crate) resolution: Resolution,
    pub(crate) containment: ContainmentMode,
}

impl PolyfillConfig {
    /// Instanciate a default configuration.
    #[must_use]
    pub const fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            containment: ContainmentMode::ContainsCentroid,
        }
    }

    /// Set the containment mode defining if a cell is in a polygon or not.
    #[must_use]
    pub const fn containment_mode(mut self, mode: ContainmentMode) -> Self {
        self.containment = mode;
        self
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

    /// This mode behaves the same as IntersectsBoundary, but also handles the
    /// case where the geometry is being covered by a cell without intersecting
    /// with its boundaries. In such cases, the covering cell is returned.
    Covers,
}
