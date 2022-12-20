use super::Geometry;
use crate::{error::InvalidGeometry, geom::ToCells, CellIndex, Resolution};
use std::boxed::Box;

/// A collection of [`geo::Geometry`].
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollection<'a>(Vec<Geometry<'a>>);

impl<'a> GeometryCollection<'a> {
    /// Initialize a collection of geometries from geometries whose coordinates
    /// are in radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the geometry is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::GeometryCollection;
    ///
    /// let p = geo::point!(x: 0.0409980285, y: 0.852850182);
    /// let pe = geo::Geometry::Point(p);
    /// let gc = geo::GeometryCollection::new_from(vec![pe]);
    /// let collection = GeometryCollection::from_radians(&gc)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        geometries: &'a geo::GeometryCollection<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            geometries
                .iter()
                .map(Geometry::from_radians)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    /// Initialize a collection of geometries from geometries whose coordinates
    /// are in degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the geometry is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::GeometryCollection;
    ///
    /// let p = geo::point!(x: 2.349014, y: 48.864716);
    /// let pe = geo::Geometry::Point(p);
    /// let gc = geo::GeometryCollection::new_from(vec![pe]);
    /// let collection = GeometryCollection::from_degrees(gc)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        geometries: geo::GeometryCollection<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            geometries
                .into_iter()
                .map(Geometry::from_degrees)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<GeometryCollection<'_>> for geo::GeometryCollection<f64> {
    fn from(value: GeometryCollection<'_>) -> Self {
        Self(value.0.into_iter().map(Into::into).collect())
    }
}

impl ToCells for GeometryCollection<'_> {
    fn max_cells_count(&self, resolution: Resolution) -> usize {
        self.0
            .iter()
            .map(|geometry| geometry.max_cells_count(resolution))
            .sum()
    }

    fn to_cells(
        &self,
        resolution: Resolution,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(
            self.0
                .iter()
                .flat_map(move |geometry| geometry.to_cells(resolution)),
        )
    }
}
