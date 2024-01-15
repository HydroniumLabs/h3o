use super::{Children, Compact, GridPathCells, Triangle};
use crate::{
    coord::{CoordIJ, CoordIJK, FaceIJK, LocalIJK, Overage},
    error::{
        CompactionError, HexGridError, InvalidCellIndex, LocalIjError,
        ResolutionMismatch,
    },
    grid,
    index::{bits, IndexMode},
    BaseCell, Boundary, DirectedEdgeIndex, Direction, Edge, ExtendedResolution,
    FaceSet, LatLng, LocalIJ, Resolution, Vertex, VertexIndex, CCW, CW,
    DEFAULT_CELL_INDEX, EARTH_RADIUS_KM, NUM_HEX_VERTS, NUM_PENT_VERTS,
};
use either::Either;
use std::{
    cmp::Ordering,
    fmt, iter,
    num::{NonZeroU64, NonZeroU8},
    str::FromStr,
};

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

/// Reverse direction from neighbor in each direction given as an index into
/// `DIRECTIONS` to facilitate rotation.
const REV_NEIGHBOR_DIRECTIONS_HEX: [u8; 6] = [5, 3, 4, 1, 0, 2];

// These sets are the relevant neighbors in the clockwise and counter-clockwise.
const NEIGHBOR_SET_CW: [Direction; 7] = [
    Direction::Center,
    Direction::JK,
    Direction::IJ,
    Direction::J,
    Direction::IK,
    Direction::K,
    Direction::I,
];
const NEIGHBOR_SET_CCW: [Direction; 7] = [
    Direction::Center,
    Direction::IK,
    Direction::JK,
    Direction::K,
    Direction::IJ,
    Direction::I,
    Direction::J,
];

/// Origin leading digit -> index leading digit -> rotations 60 CW.
///
/// Either being 1 (K axis) is invalid.
/// No good default at 0.
#[rustfmt::skip]
const PENTAGON_ROTATIONS: [[u8; 7]; 7] = [
    [ 0,    0xff, 0,    0,    0,    0,    0],    // 0
    [ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], // 1
    [ 0,    0xff, 0,    0,    0,    1,    0],    // 2
    [ 0,    0xff, 0,    0,    1,    1,    0],    // 3
    [ 0,    0xff, 0,    5,    0,    0,    0],    // 4
    [ 0,    0xff, 5,    5,    0,    0,    0],    // 5
    [ 0,    0xff, 0,    0,    0,    0,    0],    // 6
];

/// Prohibited directions when unfolding a pentagon.
///
/// Indexes by two directions, both relative to the pentagon base cell. The
/// first is the direction of the origin index and the second is the direction
/// of the index to unfold. Direction refers to the direction from base cell to
/// base cell if the indexes are on different base cells, or the leading digit
/// if within the pentagon base cell.
///
/// This previously included a Class II/Class III check but these were removed
/// due to failure cases. It's possible this could be restricted to a narrower
/// set of a failure cases. Currently, the logic is any unfolding across more
/// than one icosahedron face is not permitted.
#[allow(clippy::unusual_byte_groupings)] // Grouping by 7 is more explicit here.
const FAILED_DIRECTIONS: u64 =
    //   6       5       4       3       2       1       0
    0b0101000_1000100_0001100_1010000_0110000_0000000_0000000;

const fn validate_direction(
    origin_dir: u8,
    index_dir: u8,
) -> Result<(), LocalIjError> {
    let offset = origin_dir * 7 + index_dir;
    if (FAILED_DIRECTIONS & (1 << offset)) != 0 {
        // TODO: We may be unfolding the pentagon incorrectly in
        // this case; return an error code until this is guaranteed
        // to be correct.
        return Err(LocalIjError::Pentagon);
    }
    Ok(())
}

/// Directions in CCW order.
pub const DIRECTIONS: [Direction; NUM_HEX_VERTS as usize] = [
    Direction::J,
    Direction::JK,
    Direction::K,
    Direction::IK,
    Direction::I,
    Direction::IJ,
];

/// This base cell map to IJK {0, 0, 0}, which is needed in `to_local_ijk`.
const BASE_CELL: BaseCell = BaseCell::new_unchecked(2);

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
/// References:
/// - [H3 Index Representations](https://h3geo.org/docs/core-library/h3Indexing)
/// - [H3 Index Bit Layout](https://observablehq.com/@nrabinowitz/h3-index-bit-layout?collection=@nrabinowitz/h3)
/// - [H3 Index Inspector](https://observablehq.com/@nrabinowitz/h3-index-inspector?collection=@nrabinowitz/h3)
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        let value = h3o_bit::get_base_cell(self.0.get());
        // SAFETY: `CellIndex` only contains valid base cell (invariant).
        BaseCell::new_unchecked(value)
    }

    /// Computes the area of this H3 cell, in radians².
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.area_rads2(), 3.3032558516982826e-10);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn area_rads2(self) -> f64 {
        let center = LatLng::from(self);
        let boundary = self.boundary();

        (0..boundary.len())
            .map(|i| {
                let j = (i + 1) % boundary.len();
                Triangle::new(boundary[i], boundary[j], center).area()
            })
            .sum()
    }

    /// Computes the area of this H3 cell, in km².
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.area_km2(), 0.013407827139722947);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn area_km2(self) -> f64 {
        self.area_rads2() * EARTH_RADIUS_KM * EARTH_RADIUS_KM
    }

    /// Computes the area of this H3 cell, in m².
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.area_m2(), 13407.827139722947);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn area_m2(self) -> f64 {
        self.area_km2() * 1000. * 1000.
    }

    /// Finds all icosahedron faces intersected this cell index
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let faces = index.icosahedron_faces().iter().collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn icosahedron_faces(self) -> FaceSet {
        let resolution = self.resolution();
        let is_pentagon = self.is_pentagon();

        // We can't use the vertex-based approach here for class II pentagons,
        // because all their vertices are on the icosahedron edges. Their direct
        // child pentagons cross the same faces, so use those instead.
        if is_pentagon && !resolution.is_class3() {
            // We are working on Class II pentagons here (`resolution`
            // up to 14), hence we can always get a finer resolution.
            let child_resolution = resolution.succ().expect("child resolution");
            let bits = bits::set_resolution(self.0.get(), child_resolution);
            return Self::new_unchecked(bits::set_direction(
                bits,
                0,
                child_resolution,
            ))
            .icosahedron_faces();
        }

        // Convert to FaceIJK.
        let mut fijk = FaceIJK::from(self);

        // Get all vertices as FaceIJK addresses. For simplicity, always
        // initialize the array with 6 verts, ignoring the last one for
        // pentagons.
        let mut vertices = [FaceIJK::default(); NUM_HEX_VERTS as usize];
        let (vertex_count, resolution) = if is_pentagon {
            (
                usize::from(NUM_PENT_VERTS),
                fijk.vertices(
                    resolution,
                    &mut vertices[..usize::from(NUM_PENT_VERTS)],
                ),
            )
        } else {
            (
                usize::from(NUM_HEX_VERTS),
                fijk.vertices(resolution, &mut vertices),
            )
        };

        let mut faces = FaceSet::new();

        // Add each vertex face.
        for vertex in &mut vertices[..vertex_count] {
            // Adjust overage, determining whether this vertex is on another
            // face.
            if is_pentagon {
                vertex.adjust_pentagon_vertex_overage(resolution);
            } else {
                vertex.adjust_overage_class2::<true>(resolution, false);
            }

            faces.insert(vertex.face);
        }

        faces
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
        let unused_count = usize::from(h3o_bit::MAX_RESOLUTION) - resolution;
        let unused_bitsize = unused_count * h3o_bit::DIRECTION_BITSIZE;
        let dirs_mask = (1 << (resolution * h3o_bit::DIRECTION_BITSIZE)) - 1;
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

    /// Returns the position of the cell within an ordered list of all children
    /// of the cell's parent at the specified resolution.
    ///
    /// Returns `None` if the cell's resolution is coarser than the given
    /// resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(index.child_position(Resolution::Eight), Some(24));
    /// assert_eq!(index.child_position(Resolution::Twelve), None);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn child_position(self, resolution: Resolution) -> Option<u64> {
        let Some(parent_is_pentagon) =
            self.parent(resolution).map(Self::is_pentagon)
        else {
            // cell's resolution is coarser than `resolution`.
            return None;
        };

        Some(if parent_is_pentagon {
            Resolution::range(resolution, self.resolution())
                .skip(1)
                .map(|res| {
                    // Thansk to the `skip(1)`, iteration cannot start below
                    // resolution 1, thus the calls to pred always succeed.
                    // Moreover, the check at the start of the function ensure
                    // that we can get the parent at every iteration.
                    let parent_is_pentagon = self
                        .parent(res.pred().expect("resolution > 0"))
                        .map(Self::is_pentagon)
                        .expect("valid parent");
                    let mut digit = bits::get_direction(self.0.get(), res);
                    // Adjust digit index for deleted K-subsequence.
                    if parent_is_pentagon && digit > 0 {
                        digit -= 1;
                    };
                    if digit == 0 {
                        return 0;
                    }

                    let diff = u8::from(self.resolution()) - u8::from(res);
                    let hex_count = HEXAGON_CHILDREN_COUNTS[usize::from(diff)];
                    // The offset for the 0-digit slot depends on whether the
                    // current index is the child of a pentagon. If so, the offset
                    // is based on the count of pentagon children, otherwise,
                    // hexagon children.
                    let count0 = if parent_is_pentagon {
                        PENTAGON_CHILDREN_COUNTS[usize::from(diff)]
                    } else {
                        hex_count
                    };
                    u64::from(digit - 1) * hex_count + count0
                })
                .sum()
        } else {
            Resolution::range(resolution, self.resolution())
                .skip(1)
                .map(|res| {
                    let diff = u8::from(self.resolution()) - u8::from(res);
                    let hex_count = HEXAGON_CHILDREN_COUNTS[usize::from(diff)];
                    let digit = bits::get_direction(self.0.get(), res);
                    u64::from(digit) * hex_count
                })
                .sum()
        })
    }

    /// Returns the child cell at a given position within an ordered list of
    /// all children at the specified resolution.
    ///
    /// Returns `None` if no child can be found (coarser resolution, index out
    /// of bound, …).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x881fb46623fffff)?;
    /// assert_eq!(
    ///     index.child_at(24, Resolution::Ten),
    ///     CellIndex::try_from(0x8a1fb46622dffff).ok(),
    /// );
    /// assert_eq!(index.child_at(24, Resolution::Five), None);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn child_at(
        self,
        mut position: u64,
        resolution: Resolution,
    ) -> Option<Self> {
        #[allow(clippy::cast_possible_truncation)] // Safe thx to assert.
        fn set_direction(bits: u64, digit: u64, resolution: Resolution) -> u64 {
            assert!(digit < 7);
            bits::set_direction(bits, digit as u8, resolution)
        }

        if resolution < self.resolution()
            || position >= self.children_count(resolution)
        {
            return None;
        }

        let mut child = bits::set_resolution(self.0.get(), resolution);
        let mut cur_res = self.resolution();
        if self.is_pentagon() {
            // While we are inside a parent pentagon, we need to check
            // if this cell is a pentagon, and if not, we need to offset
            // its digit to account for the skipped direction
            for res in Resolution::range(self.resolution(), resolution).skip(1)
            {
                cur_res = res;
                let diff = u8::from(resolution) - u8::from(res);
                let pent_count = PENTAGON_CHILDREN_COUNTS[usize::from(diff)];
                if position < pent_count {
                    child = bits::set_direction(child, 0, res);
                } else {
                    let count = HEXAGON_CHILDREN_COUNTS[usize::from(diff)];
                    position -= pent_count;
                    child = set_direction(child, (position / count) + 2, res);
                    position %= count;
                    break;
                }
            }
        }
        for res in Resolution::range(cur_res, resolution).skip(1) {
            let diff = u8::from(resolution) - u8::from(res);
            let count = HEXAGON_CHILDREN_COUNTS[usize::from(diff)];
            child = set_direction(child, position / count, res);
            position %= count;
        }

        Some(Self::new_unchecked(child))
    }

    /// Return the children, at the specified resolution, of the cell index.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let children = index.children(Resolution::Eleven).collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn children(
        self,
        resolution: Resolution,
    ) -> impl Iterator<Item = Self> {
        Children::new(self, resolution)
    }

    /// Compresses a set of unique cell indexes all at the same resolution.
    ///
    /// The indexes are compressed by pruning full child branches to the parent
    /// level. This is also done for all parents recursively to get the minimum
    /// number of hex addresses that perfectly cover the defined space.
    ///
    /// # Errors
    ///
    /// All cell indexes must be unique and have the same resolution, otherwise
    /// an [`CompactionError`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let cells = [
    ///     0x081003ffffffffff,
    ///     0x081023ffffffffff,
    ///     0x081043ffffffffff,
    ///     0x081063ffffffffff,
    ///     0x081083ffffffffff,
    ///     0x0810a3ffffffffff,
    ///     0x0810c3ffffffffff,
    /// ]
    /// .into_iter()
    /// .map(|hex| CellIndex::try_from(hex))
    /// .collect::<Result<Vec<_>, _>>()?;
    /// let compacted_cells = CellIndex::compact(cells)?.collect::<Vec<_>>();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn compact(
        indexes: impl IntoIterator<Item = Self>,
    ) -> Result<impl Iterator<Item = Self>, CompactionError> {
        Compact::new(indexes)
    }

    /// Computes the exact size of the uncompacted set of cells.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let size = CellIndex::uncompact_size(std::iter::once(index), Resolution::Eleven);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn uncompact_size(
        compacted: impl IntoIterator<Item = Self>,
        resolution: Resolution,
    ) -> u64 {
        compacted
            .into_iter()
            .map(move |index| index.children_count(resolution))
            .sum()
    }

    /// Expands a compressed set of cells into a set of cells of the specified
    /// resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells = CellIndex::uncompact(
    ///     std::iter::once(index), Resolution::Eleven
    /// ).collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn uncompact(
        compacted: impl IntoIterator<Item = Self>,
        resolution: Resolution,
    ) -> impl Iterator<Item = Self> {
        compacted
            .into_iter()
            .flat_map(move |index| index.children(resolution))
    }

    /// Computes the cell boundary, in spherical coordinates, of this index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let boundary = index.boundary().iter().collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn boundary(self) -> Boundary {
        let fijk = FaceIJK::from(self);
        // SAFETY: 0 is always a valid vertex.
        let start = Vertex::new_unchecked(0);
        if self.is_pentagon() {
            fijk.pentagon_boundary(self.resolution(), start, NUM_PENT_VERTS)
        } else {
            fijk.hexagon_boundary(self.resolution(), start, NUM_HEX_VERTS)
        }
    }

    /// Returns all the base cell indexes.
    ///
    /// # Example
    ///
    /// ```
    /// let cells = h3o::CellIndex::base_cells().collect::<Vec<_>>();
    /// ```
    pub fn base_cells() -> impl Iterator<Item = Self> {
        (0..BaseCell::count()).map(|base_cell| {
            Self::new_unchecked(h3o_bit::set_base_cell(
                DEFAULT_CELL_INDEX,
                base_cell,
            ))
        })
    }

    /// Returns the edge between the current cell and the specified destination.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Direction, DirectedEdgeIndex};
    ///
    /// let src = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let dst = CellIndex::try_from(0x8a1fb46622d7fff)?;
    /// assert_eq!(src.edge(dst), DirectedEdgeIndex::try_from(0x16a1fb46622dffff).ok());
    ///
    /// // Not a neighbor, thus no shared edge.
    /// let dst = CellIndex::try_from(0x8a1fb4644937fff)?;
    /// assert!(src.edge(dst).is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn edge(self, destination: Self) -> Option<DirectedEdgeIndex> {
        // Determine the IJK direction from the origin to the destination
        grid::direction_for_neighbor(self, destination).map(|direction| {
            let bits = bits::set_mode(u64::from(self), IndexMode::DirectedEdge);
            // SAFETY: `direction_for_neighbor` always return valid edge value.
            DirectedEdgeIndex::new_unchecked(bits::set_edge(
                bits,
                Edge::new_unchecked(direction.into()),
            ))
        })
    }

    /// Returns all of the directed edges from the current index.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let edges = index.edges().collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn edges(self) -> impl Iterator<Item = DirectedEdgeIndex> {
        let template = bits::set_mode(self.0.get(), IndexMode::DirectedEdge);
        let deleted_edge = self.is_pentagon().then_some(1);

        Edge::iter()
            .filter(move |&edge| (Some(u8::from(edge)) != deleted_edge))
            .map(move |edge| {
                DirectedEdgeIndex::new_unchecked(bits::set_edge(template, edge))
            })
    }

    /// Get the specified vertex of this cell.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Vertex, VertexIndex};
    ///
    /// let index = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// assert_eq!(
    ///     index.vertex(Vertex::try_from(3)?),
    ///     VertexIndex::try_from(0x25a1fb464492ffff).ok()
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn vertex(self, vertex: Vertex) -> Option<VertexIndex> {
        let vertex_count = self.vertex_count();
        let resolution = self.resolution();

        // Check for invalid vertexes.
        if u8::from(vertex) >= vertex_count {
            return None;
        }

        // Default the owner and vertex number to current cell.
        let mut owner = self;
        let mut owner_vertex = vertex;

        // Determine the owner, looking at the three cells that share the
        // vertex.
        // By convention, the owner is the cell with the lowest numerical index.

        // If the cell is the center child of its parent, it will always have
        // the lowest index of any neighbor, so we can skip determining the
        // owner.
        if resolution == Resolution::Zero
            || bits::get_direction(self.into(), resolution)
                != u8::from(Direction::Center)
        {
            // Get the left neighbor of the vertex, with its rotations.
            let left = vertex.to_direction(self);
            let (left_cell, left_rotation) =
                grid::neighbor_rotations(self, left, 0).expect("left neighbor");
            // Set to owner if lowest index.
            if left_cell < owner {
                owner = left_cell;
            }

            // As above, skip the right neighbor if the left is known lowest
            if resolution == Resolution::Zero
                || bits::get_direction(left_cell.into(), resolution)
                    != u8::from(Direction::Center)
            {
                // Get the right neighbor of the vertex, with its rotations.
                // Note that vertex - 1 is the right side, as vertex numbers are
                // CCW.
                let right_vertex = Vertex::new_unchecked(
                    (u8::from(vertex) + vertex_count - 1) % vertex_count,
                );
                let right = right_vertex.to_direction(self);
                let (right_cell, right_rotation) =
                    grid::neighbor_rotations(self, right, 0)
                        .expect("right neighbor");

                // Set to owner if lowest index.
                if right_cell < owner {
                    owner = right_cell;
                    let direction = if owner.is_pentagon() {
                        grid::direction_for_neighbor(owner, self)
                            .expect("direction to the right")
                    } else {
                        debug_assert_ne!(right, Direction::Center);
                        let offset = (REV_NEIGHBOR_DIRECTIONS_HEX
                            [usize::from(right) - 1]
                            + right_rotation)
                            % NUM_HEX_VERTS;
                        DIRECTIONS[usize::from(offset)]
                    };

                    owner_vertex = direction.vertex(owner);
                }
            }

            // Determine the vertex number for the left neighbor.
            if owner == left_cell {
                let direction = if owner.is_pentagon() {
                    grid::direction_for_neighbor(owner, self)
                        .expect("direction to the left")
                } else {
                    debug_assert_ne!(left, Direction::Center);
                    let offset = (REV_NEIGHBOR_DIRECTIONS_HEX
                        [usize::from(left) - 1]
                        + left_rotation)
                        % NUM_HEX_VERTS;
                    DIRECTIONS[usize::from(offset)]
                };

                // For the left neighbor, we need the second vertex of the
                // edge, which may involve looping around the vertex nums.
                owner_vertex = Vertex::new_unchecked(
                    (u8::from(direction.vertex(owner)) + 1)
                        % owner.vertex_count(),
                );
            }
        }

        // Create the vertex index
        let bits = bits::set_mode(owner.into(), IndexMode::Vertex);
        Some(VertexIndex::new_unchecked(bits::set_vertex(
            bits,
            owner_vertex,
        )))
    }

    /// Returns all vertexes for the current cell.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let vertexes = index.vertexes().collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn vertexes(self) -> impl Iterator<Item = VertexIndex> {
        (0..self.vertex_count()).map(move |vertex| {
            // SAFETY: loop bound ensure valid vertex value.
            let vertex = Vertex::new_unchecked(vertex);
            // We've already filtered out invalid vertex.
            self.vertex(vertex).expect("cell vertex")
        })
    }

    /// Produce cells within grid distance `k` of the cell.
    ///
    /// This function is a convenience helper that tries
    /// [`Self::grid_disk_fast`] first and then fallback on
    /// [`Self::grid_disk_safe`] if the former fails.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells = index.grid_disk::<Vec<_>>(2);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn grid_disk<T>(self, k: u32) -> T
    where
        T: FromIterator<Self>,
    {
        self.grid_disk_fast(k)
            .collect::<Option<T>>()
            .unwrap_or_else(|| self.grid_disk_safe(k).collect())
    }

    /// Safe but slow version of [`Self::grid_disk_fast`].
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells = index.grid_disk_safe(2).collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn grid_disk_safe(self, k: u32) -> impl Iterator<Item = Self> {
        if k == 0 {
            return Either::Right(iter::once(self));
        }
        Either::Left(
            grid::DiskDistancesSafe::new(self, k).map(|(cell, _)| cell),
        )
    }

    /// Produces indexes within grid distance `k` of the cell.
    ///
    /// `0-ring` is defined as the current cell, `1-ring` is defined as
    /// `0-ring` and all neighboring indexes, and so on.
    ///
    /// This function fails (i.e. returns a None item) when a pentagon (or a
    /// pentagon distortion) is encountered.
    /// When this happen, the previously returned cells should be treated as
    /// invalid and discarded.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells = index.grid_disk_fast(2).collect::<Option<Vec<_>>>()
    ///     .unwrap_or_default();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn grid_disk_fast(self, k: u32) -> impl Iterator<Item = Option<Self>> {
        if k == 0 {
            return Either::Right(if self.is_pentagon() {
                Either::Right(iter::once(None))
            } else {
                Either::Left(iter::once(Some(self)))
            });
        }
        Either::Left(
            grid::DiskDistancesUnsafe::new(self, k)
                .map(|value| value.map(|(cell, _)| cell)),
        )
    }

    /// Produce cells and their distances from the current cell, up to distance
    /// `k`.
    ///
    /// This function is a convenience helper that tries
    /// [`Self::grid_disk_distances_fast`] first and then fallback on
    /// [`Self::grid_disk_distances_safe`] if the former fails.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells_and_dists = index.grid_disk_distances::<Vec<_>>(2);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub fn grid_disk_distances<T>(self, k: u32) -> T
    where
        T: FromIterator<(Self, u32)>,
    {
        // Optimistically try the faster fallible algorithm first.
        // If it fails, fall back to the slower always correct one.
        self.grid_disk_distances_fast(k)
            .collect::<Option<T>>()
            .unwrap_or_else(|| self.grid_disk_distances_safe(k).collect())
    }

    /// Safe but slow version of [`Self::grid_disk_distances_fast`].
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells_and_dists = index.grid_disk_distances_safe(2).collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn grid_disk_distances_safe(
        self,
        k: u32,
    ) -> impl Iterator<Item = (Self, u32)> {
        if k == 0 {
            return Either::Right(iter::once((self, 0)));
        }
        Either::Left(grid::DiskDistancesSafe::new(self, k))
    }

    /// Produce cells and their distances from the current cell, up to distance
    /// `k`.
    ///
    /// `0-ring` is defined as the current cell, `1-ring` is defined as `0-ring`
    /// and all neighboring indexes, and so on.
    ///
    /// This function fails (i.e. returns a None item) when a pentagon (or a
    /// pentagon distortion) is encountered.
    /// When this happen, the previously returned items should be treated as
    /// invalid.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells_and_dists = index.grid_disk_distances_fast(2)
    ///     .collect::<Option<Vec<_>>>()
    ///     .unwrap_or_default();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn grid_disk_distances_fast(
        self,
        k: u32,
    ) -> impl Iterator<Item = Option<(Self, u32)>> {
        if k == 0 {
            return Either::Right(if self.is_pentagon() {
                Either::Right(iter::once(None))
            } else {
                Either::Left(iter::once(Some((self, 0))))
            });
        }
        Either::Left(grid::DiskDistancesUnsafe::new(self, k))
    }

    /// Takes an list of cell indexes and a max `k-ring` and returns a stream of
    /// cell indexes sorted first by the original cell index and then by the
    /// grid `k-ring` (0 to max).
    ///
    ///
    /// This function fails (i.e. returns a None item) when a pentagon (or a
    /// pentagon distortion) is encountered.
    /// When this happen, the previously returned cells should be treated as
    /// invalid and discarded.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let indexes = vec![
    ///     CellIndex::try_from(0x8a1fb46622dffff)?,
    ///     CellIndex::try_from(0x8a1fb46622d7fff)?,
    /// ];
    /// let cells = CellIndex::grid_disks_fast(indexes, 2)
    ///     .collect::<Option<Vec<_>>>()
    ///     .unwrap_or_default();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    pub fn grid_disks_fast(
        indexes: impl IntoIterator<Item = Self>,
        k: u32,
    ) -> impl Iterator<Item = Option<Self>> {
        indexes
            .into_iter()
            .flat_map(move |index| index.grid_disk_fast(k))
    }

    /// Returns the "hollow" ring of hexagons at exactly grid distance `k` from
    /// the current cell.
    ///
    /// In particular, k=0 returns just the current hexagon.
    ///
    /// This function fails (i.e. returns a None item) when a pentagon (or a
    /// pentagon distortion) is encountered.
    /// When this happen, the previously returned cells should be treated as
    /// invalid and discarded.
    ///
    /// Failure cases may be fixed in future versions.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let cells = index.grid_ring_fast(2).collect::<Option<Vec<_>>>()
    ///     .unwrap_or_default();
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn grid_ring_fast(self, k: u32) -> impl Iterator<Item = Option<Self>> {
        if k == 0 {
            return Either::Right(iter::once(Some(self)));
        }
        Either::Left(
            grid::RingUnsafe::new(self, k)
                .map_or_else(|| Either::Left(iter::once(None)), Either::Right),
        )
    }

    /// Produces the grid distance between the two indexes.
    ///
    /// # Errors
    ///
    /// This function may fail to find the distance between two indexes, for
    /// example if they are very far apart. It may also fail when finding
    /// distances for indexes on opposite sides of a pentagon.
    /// In such case, [`LocalIjError::Pentagon`] or [`LocalIjError::HexGrid`] is
    /// returned.
    ///
    /// [`LocalIjError::ResolutionMismatch`] if the source and destination
    /// indexes don't have the same resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let src = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let dst = CellIndex::try_from(0x8a1fb46622d7fff)?;
    /// assert_eq!(src.grid_distance(dst)?, 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn grid_distance(self, to: Self) -> Result<i32, LocalIjError> {
        let src = self.to_local_ijk(self)?;
        let dst = to.to_local_ijk(self)?;

        Ok(src.coord().distance(dst.coord()))
    }

    /// Computes the number of indexes in a line from the current index to the
    /// end one.
    ///
    /// To be used for pre-allocating memory for `CellIndex::grid_path_cells`.
    ///
    /// # Errors
    ///
    /// See [`Self::grid_distance`].
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let src = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let dst = CellIndex::try_from(0x8a1fb46622d7fff)?;
    /// assert_eq!(src.grid_path_cells_size(dst)?, 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn grid_path_cells_size(self, to: Self) -> Result<i32, LocalIjError> {
        self.grid_distance(to).map(|distance| distance + 1)
    }

    /// Given two H3 indexes, return the line of indexes between them
    /// (inclusive).
    ///
    /// # Notes
    ///
    ///  - The specific output of this function should not be considered stable
    ///    across library versions. The only guarantees the library provides are
    ///    that the line length will be `start.grid_distance(end) + 1` and that
    ///    every index in the line will be a neighbor of the preceding index.
    ///  - Lines are drawn in grid space, and may not correspond exactly to
    ///    either Cartesian lines or great arcs.
    ///
    /// # Errors
    ///
    /// This function may fail to find the distance between two indexes, for
    /// example if they are very far apart. It may also fail when finding
    /// distances for indexes on opposite sides of a pentagon.
    /// In such case, [`LocalIjError::Pentagon`] or [`LocalIjError::HexGrid`] is
    /// returned.
    ///
    /// [`LocalIjError::ResolutionMismatch`] if the source and destination
    /// indexes don't have the same resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let src = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let dst = CellIndex::try_from(0x8a1fb46622d7fff)?;
    /// let cells = src.grid_path_cells(dst)?.collect::<Result<Vec<_>, _>>()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn grid_path_cells(
        self,
        to: Self,
    ) -> Result<impl Iterator<Item = Result<Self, LocalIjError>>, LocalIjError>
    {
        GridPathCells::new(self, to)
    }

    /// Returns whether or not the provided cell index is a neighbor of the
    /// current one.
    ///
    /// # Errors
    ///
    /// [`ResolutionMismatch`] if the two indexes don't have the
    /// same resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let src = CellIndex::try_from(0x8a1fb46622dffff)?;
    /// let dst = CellIndex::try_from(0x8a1fb46622d7fff)?;
    /// assert!(src.is_neighbor_with(dst)?);
    ///
    /// let dst = CellIndex::try_from(0x8a1fb4644937fff)?;
    /// assert!(!src.is_neighbor_with(dst)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_neighbor_with(
        self,
        index: Self,
    ) -> Result<bool, ResolutionMismatch> {
        // A cell cannot be neighbors with itself.
        if self == index {
            return Ok(false);
        }

        // Only indexes in the same resolution can be neighbors.
        let cur_res = self.resolution();
        let idx_res = index.resolution();
        if cur_res != idx_res {
            return Err(ResolutionMismatch);
        }

        // Cell indexes that share the same parent are very likely to be
        // neighbors
        //
        // Child 0 is neighbor with all of its parent's 'offspring', the other
        // children are neighbors with 3 of the 7 children. So a simple
        // comparison of origin and destination parents and then a lookup table
        // of the children is a super-cheap way to possibly determine if they
        // are neighbors.
        if cur_res > Resolution::Zero {
            // `cur_res` cannot be 0, thus we can always get the parent
            // resolution.
            let parent_res = cur_res.pred().expect("parent resolution");
            let cur_parent = self.parent(parent_res).expect("current's parent");
            let idx_parent = index.parent(parent_res).expect("index's parent");
            if cur_parent == idx_parent {
                let cur_res_digit = bits::get_direction(self.into(), cur_res);
                let idx_res_digit = bits::get_direction(index.into(), idx_res);
                if cur_res_digit == u8::from(Direction::Center)
                    || idx_res_digit == u8::from(Direction::Center)
                {
                    return Ok(true);
                }

                if u8::from(NEIGHBOR_SET_CW[usize::from(cur_res_digit)])
                    == idx_res_digit
                    || u8::from(NEIGHBOR_SET_CCW[usize::from(cur_res_digit)])
                        == idx_res_digit
                {
                    return Ok(true);
                }
            }
        }

        // Otherwise, we have to determine the neighbor relationship the "hard"
        // way.
        Ok(self
            .grid_disk_fast(1)
            .try_fold(false, |acc, item| {
                item.map(|neighbor| acc || index == neighbor)
            })
            .unwrap_or_else(|| {
                self.grid_disk_safe(1).any(|neighbor| index == neighbor)
            }))
    }

    /// Produces `IJ` coordinates for an index anchored by an origin.
    ///
    /// The coordinate space used by this function may have deleted regions or
    /// warping due to pentagonal distortion.
    ///
    /// Coordinates are only comparable if they come from the same origin index.
    ///
    /// This function's output is not guaranteed to be compatible across
    /// different versions of H3.
    ///
    /// # Arguments
    ///
    /// * `origin` - An anchoring index for the `IJ` coordinate system.
    /// * `index` - Index to find the coordinates of.
    ///
    /// # Errors
    ///
    /// [`LocalIjError::ResolutionMismatch`] if the index and the origin don't
    /// have the same resolution.
    ///
    /// Failure may occur if the index is too far away from the origin or if the
    /// index is on the other side of a pentagon.
    /// In such case, [`LocalIjError::Pentagon`] or [`LocalIjError::HexGrid`] is
    /// returned.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let anchor = CellIndex::try_from(0x823147fffffffff)?;
    /// let index = CellIndex::try_from(0x8230e7fffffffff)?;
    /// let localij = index.to_local_ij(anchor)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_local_ij(self, origin: Self) -> Result<LocalIJ, LocalIjError> {
        let lijk = self.to_local_ijk(origin)?;
        let coord = CoordIJ::from(&lijk.coord);
        Ok(LocalIJ::new(lijk.anchor, coord))
    }

    /// Returns the next cell, in term of ordering.
    ///
    /// Returns `None` if `self` is the last cell at this resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let start = CellIndex::try_from(0x823147fffffffff)?;
    /// let after = start.succ().expect("next cell index");
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn succ(self) -> Option<Self> {
        // Bitmask to detect IJ (6) directions.
        const IJ_MASK: u64 = 0o666666666666666;

        let resolution = self.resolution();
        let res_offset = self.resolution().direction_offset();
        // Shift to get rid of unused directions.
        let mut bits = u64::from(self) >> res_offset;

        // Find the first non-IJ direction (e.g. can be ++ w/o carry).
        // First in term of bit offset, then convert to resolution offset.
        let bitpos = (bits ^ IJ_MASK).trailing_zeros() as usize;
        let respos = bitpos / h3o_bit::DIRECTION_BITSIZE;

        // Clear directions affected by the carry propagation.
        let mask = !((1 << (respos * h3o_bit::DIRECTION_BITSIZE)) - 1);
        bits &= mask;

        // Restore unused direction.
        bits = bits::set_unused(bits << res_offset, resolution);

        // If the carry stopped before the base cell, we simply increment.
        if respos < usize::from(resolution) {
            // Everything is ready, we can increment now.
            let one = 1 << (res_offset + respos * h3o_bit::DIRECTION_BITSIZE);
            bits += one;
            // Skip deleted sub-sequence.
            return Some(Self::try_from(bits).unwrap_or_else(|_| {
                bits += one;
                Self::new_unchecked(bits)
            }));
        }

        // We moved onto another base cell.
        let base_cell = u8::from(self.base_cell());
        (base_cell != 121)
            .then(|| h3o_bit::set_base_cell(bits, base_cell + 1))
            .map(Self::new_unchecked)
    }

    /// Returns the previous cell, in term of ordering.
    ///
    /// Returns `None` if `self` is the frist cell at this resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::CellIndex;
    ///
    /// let start = CellIndex::try_from(0x823147fffffffff)?;
    /// let before = start.pred().expect("next cell index");
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    pub fn pred(self) -> Option<Self> {
        let resolution = self.resolution();
        let res_offset = self.resolution().direction_offset();
        // Shift to get rid of unused directions.
        let mut bits = u64::from(self) >> res_offset;

        // Find the first non-zero direction (e.g. can be -- w/o carry).
        // First in term of bit offset, then convert to resolution offset.
        let bitpos = bits.trailing_zeros() as usize;
        let respos = bitpos / h3o_bit::DIRECTION_BITSIZE;

        // Set directions affected by the carry propagation.
        let mask = (1 << (respos * h3o_bit::DIRECTION_BITSIZE)) - 1;
        bits |= 0o666666666666666 & mask;

        // Restore unused direction.
        bits = bits::set_unused(bits << res_offset, resolution);

        // If the carry stopped before the base cell, we simply decrement.
        if respos < usize::from(resolution) {
            // Everything is ready, we can decrement now.
            let one = 1 << (res_offset + respos * h3o_bit::DIRECTION_BITSIZE);
            bits -= one;
            // Skip deleted sub-sequence.
            return Some(Self::try_from(bits).unwrap_or_else(|_| {
                bits -= one;
                Self::new_unchecked(bits)
            }));
        }

        // We moved onto another base cell.
        let base_cell = u8::from(self.base_cell());
        (base_cell != 0)
            .then(|| h3o_bit::set_base_cell(bits, base_cell - 1))
            .map(Self::new_unchecked)
    }

    /// The first cell index at the given resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let first = CellIndex::first(Resolution::Nine);
    /// ```
    #[must_use]
    pub fn first(resolution: Resolution) -> Self {
        let bits = bits::set_resolution(0x0800_0000_0000_0000, resolution);
        Self::new_unchecked(bits::set_unused(bits, resolution))
    }

    /// The last cell index at the given resolution.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::{CellIndex, Resolution};
    ///
    /// let last = CellIndex::last(Resolution::Nine);
    /// ```
    #[must_use]
    pub fn last(resolution: Resolution) -> Self {
        let bits = bits::set_resolution(0x080f_3b6d_b6db_6db6, resolution);
        Self::new_unchecked(bits::set_unused(bits, resolution))
    }

    pub(crate) fn new_unchecked(value: u64) -> Self {
        debug_assert!(Self::try_from(value).is_ok(), "invalid cell index");
        Self(NonZeroU64::new(value).expect("valid cell index"))
    }

    /// Get the number of CCW rotations of the cell's vertex numbers compared to
    /// the directional layout of its neighbors.
    pub(crate) fn vertex_rotations(self) -> u8 {
        // Get the face and other info for the origin.
        let fijk = FaceIJK::from(self);
        let base_cell = self.base_cell();
        let leading_dir = bits::first_axe(self.into());

        // Get the base cell face.
        let base_fijk = FaceIJK::from(base_cell);
        let mut ccw_rot60 = base_cell.rotation_count(fijk.face);

        if base_cell.is_pentagon() {
            let jk = Direction::JK.axe().expect("JK");
            let ik = Direction::IK.axe().expect("IK");

            // Find the appropriate direction-to-face mapping
            let dir_faces = base_cell.pentagon_direction_faces();

            // Additional CCW rotation for polar neighbors or IK neighbors.
            if fijk.face != base_fijk.face
                && (base_cell.is_polar_pentagon()
                    || fijk.face == dir_faces[usize::from(ik.get()) - 2])
            {
                ccw_rot60 = (ccw_rot60 + 1) % 6;
            }

            // Check whether the cell crosses a deleted pentagon subsequence.
            if leading_dir == Some(jk)
                && fijk.face == dir_faces[usize::from(ik.get()) - 2]
            {
                // Crosses from JK to IK => Rotate CW.
                ccw_rot60 = (ccw_rot60 + 5) % 6;
            } else if leading_dir == Some(ik)
                && fijk.face == dir_faces[usize::from(jk.get()) - 2]
            {
                // Crosses from IK to JK => Rotate CCW.
                ccw_rot60 = (ccw_rot60 + 1) % 6;
            }
        }

        ccw_rot60
    }

    /// Produces `IJK` coordinates for an index anchored by an `origin`.
    ///
    /// The coordinate space used by this function may have deleted regions or
    /// warping due to pentagonal distortion.
    ///
    /// # Arguments
    ///
    /// * `origin` - An anchoring index for the `IJK` coordinate system.
    /// * `index` - Index to find the coordinates of.
    ///
    /// # Errors
    ///
    /// `LocalIJError::ResolutionMismatch` if the index and the origin don't
    /// have the same resolution.
    ///
    /// Failure may occur if the index is too far away from the origin or if the
    /// index is on the other side of a pentagon.
    /// In such case, [`LocalIjError::Pentagon`] or [`LocalIjError::HexGrid`] is
    /// returned.
    pub(super) fn to_local_ijk(
        mut self,
        origin: Self,
    ) -> Result<LocalIJK, LocalIjError> {
        let origin_res = origin.resolution();
        let index_res = self.resolution();

        if origin_res != index_res {
            return Err(LocalIjError::ResolutionMismatch);
        }
        let origin_base_cell = origin.base_cell();
        let base_cell = self.base_cell();

        // Direction from origin base cell to index base cell.
        let mut dir = Direction::Center;
        let mut rev_dir = if origin_base_cell == base_cell {
            Direction::Center
        } else {
            dir = origin_base_cell.direction(base_cell).ok_or_else(|| {
                HexGridError::new(
                    "cannot unfold (base cells are not neighbors)",
                )
            })?;
            base_cell
                .direction(origin_base_cell)
                .expect("reverse direction")
        };

        let origin_on_pent = origin_base_cell.is_pentagon();
        let index_on_pent = base_cell.is_pentagon();

        if dir != Direction::Center {
            // Rotate index into the orientation of the origin base cell.
            // CW because we are undoing the rotation into that base cell.
            let base_cell_rotations =
                origin_base_cell.neighbor_rotation(dir).into();
            self = Self::new_unchecked(if index_on_pent {
                (0..base_cell_rotations).fold(self.into(), |acc, _| {
                    // If the rotation will fall on the K axe, rotate twice to
                    // skip it.
                    rev_dir = if rev_dir == Direction::IK {
                        rev_dir.rotate60::<CW>(2)
                    } else {
                        rev_dir.rotate60_once::<CW>()
                    };

                    bits::pentagon_rotate60::<CW>(acc)
                })
            } else {
                rev_dir = rev_dir.rotate60::<CW>(base_cell_rotations);
                bits::rotate60::<CW>(self.into(), base_cell_rotations)
            });
        }

        // This produces coordinates in base cell coordinate space (face is
        // unused).
        let mut ijk = FaceIJK::from_bits(self.into(), index_res, BASE_CELL)
            .0
            .coord;

        if dir != Direction::Center {
            debug_assert_ne!(base_cell, origin_base_cell);
            debug_assert!(!(origin_on_pent && index_on_pent));

            let (pentagon_rotations, direction_rotations) = if origin_on_pent {
                let leading_dir = bits::first_axe(origin.into())
                    .map_or_else(|| 0, NonZeroU8::get);
                validate_direction(leading_dir, dir.into())?;

                let rotations = PENTAGON_ROTATIONS[usize::from(leading_dir)]
                    [usize::from(dir)];
                (rotations, rotations)
            } else if index_on_pent {
                let leading_dir = bits::first_axe(self.into())
                    .map_or_else(|| 0, NonZeroU8::get);
                validate_direction(leading_dir, rev_dir.into())?;

                (
                    PENTAGON_ROTATIONS[usize::from(rev_dir)]
                        [usize::from(leading_dir)],
                    0,
                )
            } else {
                (0, 0)
            };
            // No check on `direction_rotations`, it's either 0 or
            // `pentagon_rotations`.
            debug_assert!(pentagon_rotations != 0xff);

            ijk = (0..pentagon_rotations)
                .fold(ijk, |acc, _| acc.rotate60::<CW>());

            let mut offset = CoordIJK::new(0, 0, 0).neighbor(dir);
            // Scale offset based on resolution
            for res in Resolution::range(Resolution::One, origin_res).rev() {
                // SAFETY: `res` always is range thanks to loop bound.
                offset = if res.is_class3() {
                    // rotate CCW.
                    offset.down_aperture7::<CCW>()
                } else {
                    // rotate CW.
                    offset.down_aperture7::<CW>()
                };
            }

            offset = (0..direction_rotations)
                .fold(offset, |acc, _| acc.rotate60::<CW>());

            // Perform necessary translation
            ijk = (ijk + offset).normalize();
        } else if origin_on_pent && index_on_pent {
            // If the origin and index are on pentagon, and we checked that the
            // base cells are the same or neighboring, then they must be the
            // same base cell.
            debug_assert_eq!(base_cell, origin_base_cell);

            let origin_leading_dir = bits::first_axe(origin.into())
                .map_or_else(|| 0, NonZeroU8::get);
            let index_leading_dir =
                bits::first_axe(self.into()).map_or_else(|| 0, NonZeroU8::get);
            validate_direction(origin_leading_dir, index_leading_dir)?;

            let rotations = PENTAGON_ROTATIONS[usize::from(origin_leading_dir)]
                [usize::from(index_leading_dir)];

            ijk = (0..rotations).fold(ijk, |acc, _| acc.rotate60::<CW>());
        }

        Ok(LocalIJK {
            anchor: origin,
            coord: ijk,
        })
    }

    fn vertex_count(self) -> u8 {
        if self.is_pentagon() {
            NUM_PENT_VERTS
        } else {
            NUM_HEX_VERTS
        }
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
        (h3o_bit::clr_resolution(self.0.get()))
            .cmp(&h3o_bit::clr_resolution(other.0.get()))
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

impl From<CellIndex> for LatLng {
    /// Determines the spherical coordinates of the center point of an H3 index.
    fn from(value: CellIndex) -> Self {
        FaceIJK::from(value).to_latlng(value.resolution())
    }
}

impl From<CellIndex> for FaceIJK {
    /// Converts an `H3Index` to a `FaceIJK` address.
    fn from(value: CellIndex) -> Self {
        let mut bits = value.0.get();
        let base_cell = value.base_cell();
        let resolution = value.resolution();

        // Adjust for the pentagonal missing sequence; all of sub-sequence 5
        // needs to be adjusted (and some of sub-sequence 4 below).
        if base_cell.is_pentagon()
            && bits::first_axe(bits) == Direction::IK.axe()
        {
            bits = bits::rotate60::<{ CW }>(bits, 1);
        }

        // Start with the "home" face and `IJK` coordinates for the base cell.
        //
        // XXX: Because of the adjustment above, cell may have become an invalid
        // pentagon, so we need to works on the raw bits here.
        let (mut fijk, overage) = Self::from_bits(bits, resolution, base_cell);
        if !overage {
            return fijk;
        }

        // If we're here we have the potential for an "overage"; i.e., it is
        // possible that the index lies on an adjacent face.

        let original_coord = fijk.coord;

        // If we're in Class III, drop into the next finer Class II grid.
        let class2_res = if resolution.is_class3() {
            fijk.coord = fijk.coord.down_aperture7::<{ CW }>();
            ExtendedResolution::down(resolution)
        } else {
            resolution.into()
        };

        // Adjust for overage if needed.
        // A pentagon base cell with a leading 4 digit requires special
        // handling.
        let is_pent4 = base_cell.is_pentagon()
            && bits::first_axe(bits) == Direction::I.axe();

        if fijk.adjust_overage_class2::<{ CW }>(class2_res, is_pent4)
            != Overage::None
        {
            // If the base cell is a pentagon we have the potential for
            // secondary overages.
            if base_cell.is_pentagon() {
                while fijk.adjust_overage_class2::<{ CW }>(class2_res, false)
                    != Overage::None
                {}
            }

            if class2_res != resolution.into() {
                fijk.coord = fijk.coord.up_aperture7::<{ CW }>();
            }
        } else if class2_res != resolution.into() {
            fijk.coord = original_coord;
        }

        fijk
    }
}

impl TryFrom<u64> for CellIndex {
    type Error = InvalidCellIndex;

    // Basically a simpler/faster version of `h3IsValid`.
    //
    // Simpler because here we focus only on the trailing 56-bit part.
    // Faster because no loops, just plain ol' bitwise operations :)
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if (value >> 56) & 0b1000_0111 != 0 {
            return Err(Self::Error::new(Some(value), "tainted reserved bits"));
        }
        if bits::get_mode(value) != u8::from(IndexMode::Cell) {
            return Err(Self::Error::new(Some(value), "invalid index mode"));
        }

        let base = BaseCell::try_from(h3o_bit::get_base_cell(value))
            .map_err(|_| Self::Error::new(Some(value), "invalid base cell"))?;

        // Resolution is always valid: coded on 4 bits, valid range is [0; 15].
        let resolution = usize::from(bits::get_resolution(value));

        // Check that we have a tail of unused cells  after `resolution` cells.
        //
        // We expect every bit to be 1 in the tail (because unused cells are
        // represented by `0b111`), i.e. every bit set to 0 after a NOT.
        let unused_count = usize::from(h3o_bit::MAX_RESOLUTION) - resolution;
        let unused_bitsize = unused_count * h3o_bit::DIRECTION_BITSIZE;
        let unused_mask = (1 << unused_bitsize) - 1;
        if (!value) & unused_mask != 0 {
            return Err(Self::Error::new(
                Some(value),
                "invalid unused direction pattern",
            ));
        }

        // Check that we have `resolution` valid cells (no unused ones).
        let dirs_mask = (1 << (resolution * h3o_bit::DIRECTION_BITSIZE)) - 1;
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
            let offset = 64 - (resolution * h3o_bit::DIRECTION_BITSIZE);

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
        write!(f, "{self:x}")
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

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for CellIndex {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        u64::arbitrary(data).and_then(|byte| {
            Self::try_from(byte).map_err(|_| arbitrary::Error::IncorrectFormat)
        })
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
