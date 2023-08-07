use super::{Geometry, GeometryCollection};
use crate::error::InvalidGeometry;

impl TryFrom<&geojson::Geometry> for Geometry {
    type Error = InvalidGeometry;

    fn try_from(value: &geojson::Geometry) -> Result<Self, Self::Error> {
        // A GeoJSON geometry can always be mapped to GeoRust geometry.
        let geometry = geo::Geometry::try_from(&value.value).expect("geometry");
        Self::from_degrees(geometry)
    }
}

impl TryFrom<&geojson::Feature> for Geometry {
    type Error = InvalidGeometry;

    fn try_from(value: &geojson::Feature) -> Result<Self, Self::Error> {
        value
            .geometry
            .as_ref()
            .ok_or_else(|| Self::Error::new("geometryless feature"))
            // A GeoJSON geometry can always be mapped to GeoRust geometry.
            .map(|geometry| {
                geo::Geometry::try_from(&geometry.value).expect("geometry")
            })
            .and_then(Self::from_degrees)
    }
}

impl TryFrom<&geojson::FeatureCollection> for Geometry {
    type Error = InvalidGeometry;

    fn try_from(
        value: &geojson::FeatureCollection,
    ) -> Result<Self, Self::Error> {
        let geometries = value
            .features
            .iter()
            // Ignore geometryless features.
            .filter_map(|feature| {
                // A GeoJSON geometry can always be mapped to GeoRust geometry.
                feature.geometry.as_ref().map(|geometry| {
                    geo::Geometry::try_from(&geometry.value).expect("geometry")
                })
            })
            .collect::<Vec<_>>();

        GeometryCollection::from_degrees(geo::GeometryCollection(geometries))
            .map(Geometry::GeometryCollection)
    }
}

impl TryFrom<&geojson::GeoJson> for Geometry {
    type Error = InvalidGeometry;

    fn try_from(value: &geojson::GeoJson) -> Result<Self, Self::Error> {
        match *value {
            geojson::GeoJson::Geometry(ref geom) => Self::try_from(geom),
            geojson::GeoJson::Feature(ref feature) => Self::try_from(feature),
            geojson::GeoJson::FeatureCollection(ref collection) => {
                Self::try_from(collection)
            }
        }
    }
}
