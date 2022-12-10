use crate::{
    error::InvalidCellIndex,
    index::{bits, IndexMode},
    resolution, BaseCell, Direction, Resolution, DIRECTION_BITSIZE,
};
use std::{cmp::Ordering, fmt, num::NonZeroU64, str::FromStr};

/// Lookup table for number of children for hexagonal cells.
// 7.pow(resolution_delta)
const HEXAGON_CHILDREN_COUNTS: [u64; 16] = [
    1,
    7,
    49,
    343,
    2401,
    16_807,
    117_649,
    823_543,
    5_764_801,
    40_353_607,
    282_475_249,
    1_977_326_743,
    13_841_287_201,
    96_889_010_407,
    678_223_072_849,
    4_747_561_509_943,
];

/// Lookup table for number of children for pentagonal cells.
// 1 + 5 * (7.pow(resolution delta) - 1) / 6
const PENTAGON_CHILDREN_COUNTS: [u64; 16] = [
    1,
    6,
    41,
    286,
    2001,
    14_006,
    98_041,
    686_286,
    4_804_001,
    33_628_006,
    235_396_041,
    1_647_772_286,
    11_534_406_001,
    80_740_842_006,
    565_185_894_041,
    3_956_301_258_286,
];

// -----------------------------------------------------------------------------

/// Represents a cell (hexagon or pentagon) in the H3 grid system at a
/// particular resolution.
///
/// The index is encoded on 64-bit with the following bit layout:
///
/// ```text
///  ┏━┳━━━┳━━━━┳━━━━┳━━━━━━━┳━━━┳━━━┳━┈┈┈┈┈┈┈┈━┳━━━┳━━━┓
///  ┃U┃ M ┃ U  ┃ R  ┃ B     ┃C₀ ┃C₁ ┃          ┃C₁₄┃C₁₅┃
///  ┗━┻━━━┻━━━━┻━━━━┻━━━━━━━┻━━━┻━━━┻━┈┈┈┈┈┈┈┈━┻━━━┻━━━┛
/// 64 63 59   56   52      45  42  39          6   3   0
/// ```
///
/// Where:
/// - `U` are unused reserved bit, always set to 0 (bit 63 and bits 56-58).
/// - `M` is the index mode, always set to 1, coded on 4 bits (59-62).
/// - `R` is the cell resolution, in [0; 15], coded on 4 bits (52-55).
/// - `B` is the base cell, in [0; 121], coded on 7 bits (45-51)
/// - `C` are cells, coded on 3 bits each, with either a value in [0; 6] or the
///   pattern `0b111` if unused.
///
/// Refrences:
/// - [H3 Index Representations](https://h3geo.org/docs/core-library/h3Indexing)
/// - [H3 Index Bit Layout](https://observablehq.com/@nrabinowitz/h3-index-bit-layout?collection=@nrabinowitz/h3)
/// - [H3 Index Inspector](https://observablehq.com/@nrabinowitz/h3-index-inspector?collection=@nrabinowitz/h3)
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CellIndex(NonZeroU64);

impl CellIndex {
    /// Returns the resolution of the index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.resolution(), h3o::Resolution::Ten);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub const fn resolution(self) -> Resolution {
        bits::get_resolution(self.0.get())
    }

    /// Returns the base cell of the index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.base_cell(), h3o::BaseCell::try_from(15)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn base_cell(self) -> BaseCell {
        let value = bits::get_base_cell(self.0.get());
        // SAFETY: `CellIndex` only contains valid base cell (invariant).
        BaseCell::new_unchecked(value)
    }

    /// Returns true if this index represents a pentagonal cell.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert!(!index.is_pentagon());
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn is_pentagon(self) -> bool {
        let bits = self.0.get();
        let base = self.base_cell();

        let resolution = usize::from(bits::get_resolution(bits));
        let unused_count = usize::from(resolution::MAX) - resolution;
        let unused_bitsize = unused_count * DIRECTION_BITSIZE;
        let dirs_mask = (1 << (resolution * DIRECTION_BITSIZE)) - 1;
        let dirs = (bits >> unused_bitsize) & dirs_mask;

        // Pentagonal cells always have all directions but the base one set to
        // 0.
        base.is_pentagon() && dirs == 0
    }

    /// Returns the maximum number of icosahedron faces the index may intersect.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.max_face_count(), 2);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn max_face_count(self) -> usize {
        // A pentagon always intersects 5 faces.
        if self.is_pentagon() {
            5
        // An hexagon never intersects more than 2 (but may only intersect 1).
        } else {
            2
        }
    }

    /// Returns the cell at the given resolution in the index, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Direction, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.direction_at(Resolution::Five), Some(Direction::K));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn direction_at(self, resolution: Resolution) -> Option<Direction> {
        (resolution != Resolution::Zero && resolution <= self.resolution())
            .then(|| {
                let value = bits::get_direction(self.0.get(), resolution);
                Direction::new_unchecked(value)
            })
    }

    /// Returns the parent, at the specified resolution, of the cell.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(
    ///     index.parent(Resolution::Five),
    ///     CellIndex::try_from(0x851fb467fffffff).ok()
    /// );
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn parent(self, resolution: Resolution) -> Option<Self> {
        (resolution <= self.resolution()).then(|| {
            let bits = bits::set_resolution(self.0.get(), resolution);
            Self::new_unchecked(bits::set_unused(bits, resolution))
        })
    }

    /// Returns the center child index at the specified resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(
    ///     index.center_child(Resolution::Fifteen),
    ///     CellIndex::try_from(0x8f1fb46622d8000).ok()
    /// );
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn center_child(self, resolution: Resolution) -> Option<Self> {
        (resolution >= self.resolution()).then(|| {
            let start = self.resolution().direction_offset();
            let stop = resolution.direction_offset();
            let mask = (1 << (start - stop)) - 1;

            let bits = bits::set_resolution(self.0.get(), resolution);
            Self::new_unchecked(bits & !(mask << stop))
        })
    }

    /// Returns the exact number of children for a cell at a given resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.children_count(Resolution::Fifteen), 16_807);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    // In this case, `mut-let-if` is faster than the idiomatic `let-if-else`.
    // Actually 12.5% faster for hexagons and 3.5% slower for pentagons.
    // Given that hexagons are way more common than pentagons, worth it.
    #[allow(clippy::useless_let_if_seq)]
    pub fn children_count(self, resolution: Resolution) -> u64 {
        let resolution = usize::from(resolution);
        let curr_resolution = usize::from(bits::get_resolution(self.0.get()));
        if curr_resolution > resolution {
            return 0;
        }
        if curr_resolution == resolution {
            return 1;
        }

        let n = resolution - curr_resolution;
        let mut res = HEXAGON_CHILDREN_COUNTS[n];
        if self.is_pentagon() {
            res = PENTAGON_CHILDREN_COUNTS[n];
        }
        res
    }

    /// Returns all the base cell indexes.
    ///
    /// # Example
    ///
    /// ```
    /// let cells = h3o::CellIndex::base_cells().collect::<Vec<_>>();
    /// ```
    pub fn base_cells() -> impl Iterator<Item = Self> {
        // Template for a resolution 0 index
        // mode = CELL, resolution = 0, all children unset.
        const TEMPLATE: u64 = 0x0800_1fff_ffff_ffff;

        (0..BaseCell::count()).map(|base_cell| {
            Self::new_unchecked(bits::set_base_cell(TEMPLATE, base_cell))
        })
    }

    pub(crate) fn new_unchecked(value: u64) -> Self {
        debug_assert!(Self::try_from(value).is_ok(), "invalid cell index");
        Self(NonZeroU64::new(value).expect("valid cell index"))
    }
}

impl Ord for CellIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare while ignoring the resolution to get the right ordering.
        // This is useful when building hierarchical tree of H3 cells.
        //
        // To understand why, let's take an example with these two cells:
        // - Cell A: 0x89194e69d4fffff (resolution  9, 12-5-1-6-3-2-3-5-2-3)
        // - Cell B: 0x8a194e699ab7fff (resolution 10, 12-5-1-6-3-2-3-1-5-2-6)
        //
        // If we don't ignore the resolution, cell A comes BEFORE cell B
        // (because the resolution is lower AND resolution comes before the
        // cells in the bit layout, thus has more weight).
        //
        // By ignoring the resolution bits we get the right ordering.
        (bits::clr_resolution(self.0.get()))
            .cmp(&bits::clr_resolution(other.0.get()))
    }
}

impl PartialOrd for CellIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<CellIndex> for u64 {
    fn from(value: CellIndex) -> Self {
        value.0.get()
    }
}

impl TryFrom<u64> for CellIndex {
    type Error = InvalidCellIndex;

    // Basically a simpler/faster version of `h3IsValid`.
    //
    // Simpler because here we focus only on the trailing 56-bit part.
    // Faster because no loops, just plain ol' bitwise operationss :)
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if (value >> 56) & 0b1000_0111 != 0 {
            return Err(Self::Error::new(Some(value), "tainted reserved bits"));
        }
        if bits::get_mode(value) != u8::from(IndexMode::Cell) {
            return Err(Self::Error::new(Some(value), "invalid index mode"));
        }

        let base = BaseCell::try_from(bits::get_base_cell(value))
            .map_err(|_| Self::Error::new(Some(value), "invalid base cell"))?;

        // Resolution is always valid: coded on 4 bits, valid range is [0; 15].
        let resolution = usize::from(bits::get_resolution(value));

        // Check that we have a tail of unused cells  after `resolution` cells.
        //
        // We expect every bit to be 1 in the tail (because unused cells are
        // represented by `0b111`), i.e. every bit set to 0 after a NOT.
        let unused_count = usize::from(resolution::MAX) - resolution;
        let unused_bitsize = unused_count * DIRECTION_BITSIZE;
        let unused_mask = (1 << unused_bitsize) - 1;
        if (!value) & unused_mask != 0 {
            return Err(Self::Error::new(
                Some(value),
                "invalid unused direction pattern",
            ));
        }

        // Check that we have `resolution` valid cells (no unused ones).
        let dirs_mask = (1 << (resolution * DIRECTION_BITSIZE)) - 1;
        let dirs = (value >> unused_bitsize) & dirs_mask;
        if has_unused_direction(dirs) {
            return Err(Self::Error::new(
                Some(value),
                "unexpected unused direction",
            ));
        }

        // Check for pentagons with deleted subsequence.
        if base.is_pentagon() && resolution != 0 {
            // Move directions to the front, so that we can count leading
            // zeroes.
            let offset = 64 - (resolution * DIRECTION_BITSIZE);

            // Find the position of the first bit set, if it's a multiple of 3
            // that means we have a K axe as the first non-center direction,
            // which is forbidden.
            if ((dirs << offset).leading_zeros() + 1) % 3 == 0 {
                return Err(Self::Error::new(
                    Some(value),
                    "pentagonal cell index with a deleted subsequence",
                ));
            }
        }

        // XXX: 0 is rejected by the mode check (mode cannot be 0).
        Ok(Self(NonZeroU64::new(value).expect("non-zero cell index")))
    }
}

impl FromStr for CellIndex {
    type Err = InvalidCellIndex;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16)
            .map_err(|_| Self::Err {
                value: None,
                reason: "invalid 64-bit hex number",
            })
            .and_then(Self::try_from)
    }
}

impl fmt::Debug for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{:015o} ({})",
            self.base_cell(),
            u64::from(*self) & bits::DIRECTIONS_MASK,
            self
        )
    }
}

impl fmt::Display for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::Binary for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::Octal for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// -----------------------------------------------------------------------------

/// Checks if there is at least one unused direction in the given directions.
#[inline(always)]
#[rustfmt::skip] // Keep constants aligned for readability.
#[allow(clippy::unusual_byte_groupings)] // Grouping by 3-bit is better here.
const fn has_unused_direction(dirs: u64) -> bool {
    // Unused directions are represented by `0b111`, so we actually want to
    // check the absence of this pattern.
    // This is akin to splitting the data into chunks of 3 bits and looking for
    // the presence of a three-1 triplet.
    //
    // Now, looking for `0b111` is clearly not a common task, but we can twist
    // the problem a bit to find back our footing ;)
    // If we apply a NOT on our data we're now looking for `0b000` which is
    // awfully similar to the research of a nul byte, a well-known task in
    // C-land thanks to null-terminated strings.
    //
    // STOP, Archeology time!
    //
    // Let's dive into the annals of the Old Gods, a.k.a. comp.lang.c, and
    // extract this golden nugget: Alan Mycroft's null-byte detection algorithm,
    // posted in 1987
    // See: https://groups.google.com/forum/#!original/comp.lang.c/2HtQXvg7iKc/xOJeipH6KLMJ
    //
    // The spell is: (value - lo_magic) & (!value & hi_magic)
    //
    // Here's a quick rundown on how it works:
    //
    // - The first part, `value - lo_magic`, will make sure that the MSB (most
    //   significant bit) of each chunk is set if:
    //   * the chunk is null (`0b000 - 0b001` wraps around to `0b111`).
    //   * the MSB + another bit are already set, e.g. `0b101`. That's because
    //     the lowest bit absorb the subtraction and the highest one is left
    //     untouched (e.g. `0b101 - 0b001 = 0b100`)
    // - The second part, `!value & hi_magic`, will set the MSB of each chunk
    //   only if the MSB was unset in the original value.
    //
    // By ANDing both parts, we get a non-zero value if there was at least one
    // null chunk: the first part selects null chunks and the ones with the MSB
    // already set whereas the second part filter out the latter, thus leaving
    // only null chunk with a bit set.
    //
    // A little example:
    //
    //     dirs   = 001 010 111 011 110 110 000
    //     !dirs  = 110 101 000 100 001 001 111 // negate to convert 111 to 000.
    //     part 1 = 101 011 111 011 000 000 110
    //     part 2 = 000 000 100 000 100 100 000
    //     result = 000 000 100 000 000 000 000
    //
    // By tweaking this a bit to works on 64-bit AND on triplet instead of
    // bytes, the magic occurs :)
    const LO_MAGIC: u64 = 0b001_001_001_001_001_001_001_001_001_001_001_001_001_001_001;
    const HI_MAGIC: u64 = 0b100_100_100_100_100_100_100_100_100_100_100_100_100_100_100;

    ((!dirs - LO_MAGIC) & (dirs & HI_MAGIC)) != 0
}

#[cfg(test)]
#[path = "./cell_tests.rs"]
mod tests;
