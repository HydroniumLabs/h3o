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
    /// use h3o::{Resolution, geom::{Point, ToCells}};
    ///
    /// let p = geo::point!(x: 2.349014, y: 48.864716);
    /// let point = Point::from_degrees(p)?;
    /// let count = point.max_cells_count(Resolution::Nine);
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    fn max_cells_count(&self, resolution: Resolution) -> usize;

    /// Computes the coverage of the input using cell indexes of the specified
    /// resolution.
    ///
    /// The output may contain duplicate indexes in case of overlapping input
    /// geometries.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use geojson::GeoJson;
    /// use h3o::{Resolution, geom::{Geometry, Polygon, ToCells}};
    /// use std::{fs::File, io::BufReader};
    ///
    /// let file = File::open("foo.geojson")?;
    /// let reader = BufReader::new(file);
    /// let geojson = GeoJson::from_reader(reader)?;
    /// let geometry = Geometry::try_from(&geojson)?;
    /// let polygon = Polygon::try_from(geometry)?;
    /// let cells = polygon.to_cells(Resolution::Seven).collect::<Vec<_>>();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    // TODO: use `impl Iterator` when RPITIT are stabilized.
    fn to_cells(
        &self,
        resolution: Resolution,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_>;
}
