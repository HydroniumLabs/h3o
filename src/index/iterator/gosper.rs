//! Iterator for the directed edges forming the "Gosper island" outline of a
//! cell's child set at a given resolution.
//!
//! For a cell with resolution `r`, the boundary of the cell's children at
//! resolution `R` forms a closed loop of directed edges (the Gosper island).
//! For a hexagon this is `6 * 3^(R - r)` edges; for a pentagon, `5 * 3^(R -
//! r)`.
//! We refer to `R - r` as the resolution delta.
//!
//! # Geometric intuition
//!
//! At delta 0, we just return the cell's own edges in counter-clockwise order,
//! cycling through the `EDGE_DIR` array.
//!
//! At delta 1 (for a hexagon), each side of the parent subdivides into 3 edges
//! at the finer resolution, giving 6 * 3 = 18 edges total. The origin cells
//! share the parent's digit path, extended by one digit into the child
//! resolution. The `WALK_DIGIT` array maps these 18 walk positions to H3 digit
//! values: the 6 non-center child digits in counter-clockwise order, each
//! repeated 3x for the subdivision.
//!
//! # Pentagon handling
//!
//! Pentagons have 5 sides instead of 6: the K-axis (H3 digit 1) has no
//! child cell (the deleted subsequence). To handle pentagonal input, we build
//! the digit sequence as if it were a hexagon, then skip over the deleted
//! subsequences (edges that land on the missing K-axis side).

use crate::{
    CellIndex, DirectedEdgeIndex, Direction, Edge, IndexMode, Resolution,
    index::bits,
};
use core::{
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

/// H3 digit at each of the 18 boundary walk positions.
///
/// The 6 non-center digits {1,5,4,6,2,3} in counter-clockwise order around
/// the hexagon, each repeated 3x for the fractal subdivision at each side.
/// Note: `WALK_DIGIT[i]` == `EDGE_DIR[(i/3 + 1) % 6]`; the same cyclic sequence
/// of digits, rotated by 1 and each element repeated 3x.
#[rustfmt::skip]
const WALK_DIGIT: WalkDigits = WalkDigits([
    Direction::K,  Direction::K,  Direction::K,  // 1,1,1
    Direction::IK, Direction::IK, Direction::IK, // 5,5,5
    Direction::I,  Direction::I,  Direction::I,  // 4,4,4
    Direction::IJ, Direction::IJ, Direction::IJ, // 6,6,6
    Direction::J,  Direction::J,  Direction::J,  // 2,2,2
    Direction::JK, Direction::JK, Direction::JK  // 3,3,3
]);

/// H3 edge direction at each edge index, in counter-clockwise order around the
/// hexagon.
/// Numeric values: {3, 1, 5, 4, 6, 2}
const EDGE_DIR: EdgeDirections = EdgeDirections([
    Direction::JK,
    Direction::K,
    Direction::IK,
    Direction::I,
    Direction::IJ,
    Direction::J,
]);

/// Iterator for the directed edges on the boundary of a cell's child set (the
/// Gosper island outline) at a given resolution.
///
/// Each yielded edge is a directed edge whose origin cell is a child of the
/// parent cell and whose destination cell is not. The edges form a closed loop
/// in counter-clockwise order around the Gosper island.
///
/// The starting edge is not guaranteed; any cyclic rotation of the sequence is
/// a valid output.
#[derive(Clone)]
pub struct Gosper {
    /// Iterator scratch space, used to build directed edge iteratively.
    scratchpad: u64,
    /// Number of edges left to yield (including current).
    count: u64,
    /// Position in `EDGE_DIR` cycle (period 6)
    edge_idx: EdgeDirectionsIndex,
    /// Position in `WALK_DIGIT` cycle (period 18) per resolution level.
    walk_idx: WalkIndices,
    parent_resolution: Resolution,
    child_resolution: Resolution,
    is_pentagon: bool,
}

impl Gosper {
    pub fn new(index: CellIndex, child_resolution: Resolution) -> Self {
        let parent_resolution = index.resolution();
        let is_pentagon = index.is_pentagon();
        let nb_sides = 6 - u64::from(is_pentagon);

        let mut iter = Self {
            scratchpad: 0,
            count: 0,
            edge_idx: EdgeDirectionsIndex::default(),
            walk_idx: WalkIndices::default(),
            parent_resolution,
            child_resolution,
            is_pentagon,
        };

        // In that case returns an empty iterator.
        if child_resolution < parent_resolution {
            return iter;
        }

        let mut scratchpad =
            bits::set_mode(u64::from(index), IndexMode::DirectedEdge);
        // Same-resolution fast path: no digit/resolution setup needed.
        if child_resolution == parent_resolution {
            iter.scratchpad = bits::set_edge(scratchpad, Direction::JK.into());
            iter.count = nb_sides;
            return iter;
        }

        // Set number of edges for the iterator to yield.
        let delta = u8::from(child_resolution) - u8::from(parent_resolution);
        iter.count = nb_sides * 3_u64.pow(delta.into());

        // Set resolution to child level.
        scratchpad = bits::set_resolution(scratchpad, child_resolution);

        // Arrived here we are sure that `parent < child` => `succ` cannot fail.
        let first_child = parent_resolution.succ().expect("parent < child");

        // The first child level starts at walk position 0.
        let direction = Direction::K.into();
        scratchpad = bits::set_direction(scratchpad, direction, first_child);

        // Subsequent levels start at either 14 or 16, depending on the class.
        if let Some(next_child) = first_child.succ() {
            for res in Resolution::range(next_child, child_resolution) {
                iter.walk_idx[res] = WalkDigitsIndex::new(res.is_class3());

                let direction = WALK_DIGIT[iter.walk_idx[res]].into();
                scratchpad = bits::set_direction(scratchpad, direction, res);
            }
        }

        // Set initial direction.
        let edge = EDGE_DIR.edge_at(iter.edge_idx);
        iter.scratchpad = bits::set_edge(scratchpad, edge);

        // For pentagons, the walk may start on a deleted subsequence.
        // Advance past those positions to reach the first valid edge.
        iter.skip_pentagon_edges();

        iter
    }

    /// Advance to the next boundary edge (origin cell + edge direction).
    fn advance_edge(&mut self) {
        let cell_changed = self.advance_origin_cell();

        if cell_changed {
            self.edge_idx.next_cell();
        }
        self.edge_idx.next_edge();

        let edge = EDGE_DIR.edge_at(self.edge_idx);
        self.scratchpad = bits::set_edge(self.scratchpad, edge);
    }

    fn skip_pentagon_edges(&mut self) {
        if !self.is_pentagon {
            return;
        }

        // Arrived here we are sure that `parent < child` => `succ` cannot fail.
        debug_assert!(self.parent_resolution < self.child_resolution);
        while bits::get_direction(
            self.scratchpad,
            self.parent_resolution.succ().expect("parent < child"),
        ) == Direction::K.into()
        {
            self.advance_edge();
        }
    }

    /// Advance the walk along origin cells on the Gosper island boundary,
    /// updating the child digits.
    ///
    /// Each resolution level cycles through 18 walk positions (6 sides * 3
    /// segments per side). Across resolutions, each step at a coarser level
    /// maps to 3 steps at the next finer level. When the coarser level moves to
    /// the next cell, the finer level shifts back by 6 positions.
    ///
    /// Returns true if the origin cell changed.
    fn advance_origin_cell(&mut self) -> bool {
        // The reference implementation in H3 is recursive.
        // It’s not a tail call, so TCO won’t kick in (and Rust doesn’t
        // guarantee it anyway).
        //
        // But since the maximum recursion depth is bounded (and small), we can
        // easily use an iterative approach with a local stack instead. Part of
        // the stopping condition is `child_res > parent_res + 1`, and by taking
        // the extreme case of `child_res = 15` and `parent_res = 0`, we can see
        // that we will recurse at most 14 times.
        //
        // Add one to cover any off-by-one issues (since I was tired when
        // writing this), and one more to get a nice "round" size of 16, because
        // why not xD
        //
        // Anyway, by doing it this way we avoid a function call, and it also
        // seems to help the compiler decide to inline the whole function, which
        // gives a nice performance boost (30% in some cases).
        let mut stack = [0; 16];
        let mut depth = 0;

        let mut curr_res = self.child_resolution;
        let mut curr_dir = bits::get_direction(self.scratchpad, curr_res);

        // When moving to the next cell, recurse to advance the parent res.
        while let Some(prev_resolution) = curr_res.pred()
            && prev_resolution > self.parent_resolution
            && need_to_move_to_next(self.walk_idx[curr_res], curr_res)
        {
            stack[depth] = curr_dir;
            depth += 1;
            curr_res = prev_resolution;
            curr_dir = bits::get_direction(self.scratchpad, curr_res);
        }

        // Leaf call, tie in the recursion and prepare to backtrack.
        self.walk_idx[curr_res].next_position();
        let next_dir = WALK_DIGIT[self.walk_idx[curr_res]].into();
        self.scratchpad =
            bits::set_direction(self.scratchpad, next_dir, curr_res);
        let mut parent_changed = next_dir != curr_dir;

        // Backtracking, let's go!
        while depth > 0 {
            depth -= 1;
            let curr_dir = stack[depth];
            // Cannot fail by construction: we cancel previous `pred`).
            curr_res = curr_res.succ().expect("backtrack");

            if parent_changed {
                self.walk_idx[curr_res].next_parent();
            }
            self.walk_idx[curr_res].next_position();

            // Update the child digit.
            let next_dir = WALK_DIGIT[self.walk_idx[curr_res]].into();
            self.scratchpad =
                bits::set_direction(self.scratchpad, next_dir, curr_res);

            // Change in the finest digit is sufficient to detect a cell change.
            parent_changed = next_dir != curr_dir;
        }

        parent_changed
    }
}

impl Iterator for Gosper {
    type Item = DirectedEdgeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scratchpad == 0 {
            return None;
        }
        let res = DirectedEdgeIndex::new_unchecked(self.scratchpad);

        self.count -= 1;
        if self.count == 0 {
            self.scratchpad = 0;
            return Some(res);
        }

        if self.parent_resolution == self.child_resolution {
            // Easy case of 0 resolution delta: cycle through edge indices,
            // skipping the deleted subsequence for pentagons.
            self.edge_idx.next_edge();
            if self.is_pentagon && EDGE_DIR[self.edge_idx] == Direction::K {
                self.edge_idx.next_edge();
            }

            let edge = EDGE_DIR.edge_at(self.edge_idx);
            self.scratchpad = bits::set_edge(self.scratchpad, edge);
        } else {
            self.advance_edge();
            // Skip deleted subsequences for pentagons.
            self.skip_pentagon_edges();
        }

        Some(res)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = usize::try_from(self.count).unwrap_or(usize::MAX);
        (count, Some(count))
    }
}

impl ExactSizeIterator for Gosper {}
impl FusedIterator for Gosper {}

/// The point where each level moves to the next cell depends on resolution
/// class:
/// - Class II (even resolution) moves at `walk_idx % 3 == 0`
/// - Class III (odd resolution) moves at `walk_idx % 3 == 1`
#[inline]
fn need_to_move_to_next(
    walk_idx: WalkDigitsIndex,
    curr_resolution: Resolution,
) -> bool {
    // What we want to compute is basically `(walk_idx % 3 == res % 2)`.
    // `% 2` is a well-known pattern, compiles down to a bitwise AND. Perfect!
    // `% 3` is trickier, but modern compilers are very smart, and when you
    // divide/modulo by a constant, they’ll try hard to replace the "division by
    // a constant" with a multiplication by a constant and a shift:
    //
    //     imul    rcx, rax, 1431655766
    //     mov     rdx, rcx
    //     shr     rdx, 63
    //     shr     rcx, 32
    //     add     ecx, edx
    //     lea     ecx, [rcx + 2*rcx]
    //     sub     eax, ecx
    //
    // No `div` in sight, just multiplication by a magic constant, exploiting
    // two's complement wraparound, plus a couple of shifts/adds.
    //
    // Can't beat that in the general case, but here we have more information
    // that the compiler might not be able to infer: our operand is constrained
    // to `[0, 17]` since it’s an index into `WALK_DIGIT`.
    //
    // We could avoid any computation and just use a lookup table, e.g.
    //
    //     const MOD3: [u8; 18] = [0,1,2,0,1,2,0,1,2,0,1,2,0,1,2,0,1,2];
    //     MOD3[walk_idx] == (res & 1)
    //
    // which compiles down to:
    //
    //     lea     rax, [rip + .Lanon.ae42a548213d55a91f74f9adc5422b7f.0]
    //     and     sil, 1
    //     cmp     byte ptr [rdi + rax], sil
    //
    // Very tight: no more multiply, just a load, a mask, and a comparison.
    //
    // But the load is a bit annoying (what if the LUT is not in cache?), and
    // the LUT access itself is likely guarded by a bounds check in Rust.
    //
    // Notice how small the table is, and how small its values are as well?
    // What if we packed this LUT into a `u64`?
    //
    // That’s the magic constant below, and it compiles to:
    //
    //     movabs  rax, -11998638796
    //     shr     rax, cl
    //     and     eax, 3
    //     and     esi, 1
    //     cmp     eax, esi
    //
    // And all is right in the world again.
    // This micro-optimization get us 4% speedup.
    const MOD3_LUT: u64 = 0xfffffffd34d34d34;

    let shift = walk_idx.0 << 1;
    let class = u8::from(curr_resolution) & 1;

    (MOD3_LUT >> shift) & 0b11 == u64::from(class)
}

// -----------------------------------------------------------------------------

struct EdgeDirections([Direction; 6]);

impl EdgeDirections {
    fn edge_at(&self, index: EdgeDirectionsIndex) -> Edge {
        self[index].into()
    }
}

impl Index<EdgeDirectionsIndex> for EdgeDirections {
    type Output = Direction;

    fn index(&self, index: EdgeDirectionsIndex) -> &Self::Output {
        &self.0[usize::from(index.0)]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct EdgeDirectionsIndex(u8);

impl EdgeDirectionsIndex {
    const fn next_cell(&mut self) {
        // If we move to a new origin cell, shift the `edge_idx` back by 2.
        if self.0 >= 2 {
            self.0 -= 2;
        } else {
            self.0 += 4;
        }
    }

    const fn next_edge(&mut self) {
        if self.0 == 5 {
            self.0 = 0;
        } else {
            self.0 += 1;
        }
    }
}

// -----------------------------------------------------------------------------

struct WalkDigits([Direction; 18]);

impl Index<WalkDigitsIndex> for WalkDigits {
    type Output = Direction;

    fn index(&self, index: WalkDigitsIndex) -> &Self::Output {
        &self.0[usize::from(index.0)]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct WalkDigitsIndex(u8);

impl WalkDigitsIndex {
    const fn new(is_class3: bool) -> Self {
        Self(if is_class3 { 14 } else { 16 })
    }

    const fn next_parent(&mut self) {
        // New parent cell: shift back by 6 positions.
        if self.0 < 6 {
            self.0 += 12;
        } else {
            self.0 -= 6;
        }
    }

    const fn next_position(&mut self) {
        if self.0 == 17 {
            self.0 = 0;
        } else {
            self.0 += 1;
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Default)]
struct WalkIndices([WalkDigitsIndex; 16]);

impl Index<Resolution> for WalkIndices {
    type Output = WalkDigitsIndex;

    fn index(&self, index: Resolution) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<Resolution> for WalkIndices {
    fn index_mut(&mut self, index: Resolution) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

// -----------------------------------------------------------------------------

#[cfg(test)]
#[path = "./gosper_tests.rs"]
mod tests;
