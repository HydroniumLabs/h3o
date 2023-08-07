//! Bridge between H3 entities and geometrical shapes.
//!
//! The geometry types are essentially wrapper around
//! [`GeoRust`](https://georust.org/) types, with extra constraints enforced (e.g.
//! using radians) to make them compatible with the algorithms.
//!
//! The general idea here is to convert your `GeoRust` geometries before
//! applying your H3O-related processing and, when you're done, convert back to
//! a `GeoRust` type.
//! That way, you'll pay the cost of the conversion/validity check only once
//! (instead of every call). Moreover, some computations can be frontloaded and
//! cached in the wrapper type.
//!
//! Loading shapes from `GeoJSON` is also directly supported:
//!
//! ```no_run
//! use std::io;
//! use h3o::geom::Geometry;
//!
//! let geojson = geojson::GeoJson::from_reader(io::stdin())?;
//! let geometry = Geometry::try_from(&geojson)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod geometry;
mod json;
mod ring_hierarchy;
mod to_geo;
mod to_h3;
mod vertex_graph;

use ring_hierarchy::RingHierarchy;
use vertex_graph::VertexGraph;

pub use geometry::{
    Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
pub use to_geo::ToGeo;
pub use to_h3::{ContainmentMode, PolyfillConfig, ToCells};
