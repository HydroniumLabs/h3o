use ahash::HashSet;
use geo::{coord, polygon, LineString};
use h3o::{
    geom::{ContainmentMode, PolyfillConfig, Polygon, ToCells},
    CellIndex, LatLng, Resolution,
};
use std::f64::consts::PI;

const PI_2: f64 = PI / 2.;

// -----------------------------------------------------------------------------

macro_rules! world_test {
    ($name:ident, $resolution: literal, $expected: literal) => {
        #[test]
        #[allow(unused_comparisons)] // When `$exact_count` is 0.
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("resolution");
            let config = PolyfillConfig::new(resolution);
            let expected_count = usize::try_from(resolution.cell_count())
                .expect("cell count cast");

            let shape1 =  polygon![
                (x: -PI, y: -PI_2),
                (x: -PI, y:  PI_2),
                (x:  0., y:  PI_2),
                (x:  0., y: -PI_2),
                (x: -PI, y: -PI_2)
            ];
            let poly1 = Polygon::from_radians(shape1).expect("poly 1");
            let count1 = poly1.max_cells_count(config);
            let cells1 = poly1.to_cells(config).collect::<HashSet<_>>();

            assert_eq!(count1, $expected);
            assert!(count1 >= cells1.len());

            let shape2 = polygon![
                (x: 0., y: -PI_2),
                (x: 0., y:  PI_2),
                (x: PI, y:  PI_2),
                (x: PI, y: -PI_2),
                (x: 0., y: -PI_2)
            ];
            let poly2 = Polygon::from_radians(shape2).expect("poly 2");
            let count2 = poly2.max_cells_count(config);
            let cells2 = poly2.to_cells(config).collect::<HashSet<_>>();

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
            let shape = $polygon;
            let polygon = Polygon::from_radians(shape).expect("polygon");
            let resolution =
                Resolution::try_from($resolution).expect("resolution");
            let config = PolyfillConfig::new(resolution);
            let count = polygon.max_cells_count(config);
            let result = polygon.to_cells(config).count();

            assert_eq!(count, $expected_max);
            assert_eq!(result, $expected);
        }
    };
}

// https://github.com/uber/h3-js/issues/76#issuecomment-561204505
test_count!(h3js_67, h3js_67_shape(), 7, 13882, 4499);
test_count!(h3js_67_2nd, h3js_67_2nd_shape(), 7, 13318, 4609);

// https://github.com/uber/h3/issues/136
test_count!(
    h3_136,
    polygon![
        (x: 0.8920772174196191, y: 0.10068990369902957),
        (x: 0.8915914753447348, y: 0.10032914690616246),
        (x: 0.8915860128746426, y: 0.10033349237998787),
        (x: 0.8920742194546231, y: 0.10069496685903621),
        (x: 0.8920772174196191, y: 0.10068990369902957)
    ],
    13,
    653548,
    4353
);

// https://github.com/uber/h3/issues/595
test_count!(h3_595, h3_595_shape(), 5, 76, 8);

// -----------------------------------------------------------------------------

test_count!(
    san_francisco,
    polygon![
        (x: -2.1364398519396, y: 0.659966917655 ),
        (x: -2.1359434279405, y: 0.6595011102219),
        (x: -2.1354884206045, y: 0.6583348114025),
        (x: -2.1382437718946, y: 0.6581220034068),
        (x: -2.1384597563896, y: 0.6594479998527),
        (x: -2.1376771158464, y: 0.6599990002976),
        (x: -2.1364398519396, y: 0.659966917655 ),
    ],
    9,
    5613,
    1253
);

test_count!(
    hole,
    polygon![
        exterior: [
            (x: -2.1364398519396, y: 0.659966917655),
            (x: -2.1359434279405, y: 0.6595011102219),
            (x: -2.1354884206045, y: 0.6583348114025),
            (x: -2.1382437718946, y: 0.6581220034068),
            (x: -2.1384597563896, y: 0.6594479998527),
            (x: -2.1376771158464, y: 0.6599990002976),
            (x: -2.1364398519396, y: 0.659966917655),
        ],
        interiors: [
            [
                (x: -2.1371053983433, y: 0.6595072188743),
                (x: -2.1373141048153, y: 0.6591482046471),
                (x: -2.1365222838402, y: 0.6592295020837),
                (x: -2.1371053983433, y: 0.6595072188743),
            ],
        ]
    ],
    9,
    5613,
    1214
);

test_count!(
    empty,
    polygon![
        (x: -2.1364398519394, y: 0.659966917655),
        (x: -2.1364398519395, y: 0.659966917655),
        (x: -2.1364398519396, y: 0.659966917655),
        (x: -2.1364398519394, y: 0.659966917655),
    ],
    9,
    15,
    0
);

test_count!(exact, hexagon_shape(), 9, 18, 1);

test_count!(pentagon, pentagon_shape(), 9, 16, 1);

// -----------------------------------------------------------------------------

test_count!(
    prime_meridian,
    polygon![
        (x:  0.01, y:  0.01),
        (x: -0.01, y:  0.01),
        (x: -0.01, y: -0.01),
        (x:  0.01, y: -0.01),
        (x:  0.01, y:  0.01),
    ],
    7,
    16020,
    4228
);

test_count!(
    transmeridian,
    polygon![
        (x: -PI + 0.01, y:  0.01),
        (x:  PI - 0.01, y:  0.01),
        (x:  PI - 0.01, y: -0.01),
        (x: -PI + 0.01, y: -0.01),
        (x: -PI + 0.01, y:  0.01),
    ],
    7,
    16020,
    4238
);

test_count!(
    transmeridian_hole,
    polygon![
        exterior: [
            (x: -PI + 0.01, y:  0.01),
            (x:  PI - 0.01, y:  0.01),
            (x:  PI - 0.01, y: -0.01),
            (x: -PI + 0.01, y: -0.01),
            (x: -PI + 0.01, y:  0.01),
        ],
        interiors: [
            [
                (x: -PI + 0.005, y:  0.005),
                (x:  PI - 0.005, y:  0.005),
                (x:  PI - 0.005, y: -0.005),
                (x: -PI + 0.005, y: -0.005),
                (x: -PI + 0.005, y:  0.005),
            ],
        ]
    ],
    7,
    16020,
    3176
);

test_count!(
    transmeridian_complex,
    polygon![
        (x: -PI + 0.00001, y:  0.1),
        (x:  PI - 0.00001, y:  0.1),
        (x:  PI - 0.2,     y:  0.05),
        (x:  PI - 0.00001, y: -0.1),
        (x: -PI + 0.00001, y: -0.1),
        (x: -PI + 0.2,     y: -0.05),
        (x: -PI + 0.00001, y:  0.1),
    ],
    4,
    5177,
    1204
);

// -----------------------------------------------------------------------------

macro_rules! exhaustive_test {
    ($name:ident, $resolution: literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            let config = PolyfillConfig::new(resolution);
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                let ring = index.boundary().into();
                // Skip index that crosses the meridian.
                if index_is_transmeridian(&ring) {
                    continue;
                }
                let shape = geo::Polygon::new(ring, Vec::new());
                let polygon = Polygon::from_radians(shape).expect("polygon");

                let result = polygon.to_cells(config).collect::<HashSet<_>>();
                let expected =
                    index.children(resolution).collect::<HashSet<_>>();
                assert_eq!(
                    result, expected,
                    "cell {index} at given resolution"
                );

                let next_res = Resolution::try_from($resolution + 1)
                    .expect("next resolution");
                let config = PolyfillConfig::new(next_res);
                let result = polygon.to_cells(config).collect::<HashSet<_>>();
                let expected = index.children(next_res).collect::<HashSet<_>>();
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

fn h3js_67_shape() -> geo::Polygon<f64> {
    let east = -56.25_f64.to_radians();
    let north = -33.13755119234615_f64.to_radians();
    let south = -34.30714385628804_f64.to_radians();
    let west = -57.65625_f64.to_radians();

    polygon![
        (x: east, y: north),
        (x: east, y: south),
        (x: west, y: south),
        (x: west, y: north),
        (x: east, y: north)
    ]
}

fn h3js_67_2nd_shape() -> geo::Polygon<f64> {
    let east = -57.65625_f64.to_radians();
    let north = -34.30714385628804_f64.to_radians();
    let south = -35.4606699514953_f64.to_radians();
    let west = -59.0625_f64.to_radians();

    polygon![
        (x: east, y: north),
        (x: east, y: south),
        (x: west, y: south),
        (x: west, y: north),
        (x: east, y: north)
    ]
}

fn h3_595_shape() -> geo::Polygon<f64> {
    let center = CellIndex::try_from(0x85283473fffffff).expect("center");
    let center_ll = LatLng::from(center);

    // This polygon should include the center cell. The issue here arises
    // when one of the polygon vertexes is to the east of the index center,
    // with exactly the same latitude
    polygon![
        (x: -2.121207808248113,  y: center_ll.lat_radians()),
        (x: -2.1281107217935986, y: 0.6565301558937859),
        (x: -2.1345342663428695, y: 0.6515463604919347),
        (x: -2.1276313527973842, y: 0.6466583305904194),
        (x: -2.121207808248113,  y: center_ll.lat_radians())
    ]
}

fn hexagon_shape() -> geo::Polygon<f64> {
    let ll = LatLng::from_radians(1., 2.).expect("ll");
    let cell = ll.to_cell(Resolution::Nine);
    let ring = cell
        .boundary()
        .iter()
        .copied()
        .map(|ll| coord! {x: ll.lng_radians(), y:ll.lat_radians()})
        .collect();
    geo::Polygon::new(ring, Vec::new())
}

fn pentagon_shape() -> geo::Polygon<f64> {
    let pentagon = CellIndex::try_from(0x89300000003ffff).expect("pentagon");
    assert!(pentagon.is_pentagon());

    let ll = LatLng::from(pentagon);
    let coord = coord! {x: ll.lng_radians(), y: ll.lat_radians() };
    // Length of half an edge of the polygon, in radians.
    let edge_length_2 = 0.001_f64.to_radians();

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
        .filter(|(_, k)| *k == 2)
        .next()
        .expect("first k=2 of ring")
        .0
        .boundary()
        .iter()
        .copied()
        .map(|ll| coord! {x: ll.lng_radians(), y:ll.lat_radians()})
        .collect();
    let shape = geo::Polygon::new(coord_ring, Vec::new());

    // to cells
    let polygon = Polygon::from_radians(shape).expect("polygon");
    let config = PolyfillConfig::new(cell.resolution())
        .containment_mode(ContainmentMode::Covers);
    let count = polygon.max_cells_count(config);
    let result = polygon.to_cells(config).count();

    assert_eq!(count, 18);
    assert_eq!(result, 1);
}

// https://github.com/HydroniumLabs/h3o/issues/21
#[test]
fn issue_21() {
    let poly = polygon![
        (x: 156.0, y: 6.0),
        (x: 156.0, y: 3.0),
        (x: 159.0, y: 3.0),
        (x: 159.0, y: 6.0),
        (x: 156.0, y: 6.0),
    ];
    let shape = Polygon::from_degrees(poly).unwrap();
    let config = PolyfillConfig::new(Resolution::Zero)
        .containment_mode(ContainmentMode::Covers);
    let count = shape.to_cells(config).count();
    assert_eq!(count, 1);
}
