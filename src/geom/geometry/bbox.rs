use crate::{error::InvalidGeometry, LatLng, Resolution};
use geo::{coord, LineString, Rect};
use std::f64::consts::PI;

/// Create a bounding box from a polygon's ring.
///
/// # Known limitations:
/// - Does not support polygons with two adjacent points > 180 degrees of
///   longitude apart. These will be interpreted as crossing the antimeridian.
/// - Does not currently support polygons containing a pole.
pub fn compute_from_ring(
    ring: &LineString<f64>,
) -> Result<Rect, InvalidGeometry> {
    // Closed ring have at least 4 coordinate (e.g. triangle).
    if ring.0.len() < 4 {
        return Err(InvalidGeometry::new(
            "invalid ring (not enough coordinate)",
        ));
    }

    // Check for arcs > 180 degrees longitude, flagging as transmeridian.
    let is_transmeridian = ring
        .lines()
        .any(|line| (line.start.x - line.end.x).abs() > PI);

    let mut lng_range = (f64::MAX, f64::MIN);
    let mut lat_range = (f64::MAX, f64::MIN);

    for curr in ring {
        if !super::coord_is_valid(*curr) {
            return Err(InvalidGeometry::new(
                "invalid coordinate (e.g. infinite)",
            ));
        }

        let lng = (f64::from(u8::from(is_transmeridian && curr.x < 0.)) * 2.)
            .mul_add(PI, curr.x);
        lng_range = get_min_max(lng, lng_range.0, lng_range.1);
        lat_range = get_min_max(curr.y, lat_range.0, lat_range.1);
    }

    assert!(ring.is_closed());

    Ok(Rect::new(
        coord! {
            x: lng_range.0,
            y: lat_range.0,
        },
        coord! {
            x: lng_range.1,
            y: lat_range.1,
        },
    ))
}

/// Returns an estimated number of hexagons that fit within the
/// cartesian-projected bounding box.
pub fn hex_estimate(bbox: &Rect, resolution: Resolution) -> usize {
    // Area of a regular hexagon is `3/2*sqrt(3) * r * r`.
    //
    // The pentagon has the most distortion (smallest edges) and shares its
    // edges with hexagons, so the most-distorted hexagons have this area,
    // shrunk by 20% off chance that the bounding box perfectly bounds a
    // pentagon.
    const PENT_AREA_KM2: [f64; 16] = [
        2_234_512.861765512,
        258_086.57450412086,
        39_275.57632506273,
        4_924.482850651414,
        783.7641229713141,
        99.53320954786122,
        15.943754243628701,
        2.0284861954074076,
        0.32523296444385297,
        0.04138950889941268,
        0.006636970848428596,
        0.0008446600428891398,
        0.00013544711204373916,
        0.000017237890705367126,
        0.0000027642230944745357,
        0.00000035179349216609857,
    ];
    let pentagon_area_km2 = PENT_AREA_KM2[usize::from(resolution)];

    let min = bbox.min();
    let max = bbox.max();
    let p1 =
        LatLng::from_radians(min.y, min.x).expect("finite bbox-min coordinate");
    let p2 =
        LatLng::from_radians(max.y, max.x).expect("finite bbox-max coordinate");
    let diagonal = p1.distance_km(p2);
    let d1 = (p1.lng_radians() - p2.lng_radians()).abs();
    let d2 = (p1.lat_radians() - p2.lat_radians()).abs();
    let (width, length) = if d1 < d2 { (d1, d2) } else { (d2, d1) };
    // Derived constant based on: https://math.stackexchange.com/a/1921940
    // Clamped to 3 as higher values tend to rapidly drag the estimate to zero.
    #[allow(clippy::suspicious_operation_groupings)] // False positive.
    let area = (diagonal * diagonal) / (length / width);

    // Divide the two to get an estimate of the number of hexagons needed.
    let estimate = (area / pentagon_area_km2).ceil();

    // Truncate on purpose.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let estimate = estimate as usize;

    std::cmp::max(estimate, 1)
}

fn get_min_max(value: f64, min: f64, max: f64) -> (f64, f64) {
    if value > max {
        (min, value)
    } else if value < min {
        (value, max)
    } else {
        (min, max)
    }
}
