//! Compute average H3 cell edge length.
//!
//! Inspired from https://gist.github.com/mciethan/3e10802c1f41972831c325994d97ef27
use h3o::{CellIndex, DirectedEdgeIndex, Resolution};
use polyfit_rs::polyfit_rs::polyfit;

fn main() {
    // Compute the exact average for the resolution 0 to 6.
    let mut averages = Resolution::range(Resolution::Zero, Resolution::Six)
        .map(avg_edge_len_at_res)
        .collect::<Vec<_>>();
    // Extrapolate values for finer resolutions.
    let x = [0., 1., 2., 3., 4., 5., 6.];
    let y = averages.iter().map(|value| value.ln()).collect::<Vec<_>>();
    let coeffs = polyfit(&x, &y, 1).expect("polynomial coefficients");
    for resolution in 7..=15_u8 {
        averages.push((coeffs[1] * f64::from(resolution) + coeffs[0]).exp())
    }
    for avg in averages {
        let avg_km = avg * h3o::EARTH_RADIUS_KM;
        let avg_m = avg_km * 1000.;
        println!("{avg},{avg_km},{avg_m}");
    }
}

/// Returns every edge at the given resolution.
fn edges_at_res(
    resolution: Resolution,
) -> impl Iterator<Item = DirectedEdgeIndex> {
    CellIndex::uncompact(CellIndex::base_cells(), resolution)
        .flat_map(|cell| cell.edges())
}

/// Compute the average edge length (in radians) at the given resolution.
fn avg_edge_len_at_res(resolution: Resolution) -> f64 {
    let (count, sum) = edges_at_res(resolution)
        .fold((0, 0.), |(count, sum), edge| {
            (count + 1, sum + edge.length_rads())
        });
    sum / count as f64
}
