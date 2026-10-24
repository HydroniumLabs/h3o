use float_eq::assert_float_eq;
use h3o::{CellIndex, LatLng, LatLngIndexer, Resolution};
use std::f64::consts::{FRAC_PI_2, PI};

const EPSILON: f64 = 1e-9 * PI / 180.0;

#[test]
fn nan() {
    let ll = LatLng::new(f64::NAN, 10.);
    assert!(ll.is_err(), "NaN latitude");

    let ll = LatLng::new(10., f64::NAN);
    assert!(ll.is_err(), "NaN longitude");
}

#[test]
fn infinity() {
    let ll = LatLng::new(f64::INFINITY, 10.);
    assert!(ll.is_err(), "infinite latitude");

    let ll = LatLng::new(10., f64::NEG_INFINITY);
    assert!(ll.is_err(), "infinite longitude");
}

#[test]
fn display() {
    let ll = LatLng::new(2.4, 8.2).expect("ll");
    let result = ll.to_string();
    let expected = "(2.4000000000, 8.2000000000)".to_owned();

    assert_eq!(result, expected);
}

#[test]
fn debug() {
    let ll = LatLng::new(2.4, 8.2).expect("ll");
    let result = format!("{ll:?}");

    assert!(result.contains("2.4"));
    assert!(result.contains("0.04188"));
    assert!(result.contains("8.2"));
    assert!(result.contains("0.14311"));
}

#[test]
fn lat_lng() {
    let lat = 2.349014;
    let lng = 48.864716;
    let ll = LatLng::new(lat, lng).expect("ll");

    assert_eq!(ll.lat_radians(), lat.to_radians(), "lat radians");
    assert_eq!(ll.lng_radians(), lng.to_radians(), "lng radians");

    assert_eq!(ll.lat(), lat, "lat degrees");
    assert_eq!(ll.lng(), lng, "lng degrees");
}

#[test]
fn distance_rads() {
    let p1 = LatLng::new(10., 10.).expect("p1");
    let p2 = LatLng::new(0., 10.).expect("p2");

    assert_float_eq!(
        p1.distance_rads(p1),
        0.,
        abs <= EPSILON,
        "0 distance as expected"
    );

    assert_float_eq!(
        p1.distance_rads(p2),
        10.0_f64.to_radians(),
        abs <= EPSILON,
        "distance along longitude as expected"
    );
}

#[test]
fn distance_wrapped_longitude() {
    let negative_lng = LatLng::from_radians(0., -(PI + FRAC_PI_2)).expect("ll");
    let zero = LatLng::default();

    assert_float_eq!(
        negative_lng.distance_rads(zero),
        FRAC_PI_2,
        abs <= EPSILON,
        "distance with wrapped longitude"
    );
    assert_float_eq!(
        zero.distance_rads(negative_lng),
        FRAC_PI_2,
        abs <= EPSILON,
        "distance with wrapped longitude, swapped arguments"
    );
}

#[test]
fn to_cell_icosahedron_center() {
    let ll = LatLng::new(28.173218757257807, 23.03222744086644).expect("ll");
    let result = ll.to_cell(Resolution::Three);
    let expected = CellIndex::try_from(0x833e00fffffffff).expect("cell");

    assert_eq!(result, expected);
}

#[cfg(feature = "geo")]
#[test]
fn latlng_from_geo_coord() {
    let coord = geo::geometry::Coord {
        x: 23.03222744086644,
        y: 28.173218757257807,
    };
    // Geo coord are interpreted as degrees.
    let expected =
        LatLng::new(28.173218757257807, 23.03222744086644).expect("ll");
    let result = LatLng::try_from(coord).expect("ll from coord");

    assert_eq!(result, expected);
}

#[cfg(feature = "typed_floats")]
#[test]
fn latlng_from_typed_floats() {
    let lat = typed_floats::NonNaNFinite::<f64>::new(48.864716).unwrap();
    let lng = typed_floats::NonNaNFinite::<f64>::new(2.349014).unwrap();

    let _ = LatLng::from((lat, lng));
}

#[cfg(feature = "geo")]
#[test]
fn geo_traits_support() {
    use geo_traits::CoordTrait as _;

    // LatLng can be built from a type that implements CoordTrait.
    let coord = (-45., 10.);
    let ll = LatLng::from_coord(&coord).unwrap();

    // LatLng implements the CoordTrait.
    assert_eq!(ll.dim(), geo_traits::Dimensions::Xy);
    assert_eq!(ll.x(), ll.lng());
    assert_eq!(ll.y(), ll.lat());
    assert_eq!(ll.nth(0).unwrap(), ll.lng());
    assert_eq!(ll.nth(1).unwrap(), ll.lat());

    // Example from https://github.com/HydroniumLabs/h3o/issues/45
    let _c = LatLng::from_coord(&coord).unwrap().to_cell(Resolution::One);
}

#[cfg(feature = "geo")]
#[test]
#[should_panic]
fn geo_traits_support_panic_on_invalid_access() {
    use geo_traits::CoordTrait as _;

    let ll = LatLng::from_coord(&(-45., 10.)).unwrap();
    ll.nth_or_panic(42);
}

#[test]
fn indexer_to_cells_matches_to_cell() {
    // Icosahedron face centers (projection origin), pentagon centers, poles
    // and the antimeridian, followed by a deterministic scatter.
    let mut points = vec![
        LatLng::from_radians(0.803_582_649_718_99, 1.248_397_419_617_396_1),
        LatLng::from_radians(-0.803_582_649_718_99, -1.893_195_233_972_397),
        LatLng::from_radians(FRAC_PI_2, 0.),
        LatLng::from_radians(-FRAC_PI_2, 0.),
        LatLng::new(0., 180.),
        LatLng::new(0., -180.),
    ];
    points.extend(
        CellIndex::base_cells()
            .filter(|cell| cell.is_pentagon())
            .map(|cell| Ok(LatLng::from(cell))),
    );
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1_u64 << 53) as f64
    };
    for _ in 0..20_000 {
        let lat = (2.0 * next() - 1.0).asin().to_degrees();
        let lng = 360.0 * next() - 180.0;
        points.push(LatLng::new(lat, lng));
    }

    let ascending = Resolution::range(Resolution::Zero, Resolution::Fifteen)
        .collect::<Vec<_>>();
    let mixed = [
        Resolution::Ten,
        Resolution::Three,
        Resolution::Ten,
        Resolution::Nine,
        Resolution::Zero,
        Resolution::Fifteen,
    ];
    for ll in points {
        let ll = ll.expect("valid coordinate");
        for resolutions in [&ascending[..], &mixed[..], &[]] {
            let expected = resolutions
                .iter()
                .map(|&res| ll.to_cell(res))
                .collect::<Vec<_>>();
            // A fresh indexer only shares work between resolutions.
            let result = LatLngIndexer::new()
                .to_cells(ll, resolutions.iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(result, expected, "{ll}");
        }
    }
}

#[test]
fn indexer_scanline_matches_to_cell() {
    // Scanlines of ~30 m steps crossing many cells, with jumps between rows
    // (including across faces, near a pentagon and over the antimeridian).
    let origins = [
        (39.7, -105.0),
        (64.7, 10.536),
        (0.0, 179.9),
        (-45.0, 0.0),
        (89.99, 30.0),
        (46.0425, 71.52),
    ];
    let resolutions = [
        Resolution::Six,
        Resolution::Nine,
        Resolution::Ten,
        Resolution::Fifteen,
    ];
    let mut indexer = LatLngIndexer::new();
    // Per resolution: consecutive locations in the same / a different cell.
    let mut same = [0; 4];
    let mut changed = [0; 4];
    for (row, &(lat0, lng0)) in origins.iter().cycle().take(24).enumerate() {
        let lat = lat0 + row as f64 * 2.7e-4;
        let mut previous = None;
        for col in 0..3000 {
            let ll = LatLng::new(lat, lng0 + f64::from(col) * 3.5e-4)
                .expect("valid coordinate");
            let expected = resolutions
                .iter()
                .map(|&res| ll.to_cell(res))
                .collect::<Vec<_>>();
            let result = if col % 2 == 0 {
                indexer.to_cells(ll, resolutions).collect::<Vec<_>>()
            } else {
                resolutions
                    .iter()
                    .map(|&res| indexer.to_cell(ll, res))
                    .collect()
            };
            assert_eq!(result, expected, "{ll}");
            if let Some(previous) = &previous {
                for (i, (cell, last)) in
                    expected.iter().zip(previous).enumerate()
                {
                    if cell == last {
                        same[i] += 1;
                    } else {
                        changed[i] += 1;
                    }
                }
            }
            previous = Some(expected);
        }
    }
    // Make sure both the remembered and the encoding paths ran.
    assert!(same.iter().chain(&changed).all(|&count| count > 0));
}
