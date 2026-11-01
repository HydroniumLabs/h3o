use h3o::{LatLng, LatLngIndexer, Resolution};

#[test]
fn cross_resolution_indexing() {
    let res = Resolution::range(Resolution::Zero, Resolution::Fifteen)
        .collect::<Vec<_>>();
    let mut index = LatLngIndexer::default();

    // Hexagon.
    let ll = LatLng::new(39.7391989, -104.9890547).unwrap();
    let expected = res.iter().map(|&res| ll.to_cell(res)).collect::<Vec<_>>();
    let result = index.to_cells(ll, res.iter().copied()).collect::<Vec<_>>();
    assert_eq!(result, expected, "hexagon");

    // Pentagon.
    let ll = LatLng::new(64.70000012793487, 10.536199075467694).unwrap();
    let expected = res.iter().map(|&res| ll.to_cell(res)).collect::<Vec<_>>();
    let result = index.to_cells(ll, res.iter().copied()).collect::<Vec<_>>();
    assert_eq!(result, expected, "pentagon");

    // Antimeridian.
    let ll = LatLng::new(0., 180.).unwrap();
    let expected = res.iter().map(|&res| ll.to_cell(res)).collect::<Vec<_>>();
    let result = index.to_cells(ll, res.iter().copied()).collect::<Vec<_>>();
    assert_eq!(result, expected, "antimeridian");

    // Pole.
    let ll = LatLng::new(90.0, 0.).unwrap();
    let expected = res.iter().map(|&res| ll.to_cell(res)).collect::<Vec<_>>();
    let result = index.to_cells(ll, res.iter().copied()).collect::<Vec<_>>();
    assert_eq!(result, expected, "pole")
}

// Regular region inside Denver.
#[test]
fn scanline_regular() {
    let min = LatLng::new(39.7341989, -104.9990547).unwrap();
    let max = LatLng::new(39.7441989, -104.9790547).unwrap();
    scanline(min, max, Resolution::Ten);
}

// Region containing a pentagon.
#[test]
fn scanline_pentagon() {
    let min = LatLng::new(64.6950001, 10.5261991).unwrap();
    let max = LatLng::new(64.7050001, 10.5461991).unwrap();
    scanline(min, max, Resolution::Ten);
}

// Transmeridian region.
#[test]
fn scanline_transmeridian() {
    let min = LatLng::new(-16.801654, 179.9899813).unwrap();
    let max = LatLng::new(-16.791654, 180.0099813).unwrap();
    scanline(min, max, Resolution::Ten);
}

// Region around the pole.
#[test]
fn scanline_pole() {
    let min = LatLng::new(89.993, 30.263).unwrap();
    let max = LatLng::new(90.003, 30.283).unwrap();
    scanline(min, max, Resolution::Ten);
}

// Scan the given region (bounded by the min/max points) as a 3000x3000 grid and
// make sure the indexer returns the same results as the vanilla
// `LatLng.to_cell`.
fn scanline(min: LatLng, max: LatLng, resolution: Resolution) {
    let step_lat = (max.lat() - min.lat()) / 500.;
    let step_lng = (max.lng() - min.lng()) / 500.;

    let mut indexer = LatLngIndexer::default();

    let mut lat = min.lat();
    while lat < max.lat() {
        let mut lng = min.lng();
        while lng < max.lng() {
            let ll = LatLng::new(lat, lng).unwrap();
            let expected = ll.to_cell(resolution);
            let result = indexer.to_cell(ll, resolution);

            // Check that indexer returns the same output as `LatLng.to_cell`.
            assert_eq!(result, expected, "{ll}");

            lng += step_lng;
        }
        lat += step_lat;
    }
}
