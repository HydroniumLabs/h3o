use super::VertexGraph;
use crate::{
    error::OutlinerError, CellIndex, DirectedEdgeIndex, LatLng, VertexIndex,
};
use geo::{Coord, Line, LineString, MultiPolygon, Point, Polygon};
use std::convert::Infallible;

/// A trait to trace the outline of an H3 object.
pub trait ToGeo
where
    geojson::Value: for<'a> From<&'a <Self as ToGeo>::Output>,
{
    /// Output geometry type.
    type Output;
    /// The type returned in the event of an outlining error.
    type Error;

    /// Creates a geometry describing the outline(s).
    ///
    /// # Errors
    ///
    /// Error conditions depend on the implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution, geom::ToGeo};
    ///
    /// let index = CellIndex::try_from(0x089283470803ffff)?;
    /// let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();
    /// let geom = cells.to_geom(true)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn to_geom(self, use_degrees: bool) -> Result<Self::Output, Self::Error>;

    /// Creates a `GeoJSON` geometry describing the outline(s).
    ///
    /// # Errors
    ///
    /// Error conditions depend on the implementation.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution, geom::ToGeo};
    ///
    /// let index = CellIndex::try_from(0x089283470803ffff)?;
    /// let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();
    /// let geojson = cells.to_geojson()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn to_geojson(self) -> Result<geojson::Geometry, Self::Error>
    where
        Self: Sized,
    {
        self.to_geom(true)
            .map(|geom| geojson::Geometry::new(geojson::Value::from(&geom)))
    }
}

impl<T> ToGeo for T
where
    T: IntoIterator<Item = CellIndex>,
{
    type Error = OutlinerError;
    type Output = MultiPolygon<f64>;

    /// Creates a [`MultiPolygon`] describing the outline(s) of a set of cells.
    ///
    /// # Errors
    ///
    /// All cell indexes must be unique and have the same resolution, otherwise
    /// [`OutlinerError`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution, geom::ToGeo};
    ///
    /// let index = CellIndex::try_from(0x089283470803ffff)?;
    /// let cells = index.children(Resolution::Twelve).collect::<Vec<_>>();
    /// let geom = cells.to_geom(true)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn to_geom(self, use_degrees: bool) -> Result<Self::Output, Self::Error> {
        let res: Result<MultiPolygon<f64>, Self::Error> =
            VertexGraph::from_cells(self).map(Into::into);

        if use_degrees {
            return res;
        }

        res.map(|mut multipolygon| {
            for polygon in &mut multipolygon {
                polygon.exterior_mut(|exterior| {
                    for coord in exterior.coords_mut() {
                        coord.x = coord.x.to_radians();
                        coord.y = coord.y.to_radians();
                    }
                });
                polygon.interiors_mut(|interiors| {
                    for interior in &mut *interiors {
                        for coord in interior.coords_mut() {
                            coord.x = coord.x.to_radians();
                            coord.y = coord.y.to_radians();
                        }
                    }
                });
            }

            multipolygon
        })
    }
}

impl ToGeo for CellIndex {
    type Error = Infallible;
    type Output = Polygon<f64>;

    /// Creates a [`Polygon`] representing the boundary of the cell.
    ///
    /// # Errors
    ///
    /// This method cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, geom::ToGeo};
    ///
    /// let index = CellIndex::try_from(0x089283470803ffff)?;
    /// let boundary = index.to_geom(true).expect("cannot fail");
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    fn to_geom(self, use_degrees: bool) -> Result<Self::Output, Self::Error> {
        let mut boundary: LineString = self.boundary().into();

        if !use_degrees {
            for coord in boundary.coords_mut() {
                coord.x = coord.x.to_radians();
                coord.y = coord.y.to_radians();
            }
        }

        Ok(Polygon::new(boundary, Vec::new()))
    }
}

impl ToGeo for DirectedEdgeIndex {
    type Error = Infallible;
    type Output = Line<f64>;

    /// Creates a [`Line`] representing the directed edge of an H3 cell.
    ///
    /// # Errors
    ///
    /// This method cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{DirectedEdgeIndex, geom::ToGeo};
    ///
    /// let index = DirectedEdgeIndex::try_from(0x13a194e699ab7fff)?;
    /// let edge = index.to_geom(true).expect("cannot fail");
    /// # Ok::<(), h3o::error::InvalidDirectedEdgeIndex>(())
    /// ```
    fn to_geom(self, use_degrees: bool) -> Result<Self::Output, Self::Error> {
        let mut coords: Vec<Coord<f64>> =
            self.boundary().iter().copied().map(Into::into).collect();

        // We only have two point (start and end) as boundary for an edge.
        assert_eq!(coords.len(), 2);

        if !use_degrees {
            coords[0].x = coords[0].x.to_radians();
            coords[0].y = coords[0].y.to_radians();
            coords[1].x = coords[1].x.to_radians();
            coords[1].y = coords[1].y.to_radians();
        }

        Ok(Line::new(coords[0], coords[1]))
    }
}

impl ToGeo for VertexIndex {
    type Error = Infallible;
    type Output = Point<f64>;

    /// Creates a [`Point`] representing the vertex of an H3 cell.
    ///
    /// # Errors
    ///
    /// This method cannot fail.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{VertexIndex, geom::ToGeo};
    ///
    /// let index = VertexIndex::try_from(0x2302bfffffffffff)?;
    /// let point = index.to_geom(true).expect("cannot fail");
    /// # Ok::<(), h3o::error::InvalidVertexIndex>(())
    /// ```
    fn to_geom(self, use_degrees: bool) -> Result<Self::Output, Self::Error> {
        let mut coord: Coord<f64> = LatLng::from(self).into();

        if !use_degrees {
            coord.x = coord.x.to_radians();
            coord.y = coord.y.to_radians();
        }

        Ok(coord.into())
    }
}
