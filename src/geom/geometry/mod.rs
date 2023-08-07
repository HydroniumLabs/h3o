use crate::{
    error::InvalidGeometry,
    geom::{PolyfillConfig, ToCells},
    CellIndex, TWO_PI,
};
use std::{boxed::Box, f64::consts::PI};

mod bbox;
mod geometrycollection;
mod line;
mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;
mod rect;
mod ring;
mod triangle;

use ring::Ring;

pub use geometrycollection::GeometryCollection;
pub use line::Line;
pub use linestring::LineString;
pub use multilinestring::MultiLineString;
pub use multipoint::MultiPoint;
pub use multipolygon::MultiPolygon;
pub use point::Point;
pub use polygon::Polygon;
pub use rect::Rect;
pub use triangle::Triangle;

// ----------------------------------------------------------------------------

/// An enum representing any possible geometry type.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Geometry {
    /// A single point represented by one [`geo::Coord`].
    Point(Point),
    /// A line segment represented by two [`geo::Coord`]s.
    Line(Line),
    /// A series of contiguous line segments represented by two or more
    /// [`geo::Coord`]s.
    LineString(LineString),
    /// A bounded area represented by one [`LineString`] exterior ring, and zero
    /// or more [`LineString`] interior rings.
    Polygon(Polygon),
    /// A collection of [`Point`]s.
    MultiPoint(MultiPoint),
    /// A collection of [`LineString`]s.
    MultiLineString(MultiLineString),
    /// A collection of [`Polygon`]s.
    MultiPolygon(MultiPolygon),
    /// A collection of [`Geometry`]s.
    GeometryCollection(GeometryCollection),
    /// An axis-aligned bounded rectangle represented by minimum and maximum
    /// [`geo::Coord`]s.
    Rect(Rect),
    /// A bounded area represented by three [`geo::Coord`] vertices.
    Triangle(Triangle),
}

impl Geometry {
    /// Initialize a geometry from a geometry whose coordinates are in radians.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the geometry is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Geometry;
    ///
    /// let p = geo::point!(x: 0.0409980285, y: 0.852850182);
    /// let pe = geo::Geometry::Point(p);
    /// let collection = Geometry::from_radians(pe)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_radians(
        geometry: geo::Geometry<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(match geometry {
            geo::Geometry::Point(point) => {
                Self::Point(Point::from_radians(point)?)
            }
            geo::Geometry::Line(line) => Self::Line(Line::from_radians(line)?),
            geo::Geometry::LineString(line) => {
                Self::LineString(LineString::from_radians(line)?)
            }
            geo::Geometry::Polygon(polygon) => {
                Self::Polygon(Polygon::from_radians(polygon)?)
            }
            geo::Geometry::MultiPoint(points) => {
                Self::MultiPoint(MultiPoint::from_radians(points)?)
            }
            geo::Geometry::MultiLineString(lines) => {
                Self::MultiLineString(MultiLineString::from_radians(lines)?)
            }
            geo::Geometry::MultiPolygon(polygons) => {
                Self::MultiPolygon(MultiPolygon::from_radians(polygons)?)
            }
            geo::Geometry::GeometryCollection(geometries) => {
                Self::GeometryCollection(GeometryCollection::from_radians(
                    geometries,
                )?)
            }
            geo::Geometry::Rect(rect) => Self::Rect(Rect::from_radians(rect)?),
            geo::Geometry::Triangle(triangle) => {
                Self::Triangle(Triangle::from_radians(triangle)?)
            }
        })
    }

    /// Initialize a geometry from a geometry whose coordinates are in degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidGeometry`] if the geometry is invalid (e.g. contains non-finite
    /// coordinates).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::geom::Geometry;
    ///
    /// let p = geo::point!(x: 2.349014, y: 48.864716);
    /// let pe = geo::Geometry::Point(p);
    /// let geom = Geometry::from_degrees(pe)?;
    /// # Ok::<(), h3o::error::InvalidGeometry>(())
    /// ```
    pub fn from_degrees(
        geometry: geo::Geometry<f64>,
    ) -> Result<Self, InvalidGeometry> {
        Ok(match geometry {
            geo::Geometry::Point(point) => {
                Self::Point(Point::from_degrees(point)?)
            }
            geo::Geometry::Line(line) => Self::Line(Line::from_degrees(line)?),
            geo::Geometry::LineString(line) => {
                Self::LineString(LineString::from_degrees(line)?)
            }
            geo::Geometry::Polygon(polygon) => {
                Self::Polygon(Polygon::from_degrees(polygon)?)
            }
            geo::Geometry::MultiPoint(points) => {
                Self::MultiPoint(MultiPoint::from_degrees(&points)?)
            }
            geo::Geometry::MultiLineString(lines) => {
                Self::MultiLineString(MultiLineString::from_degrees(lines)?)
            }
            geo::Geometry::MultiPolygon(polygons) => {
                Self::MultiPolygon(MultiPolygon::from_degrees(polygons)?)
            }
            geo::Geometry::GeometryCollection(geometries) => {
                Self::GeometryCollection(GeometryCollection::from_degrees(
                    geometries,
                )?)
            }
            geo::Geometry::Rect(rect) => Self::Rect(Rect::from_degrees(rect)?),
            geo::Geometry::Triangle(triangle) => {
                Self::Triangle(Triangle::from_degrees(triangle)?)
            }
        })
    }
}

impl From<Geometry> for geo::Geometry<f64> {
    fn from(value: Geometry) -> Self {
        match value {
            Geometry::Point(point) => Self::Point(point.into()),
            Geometry::Line(line) => Self::Line(line.into()),
            Geometry::LineString(line) => Self::LineString(line.into()),
            Geometry::Polygon(polygon) => Self::Polygon(polygon.into()),
            Geometry::MultiPoint(points) => Self::MultiPoint(points.into()),
            Geometry::MultiLineString(lines) => {
                Self::MultiLineString(lines.into())
            }
            Geometry::MultiPolygon(polygons) => {
                Self::MultiPolygon(polygons.into())
            }
            Geometry::GeometryCollection(geometries) => {
                Self::GeometryCollection(geometries.into())
            }
            Geometry::Rect(rect) => Self::Rect(rect.into()),
            Geometry::Triangle(triangle) => Self::Triangle(triangle.into()),
        }
    }
}

impl ToCells for Geometry {
    fn max_cells_count(&self, config: PolyfillConfig) -> usize {
        match *self {
            Self::Point(ref point) => point.max_cells_count(config),
            Self::Line(ref line) => line.max_cells_count(config),
            Self::LineString(ref line) => line.max_cells_count(config),
            Self::Polygon(ref polygon) => polygon.max_cells_count(config),
            Self::MultiPoint(ref points) => points.max_cells_count(config),
            Self::MultiLineString(ref lines) => lines.max_cells_count(config),
            Self::MultiPolygon(ref polygons) => {
                polygons.max_cells_count(config)
            }
            Self::GeometryCollection(ref geometries) => {
                geometries.max_cells_count(config)
            }
            Self::Rect(ref rect) => rect.max_cells_count(config),
            Self::Triangle(ref triangle) => triangle.max_cells_count(config),
        }
    }

    fn to_cells(
        &self,
        config: PolyfillConfig,
    ) -> Box<dyn Iterator<Item = CellIndex> + '_> {
        match *self {
            Self::Point(ref point) => Box::new(point.to_cells(config)),
            Self::Line(ref line) => Box::new(line.to_cells(config)),
            Self::LineString(ref line) => Box::new(line.to_cells(config)),
            Self::Polygon(ref polygon) => Box::new(polygon.to_cells(config)),
            Self::MultiPoint(ref points) => Box::new(points.to_cells(config)),
            Self::MultiLineString(ref lines) => {
                Box::new(lines.to_cells(config))
            }
            Self::MultiPolygon(ref polygons) => {
                Box::new(polygons.to_cells(config))
            }
            Self::GeometryCollection(ref geometries) => {
                Box::new(geometries.to_cells(config))
            }
            Self::Rect(ref rect) => Box::new(rect.to_cells(config)),
            Self::Triangle(ref triangle) => Box::new(triangle.to_cells(config)),
        }
    }
}

// ----------------------------------------------------------------------------

// Check that the coordinate are finite and in a legit range.
fn coord_is_valid(coord: geo::Coord) -> bool {
    coord.x.is_finite()
        && coord.y.is_finite()
        && coord.x >= -TWO_PI
        && coord.x <= TWO_PI
        && coord.y >= -PI
        && coord.y <= PI
}
