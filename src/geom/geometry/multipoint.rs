use super::Point;
use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex,
};
use std::boxed::Box;

/// A collection of [`geo::Point`]s.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPoint(Vec<Point>);

impl MultiPoint {
    /// Initialize a collection of points from points whose coordinates are in
    /// radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the point is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::MultiPoint;
    ///
    /// let points: geo::MultiPoint<f64> = vec![
    ///     (-2.1489548115593986, 0.8584581881195188),
    ///     (-1.382430711985295,  0.7628836324009612),
    /// ].into();
    /// let points = MultiPoint::from_radians(points)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        points: geo::MultiPoint<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            points
                .into_iter()
                .map(Point::from_radians)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    /// Initialize a collection of points from points whose coordinates are in
    /// degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if one of the point is invalid (e.g. contains
    /// non-finite coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::MultiPoint;
    ///
    /// let points: geo::MultiPoint<f64> = vec![
    ///     (-123.12604106668468, 49.18603106769609),
    ///     (-79.20744526602287,  43.71001239618482),
    /// ].into();
    /// let points = MultiPoint::from_degrees(&points)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        points: &geo::MultiPoint<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(Self(
            points
                .iter()
                .copied()
                .map(Point::from_degrees)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<MultiPoint> for geo::MultiPoint<f64> {
    fn from(value: MultiPoint) -> Self {
        Self(value.0.into_iter().map(Into::into).collect())
    }
}

impl ToCells for MultiPoint {
    fn max_cells_count(&self, _config: PolyfillConfig) -> usize {
        self.0.len()
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        Box::new(self.0.iter().flat_map(move |point| point.to_cells(config)))
    }
}
