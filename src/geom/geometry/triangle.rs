use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, Polygon, ToCells},
    CellIndex,
};
use geo::CoordsIter;
use std::boxed::Box;

/// A bounded 2D area whose three vertices are defined by [`geo::Coord`]s.
#[derive(Clone, Debug, PartialEq)]
pub struct Triangle(Polygon);

impl Triangle {
    /// Initialize a new triangle from a triangle whose coordinates are in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the triangle is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Triangle;
    ///
    /// let triangle = geo::Triangle::new(
    ///     geo::coord! { x: 0.18729839227657055, y: 1.044910527020031 },
    ///     geo::coord! { x: 0.314525021194105,   y: 1.034815187125519 },
    ///     geo::coord! { x: 0.4379538402932019,  y: 1.0496570489924186 },
    /// );
    /// let triangle = Triangle::from_radians(triangle)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        triangle: geo::Triangle<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(Polygon::from_triangle(triangle)?))
    }

    /// Initialize a new triangle from a triangle whose coordinates are in
    /// degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the triangle is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Triangle;
    ///
    /// let triangle = geo::Triangle::new(
    ///     geo::coord! { x: 10.731407387033187, y: 59.868963167038345 },
    ///     geo::coord! { x: 18.020956265684987, y: 59.29054279833275 },
    ///     geo::coord! { x: 25.092906670346963, y: 60.14091884342227 },
    /// );
    /// let triangle = Triangle::from_degrees(triangle)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        triangle: geo::Triangle<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(Polygon::from_degrees(triangle.to_polygon())?))
    }
}

impl From<Triangle> for geo::Triangle<f64> {
    fn from(value: Triangle) -> Self {
        let coords = value.0.exterior();
        // 3 vertex + 1 to close the loop.
        debug_assert_eq!(coords.coords_count(), 4);

        Self::new(coords[0], coords[1], coords[2])
    }
}

impl ToCells for Triangle {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        self.0.max_cells_count(config)
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        self.0.to_cells(config)
    }
}
