use h3o::geom::Geometry;

#[test]
fn from_geometry() {
    let json = r#"
{
    "type": "Point",
    "coordinates": [ -118.2836, 34.0956 ]
}
"#;
    let geojson = json.parse::<geojson::GeoJson>().expect("geojson");
    let result = Geometry::try_from(&geojson);

    assert!(result.is_ok());
}

#[test]
fn from_feature() {
    let json = r#"
{
  "type": "Feature",
  "geometry": {
    "type": "Point",
    "coordinates": [ -118.2836, 34.0956 ]
  }
}
"#;
    let geojson = json.parse::<geojson::GeoJson>().expect("geojson");
    let result = Geometry::try_from(&geojson);

    assert!(result.is_ok());
}

#[test]
fn from_collection() {
    let json = r#"
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": {},
      "geometry": {
        "type": "Point",
        "coordinates": [
          -0.13583511114120483,
          51.5218870403801
        ]
      }
    }
  ]
}
"#;
    let geojson = json.parse::<geojson::GeoJson>().expect("geojson");
    let result = Geometry::try_from(&geojson);

    assert!(result.is_ok());
}
