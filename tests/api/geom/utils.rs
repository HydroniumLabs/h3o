use geo::{Geometry, Polygon};
use std::{fs::File, io::BufReader, path::PathBuf};

pub fn load_polygon(name: &str) -> Polygon {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let filepath = format!("dataset/shapes/{name}.geojson");
    path.push(filepath);

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    let geojson = geojson::GeoJson::from_reader(reader).expect("GeoJSON");
    let geometry = Geometry::try_from(geojson).expect("geometry");

    Polygon::try_from(geometry).expect("polygon")
}
