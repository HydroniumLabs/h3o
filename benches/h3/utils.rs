use h3o::CellIndex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub fn load_cells(resolution: u32) -> Vec<CellIndex> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let filepath = format!("dataset/Paris/cells-res{resolution}.txt");
    path.push(filepath);

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            let line = line.expect("test input");
            line.parse::<CellIndex>().expect("cell index")
        })
        .collect()
}

pub fn load_polygon(name: &str) -> h3o::geom::Polygon {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let filepath = format!("dataset/{name}/shape.geojson");
    path.push(filepath);

    let file = File::open(path).expect("open test dataset");
    let reader = BufReader::new(file);

    let geojson = geojson::GeoJson::from_reader(reader).expect("GeoJSON");
    let geometry = h3o::geom::Geometry::try_from(&geojson).expect("geometry");
    h3o::geom::Polygon::try_from(geometry).expect("polygon")
}
