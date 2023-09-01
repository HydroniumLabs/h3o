//! Hexagon grid algorithms.

use crate::{
    coord::FaceIJK, index::bits, BaseCell, CellIndex, Direction, Resolution,
    CCW, CW,
};

// -----------------------------------------------------------------------------

/// Returns the index neighboring the origin, in the specified direction.
///
/// The only case where this returns `None` is when the origin is a pentagon and
/// the translation follow the `K` axe.
///
/// # Arguments
///
/// * `origin`    - Origin index.
/// * `direction` - Direction to move in.
/// * `rotations` - Number of ccw rotations to perform to reorient the
///                 translation vector.
///
/// # Returns
///
/// The neighboring index and an updated rotation count (happens when crossing a
/// face edge).
pub fn neighbor_rotations(
    origin: CellIndex,
    direction: Direction,
    mut rotations: u8,
) -> Option<(CellIndex, u8)> {
    // Center is not a valid direction here.
    debug_assert_ne!(direction, Direction::Center);

    let mut current = u64::from(origin);

    // Adjust the indexing digits and, if needed, the base cell.
    let mut direction = direction.rotate60::<{ CCW }>(rotations.into());
    let mut res = origin.resolution();
    for resolution in Resolution::range(Resolution::One, res).rev() {
        // SAFETY: we always have a valid direction at a valid resolution.
        let from = usize::from(bits::get_direction(current, resolution));
        let to = usize::from(direction);
        let next_direction = if resolution.is_class3() {
            let direction = NEW_DIRECTION_II[from][to];
            current =
                bits::set_direction(current, u8::from(direction), resolution);
            NEW_ADJUSTMENT_II[from][to]
        } else {
            let direction = NEW_DIRECTION_III[from][to];
            current =
                bits::set_direction(current, u8::from(direction), resolution);
            NEW_ADJUSTMENT_III[from][to]
        };

        if next_direction == Direction::Center {
            // No more adjustment to perform
            break;
        }
        direction = next_direction;
        // Safe because the lower bound of the loop is resolution 1.
        res = resolution.pred().expect("parent resolution");
    }

    let old_base_cell = origin.base_cell();
    // If `res` is 1, it means we have adjusted indexing digit all the way down.
    // (i.e. no early exit from the loop).
    let new_rotations = if res == Resolution::Zero {
        if let Some(base_cell) = old_base_cell.neighbor(direction) {
            current = h3o_bit::set_base_cell(current, base_cell.into());
            old_base_cell.neighbor_rotation(direction)
        } else {
            // Adjust for the deleted k vertex at the base cell level.
            // This edge actually borders a different neighbor.
            let base_cell = old_base_cell
                .neighbor(Direction::IK)
                .expect("pentagon neighbor");
            current = h3o_bit::set_base_cell(current, base_cell.into());

            // Perform the adjustment for the k-subsequence we're skipping
            // over.
            current = bits::rotate60::<{ CCW }>(current, 1);
            rotations += 1;

            old_base_cell.neighbor_rotation(Direction::IK)
        }
    } else {
        0
    };

    // SAFETY: cell index always contain a valid base cell.
    let new_base_cell =
        BaseCell::new_unchecked(h3o_bit::get_base_cell(current));
    if new_base_cell.is_pentagon() {
        let mut already_adjusted_k_subsequence = false;

        // Force rotation out of missing k-axes sub-sequence.
        if bits::first_axe(current) == Direction::K.axe() {
            if old_base_cell == new_base_cell {
                let old_leading_dir = bits::first_axe(origin.into());

                // In this case, we traversed into the deleted k subsequence
                // from within the same pentagon base cell.
                //
                // Propagate the None because it's undefined: the k direction is
                // deleted from here.
                old_leading_dir?;

                if old_leading_dir == Direction::JK.axe() {
                    // Rotate out of the deleted K subsequence.
                    // We also need an additional change to the direction we're
                    // moving in.
                    current = bits::rotate60::<{ CCW }>(current, 1);
                    rotations += 1;
                } else if old_leading_dir == Direction::IK.axe() {
                    // Rotate out of the deleted K subsequence.
                    // We also need an additional change to the direction we're
                    // moving in.
                    current = bits::rotate60::<{ CW }>(current, 1);
                    rotations += 5;
                }
            } else {
                // In this case, we traversed into the deleted k subsequence of
                // a pentagon base cell.
                // We need to rotate out of that case depending on how we got
                // here.
                // Check for a CW/CCW offset face; default is CCW.

                if new_base_cell.is_cw_offset(FaceIJK::from(old_base_cell).face)
                {
                    current = bits::rotate60::<{ CW }>(current, 1);
                }
                already_adjusted_k_subsequence = true;
            }
        }

        current = (0..new_rotations)
            .fold(current, |acc, _| bits::pentagon_rotate60::<{ CCW }>(acc));

        // Account for differing orientation of the base cells (this edge might
        // not follow properties of some other edges).
        if old_base_cell != new_base_cell {
            let direction = bits::first_axe(current);

            if new_base_cell.is_polar_pentagon() {
                // 'polar' base cells behave differently because they have all
                // `I` neighbors.
                rotations += u8::from(
                    u8::from(old_base_cell) != 118
                        && u8::from(old_base_cell) != 8
                        && direction != Direction::JK.axe(),
                );
            } else {
                // Account for distortion introduced to the 5 neighbor by the
                // deleted `K` subsequence.
                rotations += u8::from(
                    !already_adjusted_k_subsequence
                        && direction == Direction::IK.axe(),
                );
            }
        }
    } else if new_rotations != 0 {
        current = bits::rotate60::<{ CCW }>(current, new_rotations.into());
    }

    Some((
        CellIndex::new_unchecked(current),
        (rotations + new_rotations) % 6,
    ))
}

// -----------------------------------------------------------------------------

/// Finds and returns the direction from the origin to a given neighbor.
///
/// This is effectively the reverse operation for `neighbor_rotations`.
///
/// Returns `None` if the cells are not neighbors.
///
/// **TODO**: This is currently a brute-force algorithm, but as it's `O(6)`
/// that's probably acceptable.
pub fn direction_for_neighbor(
    origin: CellIndex,
    destination: CellIndex,
) -> Option<Direction> {
    // Skips center since that would be the origin.
    // Skips deleted `K` direction for pentagons.
    let start = 1 + u8::from(origin.is_pentagon());

    // Checks each neighbor, in order, to determine which direction the
    // destination neighbor is located.
    (start..=6).find_map(|value| {
        // SAFETY: loop upper bound is 6 (a.k.a. `Direction::IJ`), which is a
        // valid direction.
        let direction = Direction::new_unchecked(value);
        let neighbor =
            neighbor_rotations(origin, direction, 0).map(|result| result.0);

        (neighbor == Some(destination)).then_some(direction)
    })
}

// -----------------------------------------------------------------------------

// Consts to save some typing below...
const CENTER: Direction = Direction::Center;
const K_AXE: Direction = Direction::K;
const J_AXE: Direction = Direction::J;
const JK_AXE: Direction = Direction::JK;
const I_AXE: Direction = Direction::I;
const IK_AXE: Direction = Direction::IK;
const IJ_AXE: Direction = Direction::IJ;

/// New digit when traversing along class II grids.
///
/// Current digit -> direction -> new digit.
const NEW_DIRECTION_II: [[Direction; 7]; 7] = [
    [CENTER, K_AXE, J_AXE, JK_AXE, I_AXE, IK_AXE, IJ_AXE],
    [K_AXE, I_AXE, JK_AXE, IJ_AXE, IK_AXE, J_AXE, CENTER],
    [J_AXE, JK_AXE, K_AXE, I_AXE, IJ_AXE, CENTER, IK_AXE],
    [JK_AXE, IJ_AXE, I_AXE, IK_AXE, CENTER, K_AXE, J_AXE],
    [I_AXE, IK_AXE, IJ_AXE, CENTER, J_AXE, JK_AXE, K_AXE],
    [IK_AXE, J_AXE, CENTER, K_AXE, JK_AXE, IJ_AXE, I_AXE],
    [IJ_AXE, CENTER, IK_AXE, J_AXE, K_AXE, I_AXE, JK_AXE],
];

/// New traversal direction when traversing along class II grids.
///
/// Current digit -> direction -> new ap7 move (at coarser level).
const NEW_ADJUSTMENT_II: [[Direction; 7]; 7] = [
    [CENTER, CENTER, CENTER, CENTER, CENTER, CENTER, CENTER],
    [CENTER, K_AXE, CENTER, K_AXE, CENTER, IK_AXE, CENTER],
    [CENTER, CENTER, J_AXE, JK_AXE, CENTER, CENTER, J_AXE],
    [CENTER, K_AXE, JK_AXE, JK_AXE, CENTER, CENTER, CENTER],
    [CENTER, CENTER, CENTER, CENTER, I_AXE, I_AXE, IJ_AXE],
    [CENTER, IK_AXE, CENTER, CENTER, I_AXE, IK_AXE, CENTER],
    [CENTER, CENTER, J_AXE, CENTER, IJ_AXE, CENTER, IJ_AXE],
];

/// New traversal direction when traversing along class III grids.
///
/// Current digit -> direction -> new ap7 move (at coarser level).
///
const NEW_DIRECTION_III: [[Direction; 7]; 7] = [
    [CENTER, K_AXE, J_AXE, JK_AXE, I_AXE, IK_AXE, IJ_AXE],
    [K_AXE, J_AXE, JK_AXE, I_AXE, IK_AXE, IJ_AXE, CENTER],
    [J_AXE, JK_AXE, I_AXE, IK_AXE, IJ_AXE, CENTER, K_AXE],
    [JK_AXE, I_AXE, IK_AXE, IJ_AXE, CENTER, K_AXE, J_AXE],
    [I_AXE, IK_AXE, IJ_AXE, CENTER, K_AXE, J_AXE, JK_AXE],
    [IK_AXE, IJ_AXE, CENTER, K_AXE, J_AXE, JK_AXE, I_AXE],
    [IJ_AXE, CENTER, K_AXE, J_AXE, JK_AXE, I_AXE, IK_AXE],
];

/// New traversal direction when traversing along class III grids.
///
/// Current digit -> direction -> new ap7 move (at coarser level).
const NEW_ADJUSTMENT_III: [[Direction; 7]; 7] = [
    [CENTER, CENTER, CENTER, CENTER, CENTER, CENTER, CENTER],
    [CENTER, K_AXE, CENTER, JK_AXE, CENTER, K_AXE, CENTER],
    [CENTER, CENTER, J_AXE, J_AXE, CENTER, CENTER, IJ_AXE],
    [CENTER, JK_AXE, J_AXE, JK_AXE, CENTER, CENTER, CENTER],
    [CENTER, CENTER, CENTER, CENTER, I_AXE, IK_AXE, I_AXE],
    [CENTER, K_AXE, CENTER, CENTER, IK_AXE, IK_AXE, CENTER],
    [CENTER, CENTER, IJ_AXE, CENTER, I_AXE, CENTER, IJ_AXE],
];
