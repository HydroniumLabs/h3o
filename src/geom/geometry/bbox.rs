use crate::{error::InvalidGeometry, LatLng, Resolution};
use geo::{coord, LineString, Rect};

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

    let mut lng_range = (f64::MAX, f64::MIN);
    let mut lat_range = (f64::MAX, f64::MIN);

    for curr in ring {
        if !super::coord_is_valid(*curr) {
            return Err(InvalidGeometry::new(
                "invalid coordinate (e.g. infinite)",
            ));
        }

        lng_range = get_min_max(curr.x, lng_range.0, lng_range.1);
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
    const PENT_AREA_RADS2: [f64; 16] = [
        0.05505118472518226,
        0.006358420186890303,
        0.0009676234334810151,
        0.00012132336301389888,
        0.000019309418286620768,
        0.0000024521770265310696,
        0.0000003928026439666205,
        0.00000004997535264470275,
        0.000000008012690511075445,
        0.0000000010197039091132572,
        0.00000000016351353999538285,
        0.000000000020809697203105007,
        0.000000000003336979666606075,
        0.0000000000004246859893033221,
        0.00000000000006810153522091642,
        0.000000000000008667056198238203,
    ];
    let pentagon_area_rads2 = PENT_AREA_RADS2[usize::from(resolution)];

    let min = bbox.min();
    let max = bbox.max();
    let p1 =
        LatLng::from_radians(min.y, min.x).expect("finite bbox-min coordinate");
    let p2 =
        LatLng::from_radians(max.y, max.x).expect("finite bbox-max coordinate");
    let diagonal = p1.distance_rads(p2);
    let d1 = (p1.lng_radians() - p2.lng_radians()).abs();
    let d2 = (p1.lat_radians() - p2.lat_radians()).abs();
    let (width, length) = if d1 < d2 { (d1, d2) } else { (d2, d1) };
    // Derived constant based on: https://math.stackexchange.com/a/1921940
    // Clamped to 3 as higher values tend to rapidly drag the estimate to zero.
    #[allow(clippy::suspicious_operation_groupings)] // False positive.
    let area = (diagonal * diagonal) / (length / width);

    // Divide the two to get an estimate of the number of hexagons needed.
    let estimate = (area / pentagon_area_rads2).ceil();

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
