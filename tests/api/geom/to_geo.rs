use approx::{assert_relative_eq, relative_eq};
use geo::{coord, point, polygon};
use h3o::{geom::ToGeo, CellIndex, DirectedEdgeIndex, Resolution, VertexIndex};

#[test]
fn from_cells() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("index");
    let mut cells = index.children(Resolution::Ten).collect::<Vec<_>>();

    // Remove the center cell.
    let center = index.center_child(Resolution::Ten).expect("center");
    let idx = cells.iter().position(|x| *x == center).expect("idx");
    cells.remove(idx);

    let json = r#"
        {
  "coordinates": [
    [
      [
        [ -122.02796455348616, 37.38525281604115 ],
        [ -122.02732437374608, 37.385758270113065 ],
        [ -122.02648011977477, 37.38558967035685 ],
        [ -122.02583992481574, 37.38609511818443 ],
        [ -122.02604398797318, 37.386769168218684 ],
        [ -122.02540378194031, 37.38727461225182 ],
        [ -122.02560784515092, 37.38794865717242 ],
        [ -122.02645212137664, 37.38811725429045 ],
        [ -122.02665619162275, 37.38879129032762 ],
        [ -122.02750047073862, 37.38895987611164 ],
        [ -122.02814066848063, 37.38845442717775 ],
        [ -122.02898493935817, 37.38862300294707 ],
        [ -122.0296251218798, 37.38811754776844 ],
        [ -122.02942103767036, 37.38744351927073 ],
        [ -122.03006120911812, 37.38693806029814 ],
        [ -122.02985712496266, 37.386264026686845 ],
        [ -122.0290128763404, 37.38609544827806 ],
        [ -122.02880879921976, 37.38542140578344 ],
        [ -122.02796455348616, 37.38525281604115 ]
      ],
      [
        [ -122.02752844388534, 37.386432316377665 ],
        [ -122.02837270074619, 37.38660090480038 ],
        [ -122.02857677792056, 37.38727494218174 ],
        [ -122.0279365912526, 37.38778039491016 ],
        [ -122.02709232326434, 37.387611807806856 ],
        [ -122.0268882530716, 37.386937766655734 ],
        [ -122.02752844388534, 37.386432316377665 ]
      ]
    ]
  ],
  "type": "MultiPolygon"
}
"#;
    let result =
        geo::MultiPolygon::try_from(cells.to_geojson().expect("geojson"))
            .expect("result");
    let expected = geo::MultiPolygon::try_from(
        json.parse::<geojson::Geometry>().expect("geojson"),
    )
    .expect("expected");

    assert_eq!(result.0.len(), expected.0.len(), "polygon count mismatch");

    let holes_result = result.0[0].interiors();
    let holes_expected = expected.0[0].interiors();

    assert_eq!(
        holes_expected.len(),
        holes_expected.len(),
        "holes count mismatch"
    );
    // Check equivalence due to hashing being used internally, starting point of
    // each ring isn't deterministic.
    for (hole_result, hole_expected) in
        holes_result.into_iter().zip(holes_expected.into_iter())
    {
        assert_line_string_equivalent(&hole_result, &hole_expected, 1e-6);
    }
    assert_line_string_equivalent(
        result.0[0].exterior(),
        expected.0[0].exterior(),
        1e-6,
    );
}

#[test]
fn from_cell_as_degrees() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("index");
    let json = r#"
        {
  "coordinates": [
    [
      [ -122.02648011977477, 37.38558967035685 ],
      [ -122.02540378194031, 37.38727461225182 ],
      [ -122.02665619162275, 37.38879129032762 ],
      [ -122.02898493935817, 37.38862300294707 ],
      [ -122.03006120911812, 37.38693806029814 ],
      [ -122.02880879921976, 37.38542140578344 ],
      [ -122.02648011977477, 37.38558967035685 ]
    ]
  ],
  "type": "Polygon"
}
"#;
    let result = geo::Geometry::try_from(index.to_geojson().expect("geojson"))
        .expect("result");
    let expected = geo::Geometry::try_from(
        json.parse::<geojson::Geometry>().expect("geojson"),
    )
    .expect("expected");

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_cell_as_radians() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("index");
    let result = index.to_geom(false).expect("polygon");
    let expected = polygon![
        (x: -122.02648011977477_f64.to_radians(), y: 37.38558967035685_f64.to_radians()),
        (x: -122.02540378194031_f64.to_radians(), y: 37.38727461225182_f64.to_radians()),
        (x: -122.02665619162275_f64.to_radians(), y: 37.38879129032762_f64.to_radians()),
        (x: -122.02898493935817_f64.to_radians(), y: 37.38862300294707_f64.to_radians()),
        (x: -122.03006120911812_f64.to_radians(), y: 37.38693806029814_f64.to_radians()),
        (x: -122.02880879921976_f64.to_radians(), y: 37.38542140578344_f64.to_radians()),
    ];

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_directed_edge_as_degrees() {
    let index =
        DirectedEdgeIndex::try_from(0x13a1_94e6_99ab_7fff).expect("index");
    let json = r#"
        {
  "coordinates": [
    [ 0.004346277485193205, 51.5333297602599 ],
    [ 0.005128094944356792, 51.53286048728922 ]
  ],
  "type": "LineString"
}
"#;
    let result = geo::Geometry::try_from(index.to_geojson().expect("geojson"))
        .expect("result");
    let expected = geo::Geometry::try_from(
        json.parse::<geojson::Geometry>().expect("geojson"),
    )
    .expect("expected");

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_directed_edge_as_radians() {
    let index =
        DirectedEdgeIndex::try_from(0x13a1_94e6_99ab_7fff).expect("index");
    let result = index.to_geom(false).expect("line");
    let expected = geo::Line {
        start: coord! {
            x: 0.004346277485193205_f64.to_radians(),
            y: 51.5333297602599_f64.to_radians(),
        },
        end: coord! {
            x: 0.005128094944356792_f64.to_radians(),
            y: 51.53286048728922_f64.to_radians(),
        },
    };

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_vertex_as_degrees() {
    let index = VertexIndex::try_from(0x2302_bfff_ffff_ffff).expect("index");
    let json = r#"
        {
  "coordinates": [
    -74.64046816708004,
    30.219492199828117
  ],
  "type": "Point"
}
"#;
    let result = geo::Geometry::try_from(index.to_geojson().expect("geojson"))
        .expect("result");
    let expected = geo::Geometry::try_from(
        json.parse::<geojson::Geometry>().expect("geojson"),
    )
    .expect("expected");

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

#[test]
fn from_vertex_as_radians() {
    let index = VertexIndex::try_from(0x2302_bfff_ffff_ffff).expect("index");
    let result = index.to_geom(false).expect("point");
    let expected = point! {
        x: -74.64046816708004_f64.to_radians(),
        y:  30.219492199828117_f64.to_radians()
    };

    assert_relative_eq!(result, expected, epsilon = 1e-6);
}

// -----------------------------------------------------------------------------

#[test]
fn duplicate() {
    #[rustfmt::skip]
    let set = [
        0x89283082813ffff, 0x89283082817ffff, 0x89283082813ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false);

    // No shape.
    assert!(result.is_err())
}

#[test]
fn consecutive_duplicate() {
    #[rustfmt::skip]
    let set = [
        0x89283082813ffff, 0x89283082817ffff, 0x89283082817ffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false);

    // No shape.
    assert!(result.is_err())
}

#[test]
fn resolution_mismatch() {
    let set = [0x89283082813ffff, 0x8828308281fffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false);

    // No shape.
    assert!(result.is_err())
}

#[test]
fn empty() {
    let result = std::iter::empty().to_geom(false).expect("geometry");

    // No shape.
    assert!(result.0.is_empty())
}

#[test]
fn hexagon() {
    //  __
    // /  \
    // \__/
    let set = [CellIndex::try_from(0x890dab6220bffff).expect("cell index")];
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 6-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
}

#[test]
fn two_contiguous_cells() {
    //     __
    //  __/  \
    // /  \__/
    // \__/
    let set = [0x8928308291bffff, 0x89283082957ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (6 + 6 - 2 shared edges).
    assert_eq!(result.0[0].exterior().lines().count(), 10);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
}

#[test]
fn two_non_contiguous_cells() {
    //  __      __
    // /  \    /  \
    // \__/    \__/
    let set = [0x8928308291bffff, 0x89283082943ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 2 polygon.
    assert_eq!(result.0.len(), 2);
    // Both with a 6-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6);
    assert_eq!(result.0[1].exterior().lines().count(), 6);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
    assert!(result.0[1].interiors().is_empty());
}

#[test]
fn three_contiguous_cells() {
    //     __
    //  __/  \
    // /  \__/
    // \__/  \
    //    \__/
    #[rustfmt::skip]
    let set = [
        0x8928308288bffff, 0x892830828d7ffff, 0x8928308289bffff,
    ]
    .into_iter()
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (3*6 - 3*2 shared edges).
    assert_eq!(result.0[0].exterior().lines().count(), 12);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 18-edge outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 18);
    // And one 6-edge hole.
    assert_eq!(result.0[0].interiors().len(), 1);
    assert_eq!(result.0[0].interiors()[0].lines().count(), 6);
}

#[test]
fn pentagon() {
    let set = [CellIndex::try_from(0x851c0003fffffff).expect("cell index")];
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With a 10-edge outer ring (distorted pentagon).
    assert_eq!(result.0[0].exterior().lines().count(), 10);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With the expected number of edges on the outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6 * (2 * 2 + 1));
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    // With the expected number of edges on the outer ring.
    assert_eq!(result.0[0].exterior().lines().count(), 6 * (2 * 2 + 1));
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

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
}

#[test]
fn contiguous_distorted() {
    let set = [0x894cc5365afffff, 0x894cc536537ffff]
        .into_iter()
        .map(|bits| CellIndex::try_from(bits).expect("cell index"));
    let result = set.to_geom(false).expect("geometry");

    // 1 polygon.
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].exterior().lines().count(), 12);
    // And no hole.
    assert!(result.0[0].interiors().is_empty());
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
    .map(|bits| CellIndex::try_from(bits).expect("cell index"));

    assert!(set.to_geom(false).is_ok());
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
            let result = set.to_geom(false).expect("geometry");
            // Account for pentagon distortion on class II resolution.
            let expected = if base_cell.is_pentagon() {
                if resolution.is_class3() {
                    30
                } else {
                    25
                }
            } else {
                30
            };

            // 1 polygon.
            assert_eq!(result.0.len(), 1);
            assert_eq!(result.0[0].exterior().lines().count(), expected);
            // And no hole.
            assert!(result.0[0].interiors().is_empty());
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
    line1: &geo::LineString,
    line2: &geo::LineString,
    epsilon: f64,
) {
    assert!(line1.is_closed(), "line1 is a LinearRing");
    let mut coords1 = line1.coords().collect::<Vec<_>>();
    coords1.pop(); // Remove the duplicated coord that close the ring
                   //
    assert!(line2.is_closed(), "line2 is a LinearRing");
    let mut coords2 = line2.coords().collect::<Vec<_>>();
    coords2.pop(); // Remove the duplicated coord that close the ring
                   //
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
