use super::*;
use crate::{Resolution, math::FloatAdder};
use approx::assert_relative_eq;
use geo::{CoordsIter as _, ToRadians as _};

macro_rules! world_test {
    ($name:ident, $resolution: literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("resolution");
            for base in CellIndex::base_cells() {
                for cell in base.children(resolution) {
                    assert_cell(cell);
                }
            }
        }
    };
}

world_test!(entire_world_res0, 0);
world_test!(entire_world_res1, 1);
world_test!(entire_world_res2, 2);

#[test]
fn resolution15() {
    assert_cell(CellIndex::try_from(0x8f754e64992d6d8).unwrap());
}

#[test]
fn all_pentagons() {
    for res in Resolution::range(Resolution::Zero, Resolution::Fifteen) {
        for cell in res.pentagons() {
            assert_cell(cell);
        }
    }
}

#[test]
fn three_polygons() {
    // Results in 3 polygons: 0 holes, 1 hole, 3 holes
    #[rustfmt::skip]
    let cells = [
        0x8027fffffffffff, 0x802bfffffffffff, 0x804dfffffffffff,
        0x8067fffffffffff, 0x806dfffffffffff, 0x8049fffffffffff,
        0x805ffffffffffff, 0x8057fffffffffff, 0x807dfffffffffff,
        0x80a5fffffffffff, 0x80a9fffffffffff, 0x808bfffffffffff,
        0x801bfffffffffff, 0x8035fffffffffff, 0x803ffffffffffff,
        0x8053fffffffffff, 0x8043fffffffffff, 0x8021fffffffffff,
        0x8011fffffffffff, 0x801ffffffffffff, 0x8097fffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);
    let expected_area = 2.2440497074541694;

    assert_eq!(mpoly.0.len(), 3, "3 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 3, "with 3 holes");
    assert_eq!(mpoly[1].num_interior_rings(), 1, "with 1 holes");
    assert_eq!(mpoly[2].num_interior_rings(), 0, "with 0 holes");

    let area = multipolygon_area(&mpoly);
    assert_relative_eq!(area, expected_area, epsilon = 1e-15);
}

// from https://github.com/uber/h3/issues/1049
#[test]
fn h3_issue_1049() {
    #[rustfmt::skip]
    let cells = [
        0x827487fffffffff, 0x82748ffffffffff, 0x827497fffffffff,
        0x82749ffffffffff, 0x8274affffffffff, 0x8274c7fffffffff,
        0x8274cffffffffff, 0x8274d7fffffffff, 0x8274e7fffffffff,
        0x8274effffffffff, 0x8274f7fffffffff, 0x82754ffffffffff,
        0x827c07fffffffff, 0x827c27fffffffff, 0x827c2ffffffffff,
        0x827c37fffffffff, 0x827d87fffffffff, 0x827d8ffffffffff,
        0x827d97fffffffff, 0x827d9ffffffffff, 0x827da7fffffffff,
        0x827daffffffffff, 0x82801ffffffffff, 0x8280a7fffffffff,
        0x8280affffffffff, 0x8280b7fffffffff, 0x828197fffffffff,
        0x82819ffffffffff, 0x8281a7fffffffff, 0x8281b7fffffffff,
        0x828207fffffffff, 0x82820ffffffffff, 0x828227fffffffff,
        0x82822ffffffffff, 0x8282e7fffffffff, 0x828307fffffffff,
        0x82830ffffffffff, 0x82831ffffffffff, 0x82832ffffffffff,
        0x828347fffffffff, 0x82834ffffffffff, 0x828357fffffffff,
        0x82835ffffffffff, 0x828367fffffffff, 0x828377fffffffff,
        0x82a447fffffffff, 0x82a457fffffffff, 0x82a45ffffffffff,
        0x82a467fffffffff, 0x82a46ffffffffff, 0x82a477fffffffff,
        0x82a4c7fffffffff, 0x82a4cffffffffff, 0x82a4d7fffffffff,
        0x82a4e7fffffffff, 0x82a4effffffffff, 0x82a4f7fffffffff,
        0x82a547fffffffff, 0x82a54ffffffffff, 0x82a557fffffffff,
        0x82a55ffffffffff, 0x82a567fffffffff, 0x82a577fffffffff,
        0x82a837fffffffff, 0x82a897fffffffff, 0x82a8a7fffffffff,
        0x82a8b7fffffffff, 0x82a917fffffffff, 0x82a927fffffffff,
        0x82a937fffffffff, 0x82a987fffffffff, 0x82a98ffffffffff,
        0x82a997fffffffff, 0x82a99ffffffffff, 0x82a9a7fffffffff,
        0x82a9affffffffff, 0x82ac47fffffffff, 0x82ac57fffffffff,
        0x82ac5ffffffffff, 0x82ac67fffffffff, 0x82ac6ffffffffff,
        0x82ac77fffffffff, 0x82ad47fffffffff, 0x82ad4ffffffffff,
        0x82ad57fffffffff, 0x82ad5ffffffffff, 0x82ad67fffffffff,
        0x82ad77fffffffff, 0x82c207fffffffff, 0x82c217fffffffff,
        0x82c227fffffffff, 0x82c237fffffffff, 0x82c287fffffffff,
        0x82c28ffffffffff, 0x82c29ffffffffff, 0x82c2a7fffffffff,
        0x82c2affffffffff, 0x82c2b7fffffffff, 0x82c307fffffffff,
        0x82c317fffffffff, 0x82c31ffffffffff, 0x82c337fffffffff,
        0x82cfb7fffffffff, 0x82d0c7fffffffff, 0x82d0d7fffffffff,
        0x82d0dffffffffff, 0x82d0e7fffffffff, 0x82d0f7fffffffff,
        0x82d147fffffffff, 0x82d157fffffffff, 0x82d15ffffffffff,
        0x82d167fffffffff, 0x82d177fffffffff, 0x82d187fffffffff,
        0x82d18ffffffffff, 0x82d197fffffffff, 0x82d19ffffffffff,
        0x82d1a7fffffffff, 0x82d1affffffffff, 0x82dc47fffffffff,
        0x82dc57fffffffff, 0x82dc5ffffffffff, 0x82dc67fffffffff,
        0x82dc6ffffffffff, 0x82dc77fffffffff, 0x82dcc7fffffffff,
        0x82dccffffffffff, 0x82dcd7fffffffff, 0x82dce7fffffffff,
        0x82dceffffffffff, 0x82dcf7fffffffff, 0x82dd1ffffffffff,
        0x82dd47fffffffff, 0x82dd4ffffffffff, 0x82dd57fffffffff,
        0x82dd5ffffffffff, 0x82dd6ffffffffff, 0x82dd87fffffffff,
        0x82dd8ffffffffff, 0x82dd97fffffffff, 0x82dd9ffffffffff,
        0x82ddaffffffffff, 0x82ddb7fffffffff, 0x82dec7fffffffff,
        0x82decffffffffff, 0x82ded7fffffffff, 0x82dee7fffffffff,
        0x82deeffffffffff, 0x82def7fffffffff, 0x82df0ffffffffff,
        0x82df1ffffffffff, 0x82df47fffffffff, 0x82df4ffffffffff,
        0x82df57fffffffff, 0x82df5ffffffffff, 0x82df77fffffffff,
        0x82df8ffffffffff, 0x82df9ffffffffff, 0x82e6c7fffffffff,
        0x82e6cffffffffff, 0x82e6d7fffffffff, 0x82e6dffffffffff,
        0x82e6effffffffff, 0x82e6f7fffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 12, "12 polygons");
    assert!(
        mpoly
            .iter()
            .all(|polygon| polygon.num_interior_rings() == 0),
        "with no holes"
    );
}

#[test]
fn equator_cells() {
    #[rustfmt::skip]
    let cells = [
        0x81807ffffffffff, 0x817efffffffffff, 0x81723ffffffffff,
        0x817ebffffffffff, 0x817c3ffffffffff, 0x817e3ffffffffff,
        0x817a3ffffffffff, 0x8166fffffffffff, 0x8172bffffffffff,
        0x816afffffffffff, 0x81933ffffffffff, 0x8168fffffffffff,
        0x8188fffffffffff, 0x81853ffffffffff, 0x817f7ffffffffff,
        0x8180bffffffffff, 0x81783ffffffffff, 0x81743ffffffffff,
        0x8170bffffffffff, 0x8173bffffffffff, 0x8179bffffffffff,
        0x817cbffffffffff, 0x8188bffffffffff, 0x81857ffffffffff,
        0x816f7ffffffffff, 0x8177bffffffffff, 0x81617ffffffffff,
        0x816f3ffffffffff, 0x8174bffffffffff, 0x8180fffffffffff,
        0x817a7ffffffffff, 0x81767ffffffffff, 0x81757ffffffffff,
        0x81957ffffffffff, 0x81787ffffffffff, 0x81847ffffffffff,
        0x81653ffffffffff, 0x817bbffffffffff, 0x816cfffffffffff,
        0x816abffffffffff, 0x815f3ffffffffff, 0x817c7ffffffffff,
        0x8168bffffffffff, 0x818cbffffffffff, 0x818cfffffffffff,
        0x818afffffffffff, 0x8174fffffffffff, 0x8172fffffffffff,
        0x8170fffffffffff, 0x816fbffffffffff, 0x81657ffffffffff,
        0x816c7ffffffffff, 0x8186bffffffffff, 0x81763ffffffffff,
        0x818a7ffffffffff, 0x8186fffffffffff, 0x81707ffffffffff,
        0x8182bffffffffff, 0x818f3ffffffffff, 0x8182fffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 1, "1 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 1, "with 1 holes");
}

#[test]
fn prime_meridian() {
    #[rustfmt::skip]
    let cells = [
        0x81efbffffffffff, 0x81c07ffffffffff, 0x81d1bffffffffff,
        0x81097ffffffffff, 0x8109bffffffffff, 0x81d0bffffffffff,
        0x81987ffffffffff, 0x81017ffffffffff, 0x81e67ffffffffff,
        0x81ddbffffffffff, 0x81ac7ffffffffff, 0x8158bffffffffff,
        0x81397ffffffffff, 0x81593ffffffffff, 0x81c17ffffffffff,
        0x81827ffffffffff, 0x81197ffffffffff, 0x81eebffffffffff,
        0x81383ffffffffff, 0x81dcbffffffffff, 0x81757ffffffffff,
        0x81093ffffffffff, 0x81073ffffffffff, 0x8159bffffffffff,
        0x81f17ffffffffff, 0x81187ffffffffff, 0x81007ffffffffff,
        0x81997ffffffffff, 0x81753ffffffffff, 0x81033ffffffffff,
        0x81f2bffffffffff, 0x8138bffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 1, "1 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 0, "with 0 holes");
}

#[test]
fn anti_meridian() {
    #[rustfmt::skip]
    let cells = [
        0x817ebffffffffff, 0x8133bffffffffff, 0x81047ffffffffff,
        0x81f3bffffffffff, 0x81dbbffffffffff, 0x8132bffffffffff,
        0x810cbffffffffff, 0x81bb3ffffffffff, 0x81db3ffffffffff,
        0x81bafffffffffff, 0x81177ffffffffff, 0x817fbffffffffff,
        0x81ba3ffffffffff, 0x815abffffffffff, 0x815bbffffffffff,
        0x81eafffffffffff, 0x81ed7ffffffffff, 0x81057ffffffffff,
        0x819a7ffffffffff, 0x81eabffffffffff, 0x819b7ffffffffff,
        0x81167ffffffffff, 0x81227ffffffffff, 0x8171bffffffffff,
        0x81237ffffffffff, 0x810dbffffffffff, 0x81033ffffffffff,
        0x81f2bffffffffff, 0x8147bffffffffff, 0x81f33ffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 1, "1 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 0, "with 0 holes");
}

#[test]
fn both_meridians() {
    #[rustfmt::skip]
    let cells = [
        0x81efbffffffffff, 0x81c07ffffffffff, 0x81d1bffffffffff,
        0x81097ffffffffff, 0x817ebffffffffff, 0x8133bffffffffff,
        0x81047ffffffffff, 0x8109bffffffffff, 0x81f3bffffffffff,
        0x81d0bffffffffff, 0x81987ffffffffff, 0x81dbbffffffffff,
        0x81017ffffffffff, 0x81e67ffffffffff, 0x81ddbffffffffff,
        0x8132bffffffffff, 0x810cbffffffffff, 0x81bb3ffffffffff,
        0x81ac7ffffffffff, 0x81db3ffffffffff, 0x8158bffffffffff,
        0x81397ffffffffff, 0x81593ffffffffff, 0x81bafffffffffff,
        0x81177ffffffffff, 0x817fbffffffffff, 0x81ba3ffffffffff,
        0x81c17ffffffffff, 0x815abffffffffff, 0x81827ffffffffff,
        0x815bbffffffffff, 0x81eafffffffffff, 0x81197ffffffffff,
        0x81ed7ffffffffff, 0x81eebffffffffff, 0x81383ffffffffff,
        0x81057ffffffffff, 0x819a7ffffffffff, 0x81dcbffffffffff,
        0x81757ffffffffff, 0x81eabffffffffff, 0x81093ffffffffff,
        0x819b7ffffffffff, 0x81073ffffffffff, 0x8159bffffffffff,
        0x8147bffffffffff, 0x81167ffffffffff, 0x81f17ffffffffff,
        0x8171bffffffffff, 0x81227ffffffffff, 0x81187ffffffffff,
        0x81237ffffffffff, 0x81007ffffffffff, 0x810dbffffffffff,
        0x81997ffffffffff, 0x81753ffffffffff, 0x81033ffffffffff,
        0x81f2bffffffffff, 0x8138bffffffffff, 0x81f33ffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 1, "1 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 1, "with 1 holes");
}

#[test]
fn meridians_and_equator() {
    #[rustfmt::skip]
    let cells = [
        0x817c3ffffffffff, 0x81047ffffffffff, 0x8188fffffffffff,
        0x817f7ffffffffff, 0x8180bffffffffff, 0x81177ffffffffff,
        0x817fbffffffffff, 0x8188bffffffffff, 0x815bbffffffffff,
        0x81eafffffffffff, 0x816f3ffffffffff, 0x817a7ffffffffff,
        0x819a7ffffffffff, 0x81757ffffffffff, 0x817bbffffffffff,
        0x816cfffffffffff, 0x8168bffffffffff, 0x81237ffffffffff,
        0x818afffffffffff, 0x8172fffffffffff, 0x816fbffffffffff,
        0x81657ffffffffff, 0x81763ffffffffff, 0x818a7ffffffffff,
        0x81eabffffffffff, 0x8138bffffffffff, 0x8182fffffffffff,
        0x81c07ffffffffff, 0x8109bffffffffff, 0x8166fffffffffff,
        0x81987ffffffffff, 0x8172bffffffffff, 0x8168fffffffffff,
        0x81853ffffffffff, 0x810cbffffffffff, 0x81bb3ffffffffff,
        0x81db3ffffffffff, 0x81743ffffffffff, 0x81bafffffffffff,
        0x8179bffffffffff, 0x818f3ffffffffff, 0x81857ffffffffff,
        0x816f7ffffffffff, 0x8177bffffffffff, 0x8174bffffffffff,
        0x81eebffffffffff, 0x81383ffffffffff, 0x81767ffffffffff,
        0x81787ffffffffff, 0x819b7ffffffffff, 0x8159bffffffffff,
        0x8171bffffffffff, 0x818cbffffffffff, 0x818cfffffffffff,
        0x8170fffffffffff, 0x81707ffffffffff, 0x8147bffffffffff,
        0x81167ffffffffff, 0x81f33ffffffffff, 0x817efffffffffff,
        0x81f3bffffffffff, 0x81017ffffffffff, 0x816afffffffffff,
        0x81e67ffffffffff, 0x81ddbffffffffff, 0x8132bffffffffff,
        0x8170bffffffffff, 0x81ba3ffffffffff, 0x81c17ffffffffff,
        0x815abffffffffff, 0x81617ffffffffff, 0x8180fffffffffff,
        0x81dcbffffffffff, 0x81957ffffffffff, 0x81093ffffffffff,
        0x81847ffffffffff, 0x81653ffffffffff, 0x81073ffffffffff,
        0x8174fffffffffff, 0x810dbffffffffff, 0x81997ffffffffff,
        0x816c7ffffffffff, 0x81033ffffffffff, 0x8186bffffffffff,
        0x81f2bffffffffff, 0x81efbffffffffff, 0x81807ffffffffff,
        0x81d1bffffffffff, 0x81097ffffffffff, 0x817ebffffffffff,
        0x81723ffffffffff, 0x8133bffffffffff, 0x817e3ffffffffff,
        0x817a3ffffffffff, 0x81d0bffffffffff, 0x81dbbffffffffff,
        0x81933ffffffffff, 0x81783ffffffffff, 0x81ac7ffffffffff,
        0x8158bffffffffff, 0x81397ffffffffff, 0x81593ffffffffff,
        0x8173bffffffffff, 0x817cbffffffffff, 0x81827ffffffffff,
        0x81197ffffffffff, 0x81ed7ffffffffff, 0x81057ffffffffff,
        0x816abffffffffff, 0x815f3ffffffffff, 0x81f17ffffffffff,
        0x81227ffffffffff, 0x817c7ffffffffff, 0x81007ffffffffff,
        0x81753ffffffffff, 0x8186fffffffffff, 0x8182bffffffffff,
        0x81187ffffffffff,
    ]
    .into_iter()
    .map(|value| CellIndex::try_from(value).unwrap())
    .collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 1, "1 polygon");
    assert_eq!(mpoly[0].num_interior_rings(), 3, "with 3 holes");
}

#[test]
fn empty() {
    let cells = Vec::new();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 0, "no geometry");
}

#[test]
#[expect(clippy::float_cmp, reason = "on purpose")]
fn world() {
    let cells = CellIndex::base_cells().collect::<Vec<_>>();
    let mpoly = dissolve(cells);

    assert_eq!(mpoly.0.len(), 8, "8 triangles");
    for polygon in &mpoly {
        assert_eq!(polygon.num_interior_rings(), 0, "no holes");
        // XXX: We assert 4 instead of 3 because in a Polygon the LineString is
        // always closed (first coord is duplicated at the end).
        assert_eq!(polygon.exterior().coords_count(), 4, "triangle");
    }

    let area = multipolygon_area(&mpoly);
    assert_eq!(area, 4. * PI, "exact area expected");
}

#[test]
fn duplicate_cells() {
    let cells = [
        CellIndex::try_from(0x81efbffffffffff).unwrap(),
        CellIndex::try_from(0x81efbffffffffff).unwrap(),
        CellIndex::try_from(0x81efbffffffffff).unwrap(),
    ];
    let result = ArcSet::new(cells).unwrap_err();
    assert_eq!(result, DissolutionError::DuplicateInput);
}

#[test]
fn heterogenous_resolution() {
    let cells = [
        CellIndex::try_from(0x8027fffffffffff).unwrap(),
        CellIndex::try_from(0x81efbffffffffff).unwrap(),
    ];
    let result = ArcSet::new(cells).unwrap_err();
    assert_eq!(result, DissolutionError::UnsupportedResolution);
}

// -----------------------------------------------------------------------------

fn linestring_area(ring: &LineString) -> f64 {
    let area = linear_ring_area(&ring.0);

    assert!(area >= 0., "ring area should be positive");
    assert!(area < 4. * PI, "ring area should be smaller than the world");

    // XXX: We count 4 because our rings are closed (first coord == last coord).
    assert!(ring.coords_count() >= 4, "ring should have 3+ coords");
    area
}

fn polygon_area(polygon: &Polygon) -> f64 {
    let mut adder = FloatAdder::default();
    adder += linestring_area(polygon.exterior());
    for hole in polygon.interiors() {
        adder += linestring_area(hole);
        // Due to clockwise order, holes will contribute area of "everything
        // except the hole", so adjust with -4π term.
        adder += -4. * PI;
    }
    adder.into()
}

fn multipolygon_area(mpoly: &MultiPolygon) -> f64 {
    mpoly
        .iter()
        .fold(FloatAdder::default(), |mut adder, polygon| {
            adder += polygon_area(polygon);
            adder
        })
        .into()
}

fn dissolve(cells: Vec<CellIndex>) -> MultiPolygon {
    let expected_area = cells
        .iter()
        .fold(FloatAdder::default(), |mut adder, cell| {
            adder += cell.area_rads2();
            adder
        })
        .into();

    let mut mpoly = MultiPolygon::from(ArcSet::new(cells).unwrap());
    mpoly.to_radians_in_place();

    let area = multipolygon_area(&mpoly);

    assert!(
        mpoly.iter().is_sorted_by_key(|polygon| core::cmp::Reverse(
            OrderedFloat(linestring_area(polygon.exterior()))
        )),
        "polygons should be ordered by area enclosed by outer loop, decreasing"
    );
    for polygon in &mpoly {
        assert_polygon(polygon);
    }

    // Polygon area should match the sum of the cells' area.
    assert_relative_eq!(area, expected_area, epsilon = 1e-8);

    mpoly
}

fn assert_polygon(polygon: &Polygon) {
    let area = polygon_area(polygon);
    let outer_area = linestring_area(polygon.exterior());

    assert!(area >= 0., "polygon area should be positive");
    assert!(area < 4. * PI, "polygon should be smaller than the world");
    assert!(area <= outer_area, "total area should be <= the outer loop");

    // The outer ring and holes should be ordered in 'increasing' order; that
    // is, since the holes are oriented clockwise, they will naively enclose
    // more area than the outer ring, which is oriented counterclockwise.
    if let Some(hole) = polygon.interiors().first() {
        assert!(
            outer_area <= linestring_area(hole),
            "outer loop should have 'less' area than first hole."
        );
    }

    assert!(
        polygon
            .interiors()
            .is_sorted_by_key(|hole| OrderedFloat(linestring_area(hole))),
        "polygon holes should be ordered by area, increasing"
    );
}
fn assert_cell(cell: CellIndex) {
    let mpoly = dissolve(vec![cell]);
    assert_eq!(mpoly.0.len(), 1, "exactly 1 polygon");

    assert_eq!(mpoly[0].num_interior_rings(), 0, "cell has zero holes");

    let exterior = mpoly[0].exterior();
    // XXX: +1 due to ring being closed.
    assert!(exterior.coords_count() >= 6, "at least 5 vertices");
    assert!(exterior.coords_count() <= 11, "at most 10 vertices");
}
