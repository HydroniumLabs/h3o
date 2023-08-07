use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, Polygon, ToCells},
    CellIndex,
};
use std::boxed::Box;

/// An axis-aligned bounded 2D rectangle whose area is defined by minimum and
/// maximum [`geo::Coord`]s.
#[derive(Clone, Debug, PartialEq)]
pub struct Rect(Polygon);

impl Rect {
    /// Initialize a new rectangle from a rect whose coordinates are in radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the rectangle is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Rect;
    ///
    /// let rect = geo::Rect::new(
    ///    geo::coord! { x: 1.808355449236779, y: 0.02086683484240935 },
    ///    geo::coord! { x: 1.816212429233187, y: 0.02571835428268519 },
    /// );
    /// let rect = Rect::from_radians(rect)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(rect: geo::Rect<f64>) -> Result<Self, InvalidGeometry> {
        Ok(Self(Polygon::from_rect(rect)?))
    }

    /// Initialize a new rectangle from a rect whose coordinates are in degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the rectangle is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Rect;
    ///
    /// let rect = geo::Rect::new(
    ///    geo::coord! { x: 103.61113510075143, y: 1.19558156826659 },
    ///    geo::coord! { x: 104.0613068942643,  y: 1.473553156420067 },
    /// );
    /// let rect = Rect::from_degrees(rect)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(rect: geo::Rect<f64>) -> Result<Self, InvalidGeometry> {
        Ok(Self(Polygon::from_degrees(rect.to_polygon())?))
    }
}

impl From<Rect> for geo::Rect<f64> {
    fn from(value: Rect) -> Self {
        value.0.bbox()
    }
}

impl ToCells for Rect {
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
