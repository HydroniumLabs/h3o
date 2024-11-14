use geo::{coord, polygon, Geometry, LineString, MultiPolygon, Polygon, Rect};
use h3o::{
    geom::{ContainmentMode, TilerBuilder},
    CellIndex, LatLng, Resolution,
};
use std::{
    collections::BTreeSet, f64::consts::PI, fs::File, io::BufReader,
    path::PathBuf,
};

#[test]
fn add_rads() {
    let mut tiler = TilerBuilder::new(Resolution::Two)
        .disable_radians_conversion()
        .build();
    let polygon = polygon!(
        exterior: [
            (x: 0.6559997912129759, y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9726707149994819),
            (x: 0.6573835290630796, y: 0.9735034901250053),
            (x: 0.6559997912129759, y: 0.9735034901250053),
            (x: 0.6559997912129759, y: 0.9726707149994819),
        ],
        interiors: [],
    );

    let result = tiler.add(polygon);

    assert!(result.is_ok());
}

#[test]
fn add_degs() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let polygon = load_polygon("Paris");
    let result = tiler.add(polygon);

    assert!(result.is_ok());
}

#[test]
fn add_batch() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let polygons = MultiPolygon::new(vec![
        load_polygon("Paris"),
        load_polygon("Rabi"),
        load_polygon("Holes"),
    ]);
    let result = tiler.add_batch(polygons.0);

    assert!(result.is_ok());
}

#[test]
fn add_polygon_with_nan() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let result = tiler.add(polygon![
        (x: -1., y: 3.),
        (x: -1., y: 1.),
        (x: -2., y: f64::NAN),
        (x: -2., y: 3.)
    ]);

    assert!(result.is_err());
}

#[test]
fn add_point() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let result = tiler.add(polygon![
        (x: -1., y: 3.),
    ]);

    assert!(result.is_err());
}

#[test]
fn add_line() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let result = tiler.add(polygon![
        (x: -1., y: 3.),
        (x: -1., y: 1.),
    ]);

    assert!(result.is_err());
}

#[test]
fn coverage_size_hint() {
    let mut tiler = TilerBuilder::new(Resolution::Two).build();
    let polygon = load_polygon("Paris");
    tiler.add(polygon).expect("failed to add polygon");
    let bound = tiler.coverage_size_hint();
    let result = tiler.into_coverage().count();

    assert!(result <= bound);
}

#[test]
fn paris_coverage_contains_centroid() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::ContainsCentroid)
        .build();
    let polygon = load_polygon("Paris");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 164, "Paris/mode=Centroid");
}

#[test]
fn paris_coverage_contains_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::ContainsBoundary)
        .build();
    let polygon = load_polygon("Paris");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 118, "Paris/mode=Contains");
}

#[test]
fn paris_coverage_intersects_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::IntersectsBoundary)
        .build();
    let polygon = load_polygon("Paris");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 203, "Paris/mode=Intersects");
}

#[test]
fn rabi_coverage_contains_centroid() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::ContainsCentroid)
        .build();
    let polygon = load_polygon("Rabi");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 163, "Rabi/mode=Centroid");
}

#[test]
fn rabi_coverage_contains_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::ContainsBoundary)
        .build();
    let polygon = load_polygon("Rabi");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 132, "Rabi/mode=Contains");
}

#[test]
fn rabi_coverage_intersects_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Eight)
        .containment_mode(ContainmentMode::IntersectsBoundary)
        .build();
    let polygon = load_polygon("Rabi");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 193, "Rabi/mode=Intersects");
}

#[test]
fn holes_coverage_contains_centroid() {
    let mut tiler = TilerBuilder::new(Resolution::Four)
        .containment_mode(ContainmentMode::ContainsCentroid)
        .build();
    let polygon = load_polygon("Holes");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 233, "Holes/mode=Centroid");
}

#[test]
fn holes_coverage_contains_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Four)
        .containment_mode(ContainmentMode::ContainsBoundary)
        .build();
    let polygon = load_polygon("Holes");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 170, "Holes/mode=Contains");
}

#[test]
fn holes_coverage_intersects_boundary() {
    let mut tiler = TilerBuilder::new(Resolution::Four)
        .containment_mode(ContainmentMode::IntersectsBoundary)
        .build();
    let polygon = load_polygon("Holes");
    tiler.add(polygon).expect("failed to add polygon");
    let result = tiler.into_coverage().count();

    assert_eq!(result, 285, "Holes/mode=Intersects");
}

// -----------------------------------------------------------------------------

macro_rules! world_test {
    ($name:ident, $resolution: literal, $expected: literal) => {
        #[test]
        #[allow(unused_comparisons, reason = "when `$exact_count` is 0")]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("resolution");
            let expected_count = usize::try_from(resolution.cell_count())
                .expect("cell count cast");

            let mut tiler = TilerBuilder::new(resolution).build();
            let shape1 = load_polygon("HalfWorld_1");
            tiler.add(shape1).expect("failed to add polygon");
            let count1 = tiler.coverage_size_hint();
            let cells1 = tiler.into_coverage().collect::<BTreeSet<_>>();

            assert_eq!(count1, $expected);
            assert!(count1 >= cells1.len());

            let mut tiler = TilerBuilder::new(resolution).build();
            let shape2 = load_polygon("HalfWorld_2");
            tiler.add(shape2).expect("failed to add polygon");
            let count2 = tiler.coverage_size_hint();
            let cells2 = tiler.into_coverage().collect::<BTreeSet<_>>();

            assert_eq!(count2, $expected);
            assert!(count2 >= cells2.len());

            assert_eq!(
                cells1.len() + cells2.len(),
                expected_count,
                "cell count"
            );
            assert!(cells1.is_disjoint(&cells2), "no overlap");
        }
    };
}

// https://github.com/uber/h3-js/issues/76#issuecomment-561204505
world_test!(entire_world_res0, 0, 192);
world_test!(entire_world_res1, 1, 1565);
world_test!(entire_world_res2, 2, 10212);

// -----------------------------------------------------------------------------

macro_rules! test_count {
    ($name:ident, $polygon:expr, $resolution: literal, $expected_max: literal, $expected: literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("resolution");

            let mut tiler = TilerBuilder::new(resolution).build();
            tiler.add($polygon).expect("failed to add polygon");
            let count = tiler.coverage_size_hint();
            let result = tiler.into_coverage().count();

            assert_eq!(count, $expected_max);
            assert_eq!(result, $expected);
        }
    };
}

// https://github.com/uber/h3-js/issues/67
test_count!(h3js_67, load_polygon("h3js_issue67_1"), 7, 13882, 4499);
test_count!(h3js_67_2nd, load_polygon("h3js_issue67_2"), 7, 13318, 4609);
// https://github.com/uber/h3/issues/136
test_count!(h3_136, load_polygon("h3_issue136"), 13, 653548, 4353);
// https://github.com/uber/h3/issues/595
test_count!(h3_595, h3_595_shape(), 5, 76, 8);
test_count!(san_francisco, load_polygon("SanFrancisco"), 9, 5613, 1253);
test_count!(hole, load_polygon("SanFranciscoHole"), 9, 5613, 1214);
test_count!(empty, load_polygon("Empty"), 9, 15, 0);
test_count!(exact, hexagon_shape(), 9, 18, 1);
test_count!(pentagon, pentagon_shape(), 9, 16, 1);
test_count!(
    prime_meridian,
    load_polygon("PrimeMeridian"),
    7,
    16020,
    4228
);
test_count!(transmeridian, load_polygon("Transmeridian"), 7, 10030, 4238);
test_count!(
    transmeridian_hole,
    load_polygon("TransmeridianHole"),
    7,
    10030,
    3176
);
test_count!(
    transmeridian_complex,
    load_polygon("TransmeridianComplex"),
    4,
    8298,
    1204
);

// -----------------------------------------------------------------------------

macro_rules! exhaustive_test {
    ($name:ident, $resolution: literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                let ring = index.boundary().into();
                // Skip index that crosses the meridian.
                if index_is_transmeridian(&ring) {
                    continue;
                }
                let mut tiler = TilerBuilder::new(resolution)
                    .disable_radians_conversion()
                    .build();
                let shape = Polygon::new(ring.clone(), Vec::new());
                tiler.add(shape).expect("failed to add polygon");

                let result = tiler.into_coverage().collect::<BTreeSet<_>>();
                let expected =
                    index.children(resolution).collect::<BTreeSet<_>>();
                assert_eq!(
                    result, expected,
                    "cell {index} at given resolution"
                );

                let next_res = Resolution::try_from($resolution + 1)
                    .expect("next resolution");
                let mut tiler = TilerBuilder::new(next_res)
                    .disable_radians_conversion()
                    .build();
                let shape = Polygon::new(ring, Vec::new());
                tiler.add(shape).expect("failed to add polygon");
                let result = tiler.into_coverage().collect::<BTreeSet<_>>();
                let expected =
                    index.children(next_res).collect::<BTreeSet<_>>();
                assert_eq!(result, expected, "cell {index} at next resolution");
            }
        }
    };
}

// Return true if the cell index crosses the meridian.
fn index_is_transmeridian(boundary: &LineString<f64>) -> bool {
    let (min_lng, max_lng) =
        boundary.coords().fold((PI, -PI), |(min, max), coord| {
            (coord.x.min(min), coord.x.max(max))
        });
    max_lng - min_lng > PI - (PI / 4.)
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);

// -----------------------------------------------------------------------------

fn h3_595_shape() -> Polygon {
    let center = CellIndex::try_from(0x85283473fffffff).expect("center");
    let center_ll = LatLng::from(center);

    // This polygon should include the center cell. The issue here arises
    // when one of the polygon vertexes is to the east of the index center,
    // with exactly the same latitude
    polygon![
        (x: -121.53625488281249, y: center_ll.lat()),
        (x: -121.9317626953125,  y: 37.61640705577992),
        (x: -122.29980468749999, y: 37.330856613297144),
        (x: -121.904296875,      y: 37.05079312980657),
        (x: -121.53625488281249, y: center_ll.lat())
    ]
}

fn hexagon_shape() -> Polygon {
    let ll = LatLng::from_radians(1., 2.).expect("ll");
    let cell = ll.to_cell(Resolution::Nine);
    let ring = cell
        .boundary()
        .iter()
        .copied()
        .map(|ll| coord! {x: ll.lng(), y:ll.lat()})
        .collect();
    Polygon::new(ring, Vec::new())
}

fn pentagon_shape() -> Polygon {
    let pentagon = CellIndex::try_from(0x89300000003ffff).expect("pentagon");
    assert!(pentagon.is_pentagon());

    let ll = LatLng::from(pentagon);
    let coord = coord! {x: ll.lng(), y: ll.lat() };
    // Length of half an edge of the polygon, in radians.
    let edge_length_2 = 0.001;

    polygon![
        (x: coord.x - edge_length_2, y: coord.y - edge_length_2),
        (x: coord.x - edge_length_2, y: coord.y + edge_length_2),
        (x: coord.x + edge_length_2, y: coord.y + edge_length_2),
        (x: coord.x + edge_length_2, y: coord.y - edge_length_2),
        (x: coord.x - edge_length_2, y: coord.y - edge_length_2)
    ]
}

#[test]
fn fully_in_cell_contained_geometry() {
    // Build a geometry that is fully contained in the target cell.
    // The geometry does not touch the cells boundary
    let ll = LatLng::from_radians(1., 2.).expect("ll");
    let cell = ll.to_cell(Resolution::One);
    let cell_ring: Vec<_> = cell
        .center_child(Resolution::Four)
        .expect("center_child")
        .grid_disk_distances(2);
    let coord_ring = cell_ring
        .iter()
        .find(|(_, k)| *k == 2)
        .expect("first k=2 of ring")
        .0
        .boundary()
        .iter()
        .copied()
        .map(|ll| coord! {x: ll.lng_radians(), y:ll.lat_radians()})
        .collect();
    let shape = Polygon::new(coord_ring, Vec::new());

    // into coverage
    let mut tiler = TilerBuilder::new(cell.resolution())
        .disable_radians_conversion()
        .containment_mode(ContainmentMode::Covers)
        .build();
    tiler.add(shape).expect("failed to add polygon");
    let count = tiler.coverage_size_hint();
    let result = tiler.into_coverage().count();

    assert_eq!(count, 18);
    assert_eq!(result, 1);
}

// https://github.com/HydroniumLabs/h3o/issues/21
#[test]
fn issue_21() {
    let poly = load_polygon("h3o_issue21");
    let mut tiler = TilerBuilder::new(Resolution::Zero)
        .containment_mode(ContainmentMode::Covers)
        .build();
    tiler.add(poly).expect("failed to add polygon");
    let count = tiler.into_coverage().count();
    assert_eq!(count, 1);
}

// https://github.com/HydroniumLabs/h3o/issues/23
#[test]
fn issue_23() {
    let poly = load_polygon("h3o_issue23");
    let mut tiler = TilerBuilder::new(Resolution::Six)
        .containment_mode(ContainmentMode::Covers)
        .disable_transmeridian_heuristic()
        .build();
    tiler.add(poly).expect("failed to add polygon");
    let count = tiler.into_coverage().count();
    assert_eq!(count, 218_375);
}

// https://github.com/uber/h3-java/issues/138
#[test]
fn bug_h3_java_138() {
    let poly = load_polygon("h3java_issue138");
    let mut tiler = TilerBuilder::new(Resolution::Ten).build();
    tiler.add(poly).expect("failed to add polygon");
    let count = tiler.into_coverage().count();
    assert_eq!(count, 14_697);
}

// https://github.com/uber/h3-py/issues/343
#[test]
fn bug_h3_python_343() {
    let poly = load_polygon("h3py_issue343_1");
    let mut tiler = TilerBuilder::new(Resolution::Nine).build();
    tiler.add(poly).expect("failed to add polygon");
    let count = tiler.into_coverage().count();
    assert_eq!(count, 896);

    let poly = load_polygon("h3py_issue343_2");
    let mut tiler = TilerBuilder::new(Resolution::Nine).build();
    tiler.add(poly).expect("failed to add polygon");
    let count = tiler.into_coverage().count();
    assert_eq!(count, 1167);
}

macro_rules! cell {
    ($x: expr) => {{
        CellIndex::try_from($x).expect("valid cell")
    }};
}

// Case 1: Non-crossing shape on the west, intersecting with a crossing cell.
//
// antimeridian
//      │ ___
//      │/   \┌────────────┐
//   +--+     │--+         │
//  /   │     │   \        │
//  \   │     │   /        │
//   +  │     │  +         │
//  /   │     │   \        │
//  \   │     │   /        │
//   +--+     │--+         │
//      │\___/└────────────┘
//      │
#[test]
fn bbox_tile_west() {
    let bbox = Rect::new(
        coord! { x: -179.9986132979393, y: -16.890643703326294 },
        coord! { x: -179.99932676553726, y: -16.889961012741797 },
    );
    let mut expected = vec![
        cell!(0x8a9b4361e757fff),
        cell!(0x8a9b4361e62ffff),
        cell!(0x8a9b4361e75ffff),
    ];

    let mut tiler = TilerBuilder::new(Resolution::Ten)
        .containment_mode(ContainmentMode::Covers)
        .build();
    tiler.add(bbox.to_polygon()).expect("failed to add polygon");
    let mut result = tiler.into_coverage().collect::<Vec<_>>();

    expected.sort_unstable();
    result.sort_unstable();
    assert_eq!(result, expected);
}

// Case 2: Non-crossing shape on the east, intersecting with a crossing cell.
//
//           antimeridian
//                │ ___
// ┌────────────┐ │/   \
// │           +│-+     +--+
// │          / │ │         \
// │          \ │ │         /
// │           +│ │        +
// │          / │ │         \
// │          \ │ │         /
// │           +│-+     +--+
// └────────────┘ │\___/
//                │
#[test]
fn bbox_tile_east() {
    let bbox = Rect::new(
        coord! { x: 179.9986132979393, y: -16.890643703326294 },
        coord! { x: 179.99932676553726, y: -16.889961012741797 },
    );
    let mut expected = vec![
        cell!(0x8a9b4361e297fff),
        cell!(0x8a9b4361e2b7fff),
        cell!(0x8a9b4361e667fff),
        cell!(0x8a9b4361e74ffff),
    ];

    let mut tiler = TilerBuilder::new(Resolution::Ten)
        .containment_mode(ContainmentMode::Covers)
        .build();
    tiler.add(bbox.to_polygon()).expect("failed to add polygon");
    let mut result = tiler.into_coverage().collect::<Vec<_>>();

    expected.sort_unstable();
    result.sort_unstable();
    assert_eq!(result, expected);
}

// Case 3: Crossing shape, intersecting with a crossing cell.
//
//       antimeridian
//           │ ___
//           │/   \
//        + -+     +--+
//  ┌────────┬──────────────┐
//  │    \   │         /    │
//  │     +  │        +     │
//  │    /   │         \    │
//  └────────┴──────────────┘
//        + -+     +--+
//           │\___/
//           │
#[test]
fn bbox_transmeridian() {
    let bbox = Rect::new(
        coord! { x: -179.9986132979393, y: -16.890643703326294 },
        coord! { x: 179.9986132979393, y: -16.889961012741797 },
    );
    let mut expected = vec![
        cell!(0x8a9b4361e757fff),
        cell!(0x8a9b4361e62ffff),
        cell!(0x8a9b4361e75ffff),
        cell!(0x8a9b4361e297fff),
        cell!(0x8a9b4361e2b7fff),
        cell!(0x8a9b4361e667fff),
        cell!(0x8a9b4361e74ffff),
    ];

    let mut tiler = TilerBuilder::new(Resolution::Ten)
        .containment_mode(ContainmentMode::Covers)
        .build();
    tiler.add(bbox.to_polygon()).expect("failed to add polygon");
    let mut result = tiler.into_coverage().collect::<Vec<_>>();

    expected.sort_unstable();
    result.sort_unstable();
    assert_eq!(result, expected);
}

//------------------------------------------------------------------------------

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
