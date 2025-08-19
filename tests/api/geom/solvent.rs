use super::utils::load_polygon;
use ahash::HashSet;
use approx::{assert_relative_eq, relative_eq};
use geo::{Area, BooleanOps, LineString, MultiPolygon, Polygon, polygon};
use h3o::{
    CellIndex, Resolution,
    geom::{SolventBuilder, TilerBuilder},
};

#[test]
fn hole_in_center() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("index");
    let mut cells = index.children(Resolution::Ten).collect::<Vec<_>>();

    // Remove the center cell.
    let center = index.center_child(Resolution::Ten).expect("center");
    let idx = cells.iter().position(|x| *x == center).expect("idx");
    cells.remove(idx);

    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(cells.iter().copied()).expect("geometry");
    let expected = MultiPolygon::new(vec![polygon!(
        exterior: [
            (x: -122.02796455348616, y: 37.38525281604115 ),
            (x: -122.02732437374608, y: 37.385758270113065),
            (x: -122.02648011977477, y: 37.38558967035685 ),
            (x: -122.02583992481574, y: 37.38609511818443 ),
            (x: -122.02604398797318, y: 37.386769168218684),
            (x: -122.02540378194031, y: 37.38727461225182 ),
            (x: -122.02560784515092, y: 37.38794865717242 ),
            (x: -122.02645212137664, y: 37.38811725429045 ),
            (x: -122.02665619162275, y: 37.38879129032762 ),
            (x: -122.02750047073862, y: 37.38895987611164 ),
            (x: -122.02814066848063, y: 37.38845442717775 ),
            (x: -122.02898493935817, y: 37.38862300294707 ),
            (x: -122.0296251218798,  y: 37.38811754776844 ),
            (x: -122.02942103767036, y: 37.38744351927073 ),
            (x: -122.03006120911812, y: 37.38693806029814 ),
            (x: -122.02985712496266, y: 37.386264026686845),
            (x: -122.0290128763404,  y: 37.38609544827806 ),
            (x: -122.02880879921976, y: 37.38542140578344 )
        ],
        interiors: [
            [
                (x: -122.02752844388534, y: 37.386432316377665),
                (x: -122.02837270074619, y: 37.38660090480038 ),
                (x: -122.02857677792056, y: 37.38727494218174 ),
                (x: -122.0279365912526,  y: 37.38778039491016 ),
                (x: -122.02709232326434, y: 37.387611807806856),
                (x: -122.0268882530716,  y: 37.386937766655734)
            ],
        ],
    )]);

    assert_multipolygon_equivalent(&result, &expected, 1e-6);
    assert_hetero_equal_homo(cells, Resolution::Ten, &result);
}

// -----------------------------------------------------------------------------

#[test]
fn duplicate_simple() {
    let set = [0x89283082813ffff, 0x89283082817ffff, 0x89283082813ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = SolventBuilder::new().build().dissolve(set);

    assert!(result.is_err())
}

#[test]
fn consecutive_duplicate() {
    let set = [0x89283082813ffff, 0x89283082817ffff, 0x89283082817ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = SolventBuilder::new().build().dissolve(set);

    assert!(result.is_err())
}

#[test]
fn duplicate_heterogeneous() {
    let set = [0x89283082813ffff, 0x89283082817ffff, 0x8a283082810ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = SolventBuilder::new()
        .enable_heterogeneous_support(Resolution::Ten)
        .build()
        .dissolve(set);

    assert!(result.is_err())
}

#[test]
fn heterogeneous_resolution() {
    let set = [0x89283082813ffff, 0x8828308299fffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let result = SolventBuilder::new().build().dissolve(set.iter().copied());

    assert!(result.is_err(), "no heterogeneous support");

    // But this is fine when heterogeneous support is enabled.
    let result = SolventBuilder::new()
        .enable_heterogeneous_support(Resolution::Nine)
        .build()
        .dissolve(set);
    assert!(result.is_ok(), "heterogeneous support");
}

#[test]
fn unsupported_resolution() {
    let set = [0x89283082813ffff, 0x8828308299fffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();

    let result = SolventBuilder::new()
        .enable_heterogeneous_support(Resolution::Eight)
        .build()
        .dissolve(set.iter().copied());
    assert!(result.is_err(), "by dup-check");

    let result = SolventBuilder::default()
        .enable_heterogeneous_support(Resolution::Eight)
        .disable_duplicate_detection()
        .build()
        .dissolve(set);
    assert!(result.is_err(), "even with dup-check disabled");
}

#[test]
fn empty() {
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(std::iter::empty()).expect("geometry");
    assert!(result.0.is_empty());

    let solvent = SolventBuilder::new()
        .enable_heterogeneous_support(Resolution::Nine)
        .build();
    let result = solvent.dissolve(std::iter::empty()).expect("geometry");
    assert!(result.0.is_empty());
}

#[test]
fn hexagon() {
    //  __
    // /  \
    // \__/
    let set = [CellIndex::try_from(0x890dab6220bffff).expect("cell index")];
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 6-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn test_two_contiguous_cells() {
    //     __
    //  __/  \
    // /  \__/
    // \__/
    let set = [0x8928308291bffff, 0x89283082957ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (6 + 6 - 2 shared edges).
    assert_eq!(result.0[0].exterior().lines().count(), 10);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn two_non_contiguous_cells() {
    //  __      __
    // /  \    /  \
    // \__/    \__/
    let set = [0x8928308291bffff, 0x89283082943ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 2 polygon.
    assert_eq!(result.0.len(), 2);
    // Both with a 6-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6);
    assert_eq!(result.0[1].exterior().lines().count(), 6);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
    assert!(result.0[1].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn three_contiguous_cells() {
    //     __
    //  __/  \
    // /  \__/
    // \__/  \
    //    \__/
    let set = [0x8928308288bffff, 0x892830828d7ffff, 0x8928308289bffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (3*6 - 3*2 shared edges).
    assert_eq!(result.0[0].exterior().lines().count(), 12);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn hole() {
    //      __
    //   __/  \__
    //  /  \__/  \
    //  \__/  \__/
    //  /  \__/  \
    //  \__/  \__/
    //     \__/
    // Hole in the center.
    #[rustfmt::skip]
    let set = [
        0x892830828c7ffff, 0x892830828d7ffff, 0x8928308289bffff,
        0x89283082813ffff, 0x8928308288fffff, 0x89283082883ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 18-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 18);
    // And one 6-edge hole.
    assert_eq!(result.0[0].interiors().len(), 1);
    assert_eq!(result.0[0].interiors()[0].lines().count(), 6);

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn pentagon() {
    let set = [CellIndex::try_from(0x851c0003fffffff).expect("cell index")];
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (distorted pentagon).
    assert_eq!(result.0[0].exterior().lines().count(), 10);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn two_ring() {
    #[rustfmt::skip]
    let set = [
        0x8930062838bffff, 0x8930062838fffff, 0x89300628383ffff,
        0x8930062839bffff, 0x893006283d7ffff, 0x893006283c7ffff,
        0x89300628313ffff, 0x89300628317ffff, 0x893006283bbffff,
        0x89300628387ffff, 0x89300628397ffff, 0x89300628393ffff,
        0x89300628067ffff, 0x8930062806fffff, 0x893006283d3ffff,
        0x893006283c3ffff, 0x893006283cfffff, 0x8930062831bffff,
        0x89300628303ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With the expected number of edges on the outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6 * (2 * 2 + 1));
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn two_ring_unordered() {
    #[rustfmt::skip]
    let set = [
        0x89300628393ffff, 0x89300628383ffff, 0x89300628397ffff,
        0x89300628067ffff, 0x89300628387ffff, 0x893006283bbffff,
        0x89300628313ffff, 0x893006283cfffff, 0x89300628303ffff,
        0x89300628317ffff, 0x8930062839bffff, 0x8930062838bffff,
        0x8930062806fffff, 0x8930062838fffff, 0x893006283d3ffff,
        0x893006283c3ffff, 0x8930062831bffff, 0x893006283d7ffff,
        0x893006283c7ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With the expected number of edges on the outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6 * (2 * 2 + 1));
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn nested_donut() {
    // Hollow 1-ring + hollow 3-ring around the same hex.
    #[rustfmt::skip]
    let set = [
        0x89283082813ffff, 0x8928308281bffff, 0x8928308280bffff,
        0x8928308280fffff, 0x89283082807ffff, 0x89283082817ffff,
        0x8928308289bffff, 0x892830828d7ffff, 0x892830828c3ffff,
        0x892830828cbffff, 0x89283082853ffff, 0x89283082843ffff,
        0x8928308284fffff, 0x8928308287bffff, 0x89283082863ffff,
        0x89283082867ffff, 0x8928308282bffff, 0x89283082823ffff,
        0x89283082837ffff, 0x892830828afffff, 0x892830828a3ffff,
        0x892830828b3ffff, 0x89283082887ffff, 0x89283082883ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 2 polygon.
    assert_eq!(result.0.len(), 2);

    // One has 42-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 42);
    // And a single 30-edge hole.
    assert_eq!(result.0[0].interiors().len(), 1);
    assert_eq!(result.0[0].interiors()[0].lines().count(), 30);

    // The other has 18-edge outer ring.
    assert_eq!(result.0[1].exterior().lines().count(), 18);
    // And a single 6-edge hole.
    assert_eq!(result.0[1].interiors().len(), 1);
    assert_eq!(result.0[1].interiors()[0].lines().count(), 6);

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn nested_donut_transmeridian() {
    // Hollow 1-ring + hollow 3-ring around the hex at (0, -180).
    #[rustfmt::skip]
    let set = [
        0x897eb5722c7ffff, 0x897eb5722cfffff, 0x897eb572257ffff,
        0x897eb57220bffff, 0x897eb572203ffff, 0x897eb572213ffff,
        0x897eb57266fffff, 0x897eb5722d3ffff, 0x897eb5722dbffff,
        0x897eb573537ffff, 0x897eb573527ffff, 0x897eb57225bffff,
        0x897eb57224bffff, 0x897eb57224fffff, 0x897eb57227bffff,
        0x897eb572263ffff, 0x897eb572277ffff, 0x897eb57223bffff,
        0x897eb572233ffff, 0x897eb5722abffff, 0x897eb5722bbffff,
        0x897eb572287ffff, 0x897eb572283ffff, 0x897eb57229bffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 2 polygon.
    assert_eq!(result.0.len(), 2);

    // One has 42-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 42);
    // And a single 30-edge hole.
    assert_eq!(result.0[0].interiors().len(), 1);
    assert_eq!(result.0[0].interiors()[0].lines().count(), 30);

    // The other has 18-edge outer ring.
    assert_eq!(result.0[1].exterior().lines().count(), 18);
    // And a single 6-edge hole.
    assert_eq!(result.0[1].interiors().len(), 1);
    assert_eq!(result.0[1].interiors()[0].lines().count(), 6);

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn contiguous_distorted() {
    let set = [0x894cc5365afffff, 0x894cc536537ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"))
        .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let result = solvent.dissolve(set.iter().copied()).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].exterior().lines().count(), 12);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());

    assert_hetero_equal_homo(set, Resolution::Nine, &result);
}

#[test]
fn paris_heterogeneous() {
    let resolution = Resolution::Nine;
    let mut tiler = TilerBuilder::new(resolution).build();
    tiler.add(load_polygon("Paris")).expect("add polygon");
    let cells = tiler.into_coverage().collect::<HashSet<_>>();
    let mut compacted = cells.iter().copied().collect::<Vec<_>>();
    CellIndex::compact(&mut compacted).expect("compact");

    let solvent = SolventBuilder::new().disable_duplicate_detection().build();
    let expected = solvent.dissolve(cells).expect("homo geom");

    assert_hetero_equal_homo(compacted, resolution, &expected);
}

#[test]
fn rabi_heterogeneous() {
    let resolution = Resolution::Nine;
    let mut tiler = TilerBuilder::new(resolution).build();
    tiler.add(load_polygon("Rabi")).expect("add polygon");
    let cells = tiler.into_coverage().collect::<HashSet<_>>();
    let mut compacted = cells.iter().copied().collect::<Vec<_>>();
    CellIndex::compact(&mut compacted).expect("compact");

    let solvent = SolventBuilder::new().disable_duplicate_detection().build();
    let expected = solvent.dissolve(cells).expect("homo geom");

    assert_hetero_equal_homo(compacted, resolution, &expected);
}

#[test]
fn holes_heterogeneous() {
    let resolution = Resolution::Five;
    let mut tiler = TilerBuilder::new(resolution).build();
    tiler.add(load_polygon("Holes")).expect("add polygon");
    let cells = tiler.into_coverage().collect::<HashSet<_>>();
    let mut compacted = cells.iter().copied().collect::<Vec<_>>();
    CellIndex::compact(&mut compacted).expect("compact");

    let solvent = SolventBuilder::new().disable_duplicate_detection().build();
    let expected = solvent.dissolve(cells).expect("homo geom");

    assert_hetero_equal_homo(compacted, resolution, &expected);
}

// See https://github.com/HydroniumLabs/h3o/issues/12
#[test]
fn issue_12() {
    let set = [
        0x8b1f25a91526fff,
        0x8b1f2506d45dfff,
        0x8b1f25a901a1fff,
        0x8b1f25324b98fff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
    .collect::<Vec<_>>();
    let solvent = SolventBuilder::new().build();
    let geom = solvent.dissolve(set.iter().copied()).expect("geometry");

    assert_hetero_equal_homo(set, Resolution::Eleven, &geom);
}

// See https://github.com/uber/h3/issues/917
#[test]
fn issue_917() {
    let set = [
        0x81ea3ffffffffff,
        0x81eabffffffffff,
        0x81eafffffffffff,
        0x81eb3ffffffffff,
        0x81eb7ffffffffff,
        0x81ebbffffffffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"))
    .collect::<Vec<_>>();

    let solvent = SolventBuilder::new().build();
    let geom = solvent.dissolve(set.iter().copied()).expect("geometry");
    assert_eq!(geom.0.len(), 1, "a single polygon");
    assert!(geom.0[0].interiors().is_empty(), "no hole");

    assert_hetero_equal_homo(set, Resolution::One, &geom);
}

// This was a non-deterministic (due to hashing) bug in RingHierarchy.
// The antimeredian handling was triggered outside of legit cases.
#[test]
fn non_deterministic_output() {
    for _ in 0..10 {
        let set = [0x871861318ffffff, 0x871861383ffffff]
            .into_iter()
            .map(|bits| CellIndex::try_from(bits).expect("cell index"))
            .collect::<Vec<_>>();

        let solvent = SolventBuilder::new().build();
        let geom = solvent.dissolve(set.iter().copied()).expect("geometry");

        assert_eq!(geom.0.len(), 2);

        assert_hetero_equal_homo(set, Resolution::Seven, &geom);
    }
}

macro_rules! grid_disk {
    ($name:ident, $base_cell:literal, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("resolution");
            let base_cell =
                CellIndex::try_from($base_cell).expect("cell index");
            let origin =
                base_cell.center_child(resolution).expect("center cell");
            let set = origin.grid_disk::<Vec<_>>(2);
            let solvent = SolventBuilder::new().build();
            let result =
                solvent.dissolve(set.iter().copied()).expect("geometry");
            // Account for pentagon distortion on class II resolution.
            let expected = if base_cell.is_pentagon() {
                if resolution.is_class3() { 30 } else { 25 }
            } else {
                30
            };

            // 1 polygon.
            assert_eq!(result.0.len(), 1);
            assert_eq!(result.0[0].exterior().lines().count(), expected);
            // And no hole.
            assert!(result.0[0].interiors().is_empty());

            assert_hetero_equal_homo(set, resolution, &result);
        }
    };
}

grid_disk!(grid_disk_res1, 0x8073fffffffffff, 1);
grid_disk!(grid_disk_res2, 0x8073fffffffffff, 2);
grid_disk!(grid_disk_res3, 0x8073fffffffffff, 3);
grid_disk!(grid_disk_res4, 0x8073fffffffffff, 4);
grid_disk!(grid_disk_res5, 0x8073fffffffffff, 5);
grid_disk!(grid_disk_res6, 0x8073fffffffffff, 6);
grid_disk!(grid_disk_res7, 0x8073fffffffffff, 7);
grid_disk!(grid_disk_res8, 0x8073fffffffffff, 8);
grid_disk!(grid_disk_res9, 0x8073fffffffffff, 9);
grid_disk!(grid_disk_res10, 0x8073fffffffffff, 10);
grid_disk!(grid_disk_res11, 0x8073fffffffffff, 11);
grid_disk!(grid_disk_res12, 0x8073fffffffffff, 12);
grid_disk!(grid_disk_res13, 0x8073fffffffffff, 13);
grid_disk!(grid_disk_res14, 0x8073fffffffffff, 14);
grid_disk!(grid_disk_res15, 0x8073fffffffffff, 15);

grid_disk!(grid_disk_pentagon_res1, 0x8031fffffffffff, 1);
grid_disk!(grid_disk_pentagon_res2, 0x8031fffffffffff, 2);
grid_disk!(grid_disk_pentagon_res3, 0x8031fffffffffff, 3);
grid_disk!(grid_disk_pentagon_res4, 0x8031fffffffffff, 4);
grid_disk!(grid_disk_pentagon_res5, 0x8031fffffffffff, 5);
grid_disk!(grid_disk_pentagon_res6, 0x8031fffffffffff, 6);
grid_disk!(grid_disk_pentagon_res7, 0x8031fffffffffff, 7);
grid_disk!(grid_disk_pentagon_res8, 0x8031fffffffffff, 8);
grid_disk!(grid_disk_pentagon_res9, 0x8031fffffffffff, 9);
grid_disk!(grid_disk_pentagon_res10, 0x8031fffffffffff, 10);
grid_disk!(grid_disk_pentagon_res11, 0x8031fffffffffff, 11);
grid_disk!(grid_disk_pentagon_res12, 0x8031fffffffffff, 12);
grid_disk!(grid_disk_pentagon_res13, 0x8031fffffffffff, 13);
grid_disk!(grid_disk_pentagon_res14, 0x8031fffffffffff, 14);
grid_disk!(grid_disk_pentagon_res15, 0x8031fffffffffff, 15);

// -----------------------------------------------------------------------------

/// Returns true if two LineString are equivalent.
///
/// LineString are equivalent if they contains the same point in the same order
/// (but they don't necessarily start at the same point).
fn assert_line_string_equivalent(
    line1: &LineString,
    line2: &LineString,
    epsilon: f64,
) {
    assert!(line1.is_closed(), "line1 is a LinearRing");
    let mut coords1 = line1.coords().collect::<Vec<_>>();
    coords1.pop(); // Remove the duplicated coord that close the ring

    assert!(line2.is_closed(), "line2 is a LinearRing");
    let mut coords2 = line2.coords().collect::<Vec<_>>();
    coords2.pop(); // Remove the duplicated coord that close the ring

    assert_eq!(coords1.len(), coords2.len(), "size mismatch");
    let offset = coords2
        .iter()
        .position(|&coord| relative_eq!(coord, coords1[0], epsilon = epsilon))
        .expect("lines are different");
    for i in 0..coords1.len() {
        assert_relative_eq!(
            coords1[i],
            coords2[(i + offset) % coords2.len()],
            epsilon = epsilon
        );
    }
}

fn assert_polygon_equivalent(poly1: &Polygon, poly2: &Polygon, epsilon: f64) {
    let holes1 = poly1.interiors();
    let holes2 = poly2.interiors();

    assert_eq!(holes1.len(), holes2.len(), "holes count mismatch");
    for (hole1, hole2) in holes1.iter().zip(holes2.iter()) {
        assert_line_string_equivalent(hole1, hole2, epsilon);
    }
    assert_line_string_equivalent(poly1.exterior(), poly2.exterior(), epsilon);
}

fn assert_multipolygon_equivalent(
    mpoly1: &MultiPolygon,
    mpoly2: &MultiPolygon,
    epsilon: f64,
) {
    assert_eq!(mpoly1.0.len(), mpoly2.0.len(), "polygon count mismatch");
    for (poly1, poly2) in mpoly1.0.iter().zip(mpoly2.0.iter()) {
        assert_polygon_equivalent(poly1, poly2, epsilon);
    }
}

fn assert_hetero_equal_homo(
    cells: impl IntoIterator<Item = CellIndex>,
    resolution: Resolution,
    expected: &MultiPolygon,
) {
    let solvent = SolventBuilder::new()
        .disable_duplicate_detection()
        .enable_heterogeneous_support(resolution)
        .build();
    let result = solvent.dissolve(cells).expect("geometry");
    let union_area = result.union(expected).unsigned_area();
    let intersection_area = result.union(expected).unsigned_area();
    assert_eq!(intersection_area / union_area, 1., "geom mismatch")
}
