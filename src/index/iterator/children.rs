use crate::{CellIndex, Direction, Resolution, index::bits};
use core::iter::FusedIterator;

/// Iterator over a children cell index at a given resolution.
pub struct Children {
    /// Starting cell resolution.
    parent_resolution: Resolution,
    /// Targeted cell resolution.
    target_resolution: Resolution,

    /// Iterator scratch space, used to build cell index iteratively
    scratchpad: u64,
    /// Number of cell index to skip for pentagonal index, there is one per
    /// resolution.
    skip_count: i16,
    /// Remaining children at the targeted resolution.
    count: u64,
}

impl Children {
    /// Returns an iterator over the children cell index at the given
    /// resolution.
    pub fn new(index: CellIndex, resolution: Resolution) -> Self {
        Self {
            parent_resolution: index.resolution(),
            target_resolution: resolution,
            scratchpad: get_starting_state(index, resolution),
            skip_count: if index.is_pentagon() {
                i16::from(u8::from(resolution))
            } else {
                -1
            },
            count: index.children_count(resolution),
        }
    }

    /// Increment the direction at `resolution` and return it.
    fn next_direction(&mut self, resolution: Resolution) -> u8 {
        // Shift the 1 to apply it on the right direction.
        let one = 1 << resolution.direction_offset();

        // Add one to the direction.
        //
        // Note that if the direction was 7 (unused) this automatically reset
        // the direction to [`Direction::CENTER`] (wraparound) AND increment the
        // direction of lower resolution (thanks to carry propagation).
        self.scratchpad += one;

        bits::get_direction(self.scratchpad, resolution)
    }
}

impl Iterator for Children {
    type Item = CellIndex;

    fn next(&mut self) -> Option<CellIndex> {
        // No more children, we're done.
        if self.count == 0 {
            return None;
        }

        // Extract the current index, to return it.
        let index = CellIndex::new_unchecked(self.scratchpad);
        self.count -= 1;

        // Prepare the next iteration, if any, by incrementing the scratchpad.
        if self.count != 0 {
            for resolution in Resolution::range(
                self.parent_resolution,
                self.target_resolution,
            )
            .rev()
            {
                // Move to the next direction value.
                let direction = self.next_direction(resolution);

                // First K axe of each resolution is skipped for pentagonal
                // index.
                if self.skip_count == i16::from(resolution)
                    && direction == u8::from(Direction::K)
                {
                    self.next_direction(resolution);
                    self.skip_count -= 1;
                }

                // If we have exhausted this resolution, move to the lower one.
                if direction > crate::direction::MAX {
                    self.scratchpad =
                        bits::clr_direction(self.scratchpad, resolution);
                    continue;
                }

                break;
            }
        }
        Some(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = usize::try_from(self.count).unwrap_or(usize::MAX);
        (count, Some(count))
    }
}

impl ExactSizeIterator for Children {}
impl FusedIterator for Children {}

// -----------------------------------------------------------------------------

/// Return the starting state for the listing process.
fn get_starting_state(index: CellIndex, resolution: Resolution) -> u64 {
    let parent_resolution = index.resolution();

    // Compute the range of resolution to iterate over.
    //
    // e.g. if we list children for cell index at resolution 2 to resolution 6
    // we need to iterate of 4 resolution (resolutions 3, 4, 5 and 6).
    let range =
        usize::from(resolution).saturating_sub(parent_resolution.into());

    let mut scratchpad = u64::from(index);
    // If we have resolution between current and targeted one we clear their
    // directions.
    if range != 0 {
        // Mask with the right number of bit to cover the directions.
        let mask = (1 << (range * h3o_bit::DIRECTION_BITSIZE)) - 1;
        // Mask offset required to clear the directions.
        let offset = resolution.direction_offset();

        // Clear directions.
        scratchpad &= !(mask << offset);
        // Set resolution.
        scratchpad = bits::set_resolution(scratchpad, resolution);
    }

    scratchpad
}

#[cfg(test)]
#[path = "./children_tests.rs"]
mod tests;
