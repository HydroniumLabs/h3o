use crate::{CellIndex, Direction};
use ahash::{HashSet, HashSetExt};
use std::collections::VecDeque;

/// Direction to the next ring.
const NEXT_RING_DIRECTION: Direction = Direction::I;

/// Directions used for traversing an hexagonal ring counterclockwise around
/// {1, 0, 0}.
///
/// ```text
///       _
///     _/ \\_
///    / \\5/ \\
///    \\0/ \\4/
///    / \\_/ \\
///    \\1/ \\3/
///      \\2/
/// ```
const DIRECTIONS: [Direction; 6] = [
    Direction::J,
    Direction::JK,
    Direction::K,
    Direction::IK,
    Direction::I,
    Direction::IJ,
];

/// Iterator over indexes within k distance of the origin.
pub struct DiskDistancesSafe {
    /// Max distance.
    k: u32,

    /// Already visited neighbors.
    seen: HashSet<CellIndex>,
    /// Next set of neighbors to visit.
    candidates: VecDeque<(CellIndex, u32)>,
}

impl DiskDistancesSafe {
    pub fn new(origin: CellIndex, k: u32) -> Self {
        let size = usize::try_from(crate::max_grid_disk_size(k))
            .expect("grid too large");
        // Empirically found that `candidates` usually peak at ~2`k`, so
        // allocate 2.5 to be safe.
        // We got this number by tracing peak size of `candidates` within the
        // test suite.
        let mut candidates = VecDeque::with_capacity(size * 5 / 2);
        candidates.push_back((origin, 0));
        Self {
            k,
            seen: HashSet::with_capacity(size),
            candidates,
        }
    }
}

impl Iterator for DiskDistancesSafe {
    type Item = (CellIndex, u32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((cell, ring)) = self.candidates.pop_front() {
            if ring > self.k || self.seen.contains(&cell) {
                continue;
            }

            if ring < self.k {
                // Recurse to all neighbors in no particular order.
                self.candidates.extend((0..6).filter_map(|i| {
                    super::neighbor_rotations(cell, DIRECTIONS[i], 0)
                        .map(|(origin, _)| (origin, ring + 1))
                }));
            }

            self.seen.insert(cell);
            return Some((cell, ring));
        }

        // We've exhausted the candidate set, we're done.
        None
    }
}

// -----------------------------------------------------------------------------

/// Fallible, but faster, iterator over indexes within k distance of the origin.
pub struct DiskDistancesUnsafe {
    /// Starting point.
    origin: CellIndex,
    /// Max distance.
    k: u32,
    /// If the iterator is in a failed state.
    is_failed: bool,

    /// Current ring.
    ring: u32,
    /// Current side of the ring.
    side: u8,
    /// Current position on the side of the ring.
    position: u32,

    /// Number of 60 degree ccw rotations to perform on the direction
    /// (based on which faces have been crossed).
    rotations: u8,
}

impl DiskDistancesUnsafe {
    pub const fn new(origin: CellIndex, k: u32) -> Self {
        Self {
            origin,
            k,
            is_failed: false,
            ring: 0,
            side: 0,
            position: 0,
            rotations: 0,
        }
    }
}

impl Iterator for DiskDistancesUnsafe {
    type Item = Option<(CellIndex, u32)>;

    fn next(&mut self) -> Option<Self::Item> {
        // We've explored the `k` rings, we're done.
        // Also return None if we already returned Some`(None)` before (error).
        if self.is_failed || self.ring > self.k {
            return None;
        }

        if self.side == 0 && self.position == 0 {
            // 0-ring is the origin cell itself, nothing more.
            if self.ring == 0 {
                self.ring += 1;
                return Some(if self.origin.is_pentagon() {
                    // Pentagon was encountered; bail out as user doesn't want this.
                    self.is_failed = true;
                    None
                } else {
                    Some((self.origin, 0))
                });
            }

            // Move to the next ring.
            let Some((new_origin, new_rotations)) = super::neighbor_rotations(
                self.origin,
                NEXT_RING_DIRECTION,
                self.rotations,
            ) else {
                self.is_failed = true;
                return Some(None);
            };
            self.origin = new_origin;
            self.rotations = new_rotations;
        }

        // Move to the next cell.
        let Some((new_origin, new_rotations)) = super::neighbor_rotations(
            self.origin,
            DIRECTIONS[usize::from(self.side)],
            self.rotations,
        ) else {
            self.is_failed = true;
            return Some(None);
        };
        self.origin = new_origin;
        self.rotations = new_rotations;
        let distance = self.ring;

        self.position += 1;
        // Check if end of this side of the k-ring.
        if self.position == self.ring {
            self.position = 0;
            self.side += 1;
            // Check if end of this ring.
            if self.side == 6 {
                self.side = 0;
                self.ring += 1;
            }
        }

        Some(if self.origin.is_pentagon() {
            // Pentagon was encountered; bail out as user doesn't want this.
            self.is_failed = true;
            None
        } else {
            Some((self.origin, distance))
        })
    }
}

// -----------------------------------------------------------------------------

/// Iterator over indexes at exactly grid distance `k` of the origin.
pub struct RingUnsafe {
    /// Distance.
    k: u32,

    /// Number of 60 degree ccw rotations to perform on the direction (based on
    /// which faces have been crossed).
    rotations: u8,

    direction: u8,
    position: u32,

    // Start of the ring.
    origin: CellIndex,
    // Expected last index.
    last_index: CellIndex,
}

impl RingUnsafe {
    pub fn new(mut origin: CellIndex, k: u32) -> Option<Self> {
        let mut rotations = 0;

        if origin.is_pentagon() {
            // Pentagon was encountered; bail out as user doesn't want this.
            return None;
        }

        for _ in 0..k {
            (origin, rotations) = super::neighbor_rotations(
                origin,
                NEXT_RING_DIRECTION,
                rotations,
            )?;
            if origin.is_pentagon() {
                return None;
            }
        }

        Some(Self {
            k,
            rotations,
            direction: 0,
            position: 0,
            origin,
            last_index: origin,
        })
    }
}

impl Iterator for RingUnsafe {
    type Item = Option<CellIndex>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.direction == 6 {
            // Check that this matches the expected last index.
            //
            // If it doesn't, it indicates pentagonal distortion occurred and we
            // should report failure.
            if self.origin != self.last_index {
                return Some(None);
            }
            return None;
        }

        let item = self.origin;

        // Prepare the next iteration.
        (self.origin, self.rotations) = super::neighbor_rotations(
            self.origin,
            DIRECTIONS[usize::from(self.direction)],
            self.rotations,
        )
        .expect("ring neighbor");

        if self.origin.is_pentagon() {
            return Some(None);
        }

        self.position += 1;
        if self.position == self.k {
            self.position = 0;
            self.direction += 1;
        }
        Some(Some(item))
    }
}

#[cfg(test)]
#[path = "./iterator_tests.rs"]
mod tests;
