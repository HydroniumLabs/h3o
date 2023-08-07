use super::Polygon;
use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex,
};
use std::boxed::Box;

/// A collection of [`geo::Polygon`].
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPolygon(Vec<Polygon>);

impl MultiPolygon {
    /// Initialize a collection of polygons from polygons whose coordinates are
    /// in radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the polygon is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::polygon;
    /// use h3o::geom::MultiPolygon;
    ///
    /// let p: geo::Polygon<f64> = polygon![
    ///     (x: 0.6559997912129759, y: 0.9726707149994819),
    ///     (x: 0.6573835290630796, y: 0.9726707149994819),
    ///     (x: 0.6573835290630796, y: 0.9735034901250053),
    ///     (x: 0.6559997912129759, y: 0.9735034901250053),
    ///     (x: 0.6559997912129759, y: 0.9726707149994819),
    /// ];
    /// let mp = geo::MultiPolygon::new(vec![p]);
    /// let multipolygon = MultiPolygon::from_radians(mp)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        polygons: geo::MultiPolygon<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            polygons
                .into_iter()
                .map(Polygon::from_radians)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    /// Initialize a collection of polygons from polygons whose coordinates are
    /// in degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the polygon is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use geo::polygon;
    /// use h3o::geom::MultiPolygon;
    ///
    /// let p: geo::Polygon<f64> = polygon![
    ///     (x: 37.58601939796671, y: 55.72992682544245),
    ///     (x: 37.66530173673016, y: 55.72992682544245),
    ///     (x: 37.66530173673016, y: 55.777641325418415),
    ///     (x: 37.58601939796671, y: 55.777641325418415),
    ///     (x: 37.58601939796671, y: 55.72992682544245),
    /// ];
    /// let mp = geo::MultiPolygon::new(vec![p]);
    /// let multipolygon = MultiPolygon::from_degrees(mp)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        polygons: geo::MultiPolygon<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            polygons
                .into_iter()
                .map(Polygon::from_degrees)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<MultiPolygon> for geo::MultiPolygon<f64> {
    fn from(value: MultiPolygon) -> Self {
        Self(value.0.into_iter().map(Into::into).collect())
    }
}

impl ToCells for MultiPolygon {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        self.0
            .iter()
            .map(|polygon| polygon.max_cells_count(config))
            .sum()
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(
            self.0
                .iter()
                .flat_map(move |polygon| polygon.to_cells(config)),
        )
    }
}
