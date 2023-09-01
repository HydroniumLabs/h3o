//! The H3 system centers an `IJK` coordinate system on each face of the
//! icosahedron; the combination of a face number and `IJK` coordinates on that
//! face's coordinate system is represented using the structure type `FaceIJK`.
//!
//! Each grid resolution is rotated ~19.1° relative to the next coarser
//! resolution. The rotation alternates between counterclockwise and clockwise
//! at each successive resolution, so that each resolution will have one of two
//! possible orientations: Class II or Class III (using a terminology coined by
//! R. Buckminster Fuller). The base cells, which make up resolution 0, are
//! Class II.

use super::{CoordIJK, Vec2d, SQRT3_2};
use crate::{
    face::{self, FaceOrientIJK},
    index::bits,
    BaseCell, Boundary, CellIndex, Direction, ExtendedResolution, Face, LatLng,
    Resolution, Vertex, CCW, CW, DEFAULT_CELL_INDEX, NUM_HEX_VERTS,
    NUM_ICOSA_FACES, NUM_PENT_VERTS,
};

static ZERO: CoordIJK = CoordIJK::new(0, 0, 0);

// -----------------------------------------------------------------------------

/// Face number and `IJK` coordinates on that face-centered coordinate system.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct FaceIJK {
    /// Face number.
    pub face: Face,
    /// `ijk` coordinates on that face.
    pub coord: CoordIJK,
}

impl FaceIJK {
    pub const fn new(face: Face, coord: CoordIJK) -> Self {
        Self { face, coord }
    }

    /// Returns the number of 60° counterclockwise rotations to rotate into the
    /// coordinate system of the base cell at that coordinates.
    #[allow(clippy::cast_sign_loss)] // Safe because components values are in [0; 2].
    pub fn base_cell_rotation(&self) -> Rotation {
        // Should always be the case, or we have a nasty bug to fix.
        debug_assert!(
            (0..=2).contains(&self.coord.i())
                && (0..=2).contains(&self.coord.j())
                && (0..=2).contains(&self.coord.k())
        );

        let f = usize::from(self.face);
        let i = self.coord.i() as usize;
        let j = self.coord.j() as usize;
        let k = self.coord.k() as usize;

        FACE_IJK_BASE_CELLS[f][i][j][k]
    }

    /// Converts a `FaceIJK` address to the corresponding [`CellIndex`].
    pub(crate) fn to_cell(mut self, resolution: Resolution) -> CellIndex {
        // Initialize the index.
        let mut bits = bits::set_resolution(DEFAULT_CELL_INDEX, resolution);

        // Handle resolution 0 (base cell).
        if resolution == Resolution::Zero {
            let rotation = self.base_cell_rotation();
            return CellIndex::new_unchecked(h3o_bit::set_base_cell(
                bits,
                rotation.base_cell.into(),
            ));
        }

        // We need to find the correct base cell FaceIJK for this H3 index

        // Build the index from finest resolution up.
        self.coord =
            directions_bits_from_ijk(self.coord, &mut bits, resolution);

        // Lookup the correct base cell.
        let rotation = self.base_cell_rotation();
        bits = h3o_bit::set_base_cell(bits, rotation.base_cell.into());

        // Rotate if necessary to get canonical base cell orientation
        // for this base cell
        if rotation.base_cell.is_pentagon() {
            // Force rotation out of missing k-axes sub-sequence
            if bits::first_axe(bits) == Direction::K.axe() {
                // Check for a CW/CCW offset face (default is CCW).
                if rotation.base_cell.is_cw_offset(self.face) {
                    bits = bits::rotate60::<{ CW }>(bits, 1);
                } else {
                    bits = bits::rotate60::<{ CCW }>(bits, 1);
                }
            }

            for _ in 0..rotation.count {
                bits = bits::pentagon_rotate60::<{ CCW }>(bits);
            }
        } else {
            bits = bits::rotate60::<{ CCW }>(bits, rotation.count.into());
        }

        CellIndex::new_unchecked(bits)
    }

    /// Determines the center point in spherical coordinates of a cell given by
    /// a `FaceIJK` address at a specified resolution.
    ///
    /// # Arguments
    ///
    /// * `fijk` - The `FaceIJK` address of the cell.
    /// * `resolution` - The H3 resolution of the cell.
    pub fn to_latlng(self, resolution: Resolution) -> LatLng {
        Vec2d::from(self.coord).to_latlng(self.face, resolution.into(), false)
    }

    /// Returns the `FaceIJK` address that correspond to a given cell index.
    ///
    /// # Arguments
    ///
    /// * `bits` - Raw bits of the cell index.
    /// * `resolution` - Resolution of the cell index.
    /// * `base_cell` - Base cell of the cell index.
    pub fn from_bits(
        bits: u64,
        resolution: Resolution,
        base_cell: BaseCell,
    ) -> (Self, bool) {
        let mut fijk = Self::from(base_cell);
        let possible_overage = base_cell.is_pentagon()
            || (resolution != Resolution::Zero || fijk.coord != ZERO);

        for res in Resolution::range(Resolution::One, resolution) {
            if res.is_class3() {
                // Class III == rotate CCW.
                fijk.coord = fijk.coord.down_aperture7::<{ CCW }>();
            } else {
                // Class II == rotate CW.
                fijk.coord = fijk.coord.down_aperture7::<{ CW }>();
            }

            // SAFETY: loop upper bound is the index resolution.
            let direction =
                Direction::new_unchecked(bits::get_direction(bits, res));
            fijk.coord = fijk.coord.neighbor(direction);
        }

        (fijk, possible_overage)
    }

    /// Adjusts a `FaceIJK` address in place so that the resulting cell address
    /// is relative to the correct icosahedral face.
    ///
    /// We usually assume that the cell center point falls on the same
    /// icosahedron face as its base cell, but it is possible that the cell
    /// center point lies on an adjacent face (that's what we call an overage in
    /// the code), in which case we need to use a projection centered on that
    /// adjacent face instead.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The H3 resolution of the cell.
    /// * `is_pent4` - Whether or not the cell is a pentagon with a leading
    ///                digit 4.
    /// * `in_substrate` - Whether or not the cell is in a substrate grid.
    pub fn adjust_overage_class2<const IS_SUBSTRATE: bool>(
        &mut self,
        class2_res: ExtendedResolution,
        is_pent4: bool,
    ) -> Overage {
        let class2_res = usize::from(class2_res);
        let factor = if IS_SUBSTRATE { 3 } else { 1 };
        let face = usize::from(self.face);
        let dimension = self.coord.i() + self.coord.j() + self.coord.k();

        // Get the maximum dimension value; scale if a substrate grid.
        let max_dim = MAX_DIM_BY_CII_RES[class2_res] * factor;

        // Check for overage.
        if IS_SUBSTRATE && dimension == max_dim {
            return Overage::FaceEdge;
        }

        if dimension > max_dim {
            let orientation = if self.coord.k() > 0 {
                if self.coord.j() > 0 {
                    face::NEIGHBORS[face][face::JK]
                } else {
                    // Adjust for the pentagonal missing sequence.
                    if is_pent4 {
                        // Translate origin to center of pentagon and rotate to
                        // adjust for the missing sequence.
                        let origin = CoordIJK::new(max_dim, 0, 0);
                        let tmp = (self.coord - origin).rotate60::<{ CW }>();
                        // Translate the origin back to the center of the triangle
                        self.coord = tmp + origin;
                    }
                    face::NEIGHBORS[face][face::KI]
                }
            } else {
                face::NEIGHBORS[face][face::IJ]
            };
            self.face = orientation.face;

            // Rotate and translate for adjacent face.
            for _ in 0..orientation.ccw_rot60 {
                self.coord = self.coord.rotate60::<{ CCW }>();
            }

            let mut trans_vec = orientation.translate;
            let unit_scale = UNIT_SCALE_BY_CII_RES[class2_res] * factor;
            trans_vec *= unit_scale;
            self.coord = (self.coord + trans_vec).normalize();

            // Overage points on pentagon boundaries can end up on edges.
            if IS_SUBSTRATE
                && self.coord.i() + self.coord.j() + self.coord.k() == max_dim
            {
                return Overage::FaceEdge;
            }
            return Overage::NewFace;
        }

        Overage::None
    }

    /// Adjusts a `FaceIJK` address for a pentagon vertex in a substrate grid so
    /// that the resulting cell address is relative to the correct icosahedral
    /// face.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The H3 resolution of the cell.
    pub fn adjust_pentagon_vertex_overage(
        &mut self,
        resolution: ExtendedResolution,
    ) {
        while self.adjust_overage_class2::<true>(resolution, false)
            == Overage::NewFace
        {}
    }

    /// Generates the cell boundary in spherical coordinates for a pentagonal
    /// cell given by a `FaceIJK` address at a specified resolution.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The H3 resolution of the cell.
    /// * `start` - The first topological vertex to return.
    /// * `length` - The number of topological vertexes to return.
    pub fn pentagon_boundary(
        &self,
        resolution: Resolution,
        start: Vertex,
        length: u8,
    ) -> Boundary {
        let mut boundary = Boundary::new();
        let start = u8::from(start);
        let mut center = *self;
        let mut vertices = [Self::default(); NUM_PENT_VERTS as usize];
        let adjusted_resolution = center.vertices(resolution, &mut vertices);

        // If we're returning the entire loop, we need one more iteration in case
        // of a distortion vertex on the last edge.
        let additional_iteration = u8::from(length == NUM_PENT_VERTS);

        // Convert each vertex to lat/lng.
        // Adjust the face of each vertex as appropriate and introduce
        // edge-crossing vertices as needed.
        let mut last_fijk = Self::default();
        for vert in start..(start + length + additional_iteration) {
            let mut fijk: Self = vertices[usize::from(vert % NUM_PENT_VERTS)];
            fijk.adjust_pentagon_vertex_overage(adjusted_resolution);

            // All Class III pentagon edges cross icosahedron edges.
            //
            // Note that Class II pentagons have vertices on the edge,
            // not edge intersections.
            if resolution.is_class3() && vert > start {
                // Find hex2d of the two vertexes on the last face.
                let mut tmp_fijk = fijk;

                let orig2d0 = Vec2d::from(last_fijk.coord);
                let current_to_last_dir: u8 =
                    get_adjacent_face_dir(tmp_fijk.face, last_fijk.face);

                let fijk_orientation: &FaceOrientIJK = &face::NEIGHBORS
                    [usize::from(tmp_fijk.face)]
                    [usize::from(current_to_last_dir)];

                tmp_fijk.face = fijk_orientation.face;

                // Rotate and translate for adjacent face.
                for _ in 0..fijk_orientation.ccw_rot60 {
                    tmp_fijk.coord = tmp_fijk.coord.rotate60::<{ CCW }>();
                }
                let mut trans_vec = fijk_orientation.translate;
                trans_vec *=
                    UNIT_SCALE_BY_CII_RES[usize::from(adjusted_resolution)] * 3;
                tmp_fijk.coord = (tmp_fijk.coord + trans_vec).normalize();

                let orig2d1 = Vec2d::from(tmp_fijk.coord);

                // Find the appropriate icosahedron face edge vertexes.
                let max_dim = f64::from(
                    MAX_DIM_BY_CII_RES[usize::from(adjusted_resolution)],
                );
                let v0 = Vec2d::new(3.0 * max_dim, 0.0);
                let v1 = Vec2d::new(-1.5 * max_dim, 3.0 * SQRT3_2 * max_dim);
                let v2 = Vec2d::new(-1.5 * max_dim, -3.0 * SQRT3_2 * max_dim);

                let (edge0, edge1) = match usize::from(get_adjacent_face_dir(
                    tmp_fijk.face,
                    fijk.face,
                )) {
                    face::IJ => (v0, v1),
                    face::JK => (v1, v2),
                    face::KI => (v2, v0),
                    _ => unreachable!("invalid face direction"),
                };

                // Find the intersection and add the lat/lng point to the
                // result.
                let intersection =
                    Vec2d::intersection((orig2d0, orig2d1), (edge0, edge1));
                boundary.push(intersection.to_latlng(
                    tmp_fijk.face,
                    adjusted_resolution,
                    true,
                ));
            }

            // convert vertex to lat/lng and add to the result
            // vert == start + NUM_PENT_VERTS is only used to test for possible
            // intersection on last edge
            if vert < start + NUM_PENT_VERTS {
                boundary.push(Vec2d::from(fijk.coord).to_latlng(
                    fijk.face,
                    adjusted_resolution,
                    true,
                ));
            }

            last_fijk = fijk;
        }

        boundary
    }

    /// Generates the cell boundary in spherical coordinates for a cell given by
    /// a `FaceIJK` address at a specified resolution.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The H3 resolution of the cell.
    /// * `start` - The first topological vertex to return.
    /// * `length` - The number of topological vertexes to return.
    pub fn hexagon_boundary(
        &self,
        resolution: Resolution,
        start: Vertex,
        length: u8,
    ) -> Boundary {
        let mut boundary = Boundary::new();
        let start = u8::from(start);
        let mut center = *self;
        let mut vertices = [Self::default(); NUM_HEX_VERTS as usize];
        let adjusted_resolution = center.vertices(resolution, &mut vertices);

        // If we're returning the entire loop, we need one more iteration in
        // case of a distortion vertex on the last edge.
        let additional_iteration = u8::from(length == NUM_HEX_VERTS);

        // Convert each vertex to lat/lng.
        // Adjust the face of each vertex as appropriate and introduce
        // edge-crossing vertices as needed.

        let mut last_face = usize::MAX;
        let mut last_overage = Overage::None;
        for vert in start..(start + length + additional_iteration) {
            let v = usize::from(vert % NUM_HEX_VERTS);
            let mut fijk = vertices[v];
            let overage =
                fijk.adjust_overage_class2::<true>(adjusted_resolution, false);

            // Check for edge-crossing.
            //
            // Each face of the underlying icosahedron is a different projection
            // plane. So if an edge of the hexagon crosses an icosahedron edge,
            // an additional vertex must be introduced at that intersection
            // point. Then each half of the cell edge can be projected to
            // geographic coordinates using the appropriate icosahedron face
            // projection.
            // Note that Class II cell edges have vertices on the face edge,
            // with no edge line intersections.
            if resolution.is_class3()
                && vert > start
                && usize::from(fijk.face) != last_face
                && last_overage != Overage::FaceEdge
            {
                // Find hex2d of the two vertexes on original face.
                let last_v: usize = (v + 5) % usize::from(NUM_HEX_VERTS);
                let orig2d0 = Vec2d::from(vertices[last_v].coord);
                let orig2d1 = Vec2d::from(vertices[v].coord);

                // Find the appropriate icosahedron face edge vertexes.
                let max_dim = f64::from(
                    MAX_DIM_BY_CII_RES[usize::from(adjusted_resolution)],
                );
                let v0 = Vec2d::new(3.0 * max_dim, 0.0);
                let v1 = Vec2d::new(-1.5 * max_dim, 3.0 * SQRT3_2 * max_dim);
                let v2 = Vec2d::new(-1.5 * max_dim, -3.0 * SQRT3_2 * max_dim);

                let face2 = if last_face == usize::from(center.face) {
                    fijk.face
                } else {
                    Face::new_unchecked(last_face)
                };
                let (edge0, edge1) = match usize::from(get_adjacent_face_dir(
                    center.face,
                    face2,
                )) {
                    face::IJ => (v0, v1),
                    face::JK => (v1, v2),
                    face::KI => (v2, v0),
                    _ => unreachable!("invalid adjacent face direction"),
                };

                // Find the intersection and add the lat/lng point to the result
                let intersection =
                    Vec2d::intersection((orig2d0, orig2d1), (edge0, edge1));
                // If a point of intersection occurs at a hexagon vertex, then
                // each adjacent hexagon edge will lie completely on a single
                // icosahedron face, and no additional vertex is required.
                let is_intersection_at_vertex =
                    orig2d0 == intersection || orig2d1 == intersection;
                if !is_intersection_at_vertex {
                    boundary.push(intersection.to_latlng(
                        center.face,
                        adjusted_resolution,
                        true,
                    ));
                }
            }

            // Convert vertex to lat/lng and add to the result.
            //
            // `vert == start + NUM_HEX_VERTS` is only used to test for possible
            // intersection on last edge.
            if vert < start + NUM_HEX_VERTS {
                boundary.push(Vec2d::from(fijk.coord).to_latlng(
                    fijk.face,
                    adjusted_resolution,
                    true,
                ));
            }

            last_face = fijk.face.into();
            last_overage = overage;
        }

        boundary
    }

    /// Returns the vertices of a cell as substrate `FaceIJK` addresses.
    ///
    /// # Arguments
    ///
    /// * `resolution` - The H3 resolution of the cell. This may be adjusted if
    ///                  necessary for the substrate grid resolution.
    /// * `vertices` - output array for the vertices.
    pub fn vertices(
        &mut self,
        resolution: Resolution,
        vertices: &mut [Self],
    ) -> ExtendedResolution {
        // The vertexes of an origin-centered cell in a Class II resolution on a
        // substrate grid with aperture sequence 33r. The aperture 3 gets us the
        // vertices, and the 3r gets us back to Class II.
        //
        // Vertices listed CCW from the i-axes.
        const VERTS_CII: [CoordIJK; 6] = [
            CoordIJK::new(2, 1, 0),
            CoordIJK::new(1, 2, 0),
            CoordIJK::new(0, 2, 1),
            CoordIJK::new(0, 1, 2),
            CoordIJK::new(1, 0, 2),
            CoordIJK::new(2, 0, 1),
        ];

        // The vertexes of an origin-centered cell in a Class III resolution on
        // a substrate grid with aperture sequence 33r7r. The aperture 3 gets us
        // the vertices, and the 3r7r gets us to Class II.
        //
        // Vertices listed CCW from the i-axes.
        const VERTS_CIII: [CoordIJK; 6] = [
            CoordIJK::new(5, 4, 0),
            CoordIJK::new(1, 5, 0),
            CoordIJK::new(0, 5, 4),
            CoordIJK::new(0, 1, 5),
            CoordIJK::new(4, 0, 5),
            CoordIJK::new(5, 0, 1),
        ];

        // Adjust the center point to be in an aperture 33r substrate grid.
        self.coord = self.coord.down_aperture3::<{ CCW }>();
        self.coord = self.coord.down_aperture3::<{ CW }>();

        // If res is Class III we need to add a CW aperture 7 to get to
        // icosahedral Class II.
        let (verts, adjusted_resolution) = if resolution.is_class3() {
            self.coord = self.coord.down_aperture7::<{ CW }>();
            (&VERTS_CIII, ExtendedResolution::down(resolution))
        } else {
            (&VERTS_CII, resolution.into())
        };

        // The center point is now in the same substrate grid as the origin
        // cell vertices.
        // Add the center point substrate coordinates to each vertex to translate
        // the vertices to that cell.
        for (i, vertex) in vertices.iter_mut().enumerate() {
            vertex.face = self.face;
            vertex.coord = (self.coord + verts[i]).normalize();
        }

        adjusted_resolution
    }
}

/// Set the directions of a cell index (in-place) from finest resolution up.
///
/// IJK coordinates are adjusted during the traversal so that, at the end, they
/// should match the IJK of the base cell in the coordinate system of the
/// current base cell.
///
/// Returns the adjusted `IJK` coordinates.
#[allow(clippy::inline_always)] // 4-5% boost, up to 13% at resolution 1.
#[inline(always)]
fn directions_bits_from_ijk(
    mut ijk: CoordIJK,
    bits: &mut u64,
    resolution: Resolution,
) -> CoordIJK {
    for res in Resolution::range(Resolution::One, resolution).rev() {
        let last_ijk = ijk;
        let last_center = if res.is_class3() {
            // Rotate CCW.
            ijk = ijk.up_aperture7::<{ CCW }>();
            ijk.down_aperture7::<{ CCW }>()
        } else {
            // Rotate CW.
            ijk = ijk.up_aperture7::<{ CW }>();
            ijk.down_aperture7::<{ CW }>()
        };

        let diff = (last_ijk - last_center).normalize();
        let direction = Direction::try_from(diff).expect("unit IJK coordinate");
        // SAFETY: `res` is in [resolution; 1], thus valid.
        *bits = bits::set_direction(*bits, direction.into(), res);
    }

    ijk
}

// -----------------------------------------------------------------------------

/// Overage type.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Overage {
    /// No overage (on original face).
    None,
    /// On face edge (only occurs on substrate grids).
    FaceEdge,
    /// Overage on new face interior.
    NewFace,
}

// -----------------------------------------------------------------------------

fn get_adjacent_face_dir(i: Face, j: Face) -> u8 {
    ((ADJACENT_FACE_DIR[usize::from(i)] >> (usize::from(j) * 3)) & 0b111) as u8
}

// To reduce the footprints of the lookup table, we use a bitset where the
// direction is encoded on 3-bit:
// - `000`: central face.
// - `001`: IJ quadrant.
// - `010`: KI quadrant.
// - `011`: JK quadrant.
// - `111`: invalid face.
macro_rules! adjacent_face_dir {
    (central = $central:literal, IJ = $ij:literal, KI = $ki:literal, JK = $jk:literal) => {
        !(0 | (0b111 << (3 * $central))
            | (0b110 << (3 * $ij))
            | (0b101 << (3 * $ki))
            | (0b100 << (3 * $jk)))
    };
}

/// Direction from the origin face to the destination face, relative to the
/// origin face's coordinate system.
#[rustfmt::skip]
static ADJACENT_FACE_DIR: [u64; NUM_ICOSA_FACES] = [
    adjacent_face_dir!(central = 0,  IJ = 4,  KI = 1,  JK = 5), // Face  0.
    adjacent_face_dir!(central = 1,  IJ = 0,  KI = 2,  JK = 6), // Face  1.
    adjacent_face_dir!(central = 2,  IJ = 1,  KI = 3,  JK = 7), // Face  2.
    adjacent_face_dir!(central = 3,  IJ = 2,  KI = 4,  JK = 8), // Face  3.
    adjacent_face_dir!(central = 4,  IJ = 3,  KI = 0,  JK = 9), // Face  4.
    adjacent_face_dir!(central = 5,  IJ = 10, KI = 14, JK = 0), // Face  5.
    adjacent_face_dir!(central = 6,  IJ = 11, KI = 10, JK = 1), // Face  6.
    adjacent_face_dir!(central = 7,  IJ = 12, KI = 11, JK = 2), // Face  7.
    adjacent_face_dir!(central = 8,  IJ = 13, KI = 12, JK = 3), // Face  8.
    adjacent_face_dir!(central = 9,  IJ = 14, KI = 13, JK = 4), // Face  9.
    adjacent_face_dir!(central = 10, IJ = 5,  KI = 6,  JK = 15), // Face 10.
    adjacent_face_dir!(central = 11, IJ = 6,  KI = 7,  JK = 16), // Face 11.
    adjacent_face_dir!(central = 12, IJ = 7,  KI = 8,  JK = 17), // Face 12.
    adjacent_face_dir!(central = 13, IJ = 8,  KI = 9,  JK = 18), // Face 13.
    adjacent_face_dir!(central = 14, IJ = 9,  KI = 5,  JK = 19), // Face 14.
    adjacent_face_dir!(central = 15, IJ = 16, KI = 19, JK = 10), // Face 15.
    adjacent_face_dir!(central = 16, IJ = 17, KI = 15, JK = 11), // Face 16.
    adjacent_face_dir!(central = 17, IJ = 18, KI = 16, JK = 12), // Face 17.
    adjacent_face_dir!(central = 18, IJ = 19, KI = 17, JK = 13), // Face 18.
    adjacent_face_dir!(central = 19, IJ = 15, KI = 18, JK = 14), // Face 19.
];

// -----------------------------------------------------------------------------

/// Overage distance table.
#[rustfmt::skip]
static MAX_DIM_BY_CII_RES: &[i32] = &[
     2,        // Resolution  0.
    -1,        // Resolution  1.
     14,       // Resolution  2.
    -1,        // Resolution  3.
     98,       // Resolution  4.
    -1,        // Resolution  5.
     686,      // Resolution  6.
    -1,        // Resolution  7.
     4802,     // Resolution  8.
    -1,        // Resolution  9.
     33614,    // Resolution 10.
    -1,        // Resolution 11.
     235298,   // Resolution 12.
    -1,        // Resolution 13.
     1647086,  // Resolution 14.
    -1,        // Resolution 15.
     11529602, // Resolution 16.
];

// -----------------------------------------------------------------------------

/// Unit scale distance table.
#[rustfmt::skip]
static UNIT_SCALE_BY_CII_RES: &[i32] = &[
     1,       // Resolution  0.
    -1,       // Resolution  1.
     7,       // Resolution  2.
    -1,       // Resolution  3.
     49,      // Resolution  4.
    -1,       // Resolution  5.
     343,     // Resolution  6.
    -1,       // Resolution  7.
     2401,    // Resolution  8.
    -1,       // Resolution  9.
     16807,   // Resolution 10.
    -1,       // Resolution 11.
     117649,  // Resolution 12.
    -1,       // Resolution 13.
     823543,  // Resolution 14.
    -1,       // Resolution 15.
     5764801, // Resolution 16.
];

// -----------------------------------------------------------------------------

/// Base cell and its associated number of 60° CCW rotations.
#[derive(Clone, Copy)]
pub struct Rotation {
    /// Base cell.
    pub base_cell: BaseCell,
    /// Number of 60° CCW rotations.
    pub count: u8,
}

// Macro to save typing when declaring base cell rotations.
macro_rules! bcr {
    ($base_cell:literal, $rotation:literal) => {
        Rotation {
            base_cell: BaseCell::new_unchecked($base_cell),
            count: $rotation,
        }
    };
}

/// Resolution 0 base cell lookup table for each face.
#[rustfmt::skip]
const FACE_IJK_BASE_CELLS: [[[[Rotation; 3]; 3]; 3]; NUM_ICOSA_FACES] = [
    [
        [
            [bcr!(16, 0), bcr!(18, 0), bcr!(24, 0)],
            [bcr!(33, 0), bcr!(30, 0), bcr!(32, 3)],
            [bcr!(49, 1), bcr!(48, 3), bcr!(50, 3)],
        ], [
            [bcr!(8,  0), bcr!(5,  5), bcr!(10, 5)],
            [bcr!(22, 0), bcr!(16, 0), bcr!(18, 0)],
            [bcr!(41, 1), bcr!(33, 0), bcr!(30, 0)],
        ], [
            [bcr!(4,  0), bcr!(0,  5), bcr!(2,  5)],
            [bcr!(15, 1), bcr!(8,  0), bcr!(5,  5)],
            [bcr!(31, 1), bcr!(22, 0), bcr!(16, 0)],
        ],
    ], [
        [
            [bcr!(2,  0), bcr!(6,  0), bcr!(14, 0)],
            [bcr!(10, 0), bcr!(11, 0), bcr!(17, 3)],
            [bcr!(24, 1), bcr!(23, 3), bcr!(25, 3)],
        ], [
            [bcr!(0,  0), bcr!(1,  5), bcr!(9,  5)],
            [bcr!(5,  0), bcr!(2,  0), bcr!(6,  0)],
            [bcr!(18, 1), bcr!(10, 0), bcr!(11, 0)],
        ], [
            [bcr!(4,  1), bcr!(3, 5), bcr!(7, 5)],
            [bcr!(8,  1), bcr!(0, 0), bcr!(1, 5)],
            [bcr!(16, 1), bcr!(5, 0), bcr!(2, 0)],
        ],
    ], [
        [
            [bcr!(7,  0), bcr!(21, 0), bcr!(38, 0)],
            [bcr!(9,  0), bcr!(19, 0), bcr!(34, 3)],
            [bcr!(14, 1), bcr!(20, 3), bcr!(36, 3)],
        ], [
            [bcr!(3, 0), bcr!(13, 5), bcr!(29, 5)],
            [bcr!(1, 0), bcr!(7,  0), bcr!(21, 0)],
            [bcr!(6, 1), bcr!(9,  0), bcr!(19, 0)],
        ], [
            [bcr!(4, 2), bcr!(12, 5), bcr!(26, 5)],
            [bcr!(0, 1), bcr!(3,  0), bcr!(13, 5)],
            [bcr!(2, 1), bcr!(1,  0), bcr!(7,  0)],
        ]
    ], [
        [
            [bcr!(26, 0), bcr!(42, 0), bcr!(58, 0)],
            [bcr!(29, 0), bcr!(43, 0), bcr!(62, 3)],
            [bcr!(38, 1), bcr!(47, 3), bcr!(64, 3)],
        ], [
            [bcr!(12, 0), bcr!(28, 5), bcr!(44, 5)],
            [bcr!(13, 0), bcr!(26, 0), bcr!(42, 0)],
            [bcr!(21, 1), bcr!(29, 0), bcr!(43, 0)],
        ], [
            [bcr!(4, 3), bcr!(15, 5), bcr!(31, 5)],
            [bcr!(3, 1), bcr!(12, 0), bcr!(28, 5)],
            [bcr!(7, 1), bcr!(13, 0), bcr!(26, 0)],
        ]
    ], [
        [
            [bcr!(31, 0), bcr!(41, 0), bcr!(49, 0)],
            [bcr!(44, 0), bcr!(53, 0), bcr!(61, 3)],
            [bcr!(58, 1), bcr!(65, 3), bcr!(75, 3)],
        ], [
            [bcr!(15, 0), bcr!(22, 5), bcr!(33, 5)],
            [bcr!(28, 0), bcr!(31, 0), bcr!(41, 0)],
            [bcr!(42, 1), bcr!(44, 0), bcr!(53, 0)],
        ], [
            [bcr!(4,  4), bcr!(8,  5), bcr!(16, 5)],
            [bcr!(12, 1), bcr!(15, 0), bcr!(22, 5)],
            [bcr!(26, 1), bcr!(28, 0), bcr!(31, 0)],
        ]
    ], [
        [
            [bcr!(50, 0), bcr!(48, 0), bcr!(49, 3)],
            [bcr!(32, 0), bcr!(30, 3), bcr!(33, 3)],
            [bcr!(24, 3), bcr!(18, 3), bcr!(16, 3)],
        ], [
            [bcr!(70, 0), bcr!(67, 0), bcr!(66, 3)],
            [bcr!(52, 3), bcr!(50, 0), bcr!(48, 0)],
            [bcr!(37, 3), bcr!(32, 0), bcr!(30, 3)],
        ], [
            [bcr!(83, 0), bcr!(87, 3), bcr!(85, 3)],
            [bcr!(74, 3), bcr!(70, 0), bcr!(67, 0)],
            [bcr!(57, 1), bcr!(52, 3), bcr!(50, 0)],
        ]
    ], [
        [
            [bcr!(25, 0), bcr!(23, 0), bcr!(24, 3)],
            [bcr!(17, 0), bcr!(11, 3), bcr!(10, 3)],
            [bcr!(14, 3), bcr!(6,  3), bcr!(2,  3)],
        ], [
            [bcr!(45, 0), bcr!(39, 0), bcr!(37, 3)],
            [bcr!(35, 3), bcr!(25, 0), bcr!(23, 0)],
            [bcr!(27, 3), bcr!(17, 0), bcr!(11, 3)],
        ], [
            [bcr!(63, 0), bcr!(59, 3), bcr!(57, 3)],
            [bcr!(56, 3), bcr!(45, 0), bcr!(39, 0)],
            [bcr!(46, 3), bcr!(35, 3), bcr!(25, 0)],
        ]
    ], [
        [
            [bcr!(36, 0), bcr!(20, 0), bcr!(14, 3)],
            [bcr!(34, 0), bcr!(19, 3), bcr!(9,  3)],
            [bcr!(38, 3), bcr!(21, 3), bcr!(7,  3)],
        ], [
            [bcr!(55, 0), bcr!(40, 0), bcr!(27, 3)],
            [bcr!(54, 3), bcr!(36, 0), bcr!(20, 0)],
            [bcr!(51, 3), bcr!(34, 0), bcr!(19, 3)],
        ], [
            [bcr!(72, 0), bcr!(60, 3), bcr!(46, 3)],
            [bcr!(73, 3), bcr!(55, 0), bcr!(40, 0)],
            [bcr!(71, 3), bcr!(54, 3), bcr!(36, 0)],
        ]
    ], [
        [
            [bcr!(64, 0), bcr!(47, 0), bcr!(38, 3)],
            [bcr!(62, 0), bcr!(43, 3), bcr!(29, 3)],
            [bcr!(58, 3), bcr!(42, 3), bcr!(26, 3)],
        ], [
            [bcr!(84, 0), bcr!(69, 0), bcr!(51, 3)],
            [bcr!(82, 3), bcr!(64, 0), bcr!(47, 0)],
            [bcr!(76, 3), bcr!(62, 0), bcr!(43, 3)],
        ], [
            [bcr!(97, 0), bcr!(89, 3), bcr!(71, 3)],
            [bcr!(98, 3), bcr!(84, 0), bcr!(69, 0)],
            [bcr!(96, 3), bcr!(82, 3), bcr!(64, 0)],
        ]
    ], [
        [
            [bcr!(75, 0), bcr!(65, 0), bcr!(58, 3)],
            [bcr!(61, 0), bcr!(53, 3), bcr!(44, 3)],
            [bcr!(49, 3), bcr!(41, 3), bcr!(31, 3)],
        ], [
            [bcr!(94, 0), bcr!(86, 0), bcr!(76, 3)],
            [bcr!(81, 3), bcr!(75, 0), bcr!(65, 0)],
            [bcr!(66, 3), bcr!(61, 0), bcr!(53, 3)],
        ], [
            [bcr!(107, 0), bcr!(104, 3), bcr!(96, 3)],
            [bcr!(101, 3), bcr!(94,  0), bcr!(86, 0)],
            [bcr!(85,  3), bcr!(81,  3), bcr!(75, 0)],
        ]
    ], [
        [
            [bcr!(57, 0), bcr!(59, 0), bcr!(63, 3)],
            [bcr!(74, 0), bcr!(78, 3), bcr!(79, 3)],
            [bcr!(83, 3), bcr!(92, 3), bcr!(95, 3)],
        ], [
            [bcr!(37, 0), bcr!(39, 3), bcr!(45, 3)],
            [bcr!(52, 0), bcr!(57, 0), bcr!(59, 0)],
            [bcr!(70, 3), bcr!(74, 0), bcr!(78, 3)],
        ], [
            [bcr!(24, 0), bcr!(23, 3), bcr!(25, 3)],
            [bcr!(32, 3), bcr!(37, 0), bcr!(39, 3)],
            [bcr!(50, 3), bcr!(52, 0), bcr!(57, 0)],
        ]
    ], [
        [
            [bcr!(46, 0), bcr!(60, 0), bcr!(72, 3)],
            [bcr!(56, 0), bcr!(68, 3), bcr!(80, 3)],
            [bcr!(63, 3), bcr!(77, 3), bcr!(90, 3)],
        ], [
            [bcr!(27, 0), bcr!(40, 3), bcr!(55, 3)],
            [bcr!(35, 0), bcr!(46, 0), bcr!(60, 0)],
            [bcr!(45, 3), bcr!(56, 0), bcr!(68, 3)],
        ], [
            [bcr!(14, 0), bcr!(20, 3), bcr!(36, 3)],
            [bcr!(17, 3), bcr!(27, 0), bcr!(40, 3)],
            [bcr!(25, 3), bcr!(35, 0), bcr!(46, 0)],
        ]
    ], [
        [
            [bcr!(71, 0), bcr!(89, 0), bcr!(97,  3)],
            [bcr!(73, 0), bcr!(91, 3), bcr!(103, 3)],
            [bcr!(72, 3), bcr!(88, 3), bcr!(105, 3)],
        ], [
            [bcr!(51, 0), bcr!(69, 3), bcr!(84, 3)],
            [bcr!(54, 0), bcr!(71, 0), bcr!(89, 0)],
            [bcr!(55, 3), bcr!(73, 0), bcr!(91, 3)],
        ], [
            [bcr!(38, 0), bcr!(47, 3), bcr!(64, 3)],
            [bcr!(34, 3), bcr!(51, 0), bcr!(69, 3)],
            [bcr!(36, 3), bcr!(54, 0), bcr!(71, 0)],
        ]
    ], [
        [
            [bcr!(96, 0), bcr!(104, 0), bcr!(107, 3)],
            [bcr!(98, 0), bcr!(110, 3), bcr!(115, 3)],
            [bcr!(97, 3), bcr!(111, 3), bcr!(119, 3)],
        ], [
            [bcr!(76, 0), bcr!(86, 3), bcr!(94,  3)],
            [bcr!(82, 0), bcr!(96, 0), bcr!(104, 0)],
            [bcr!(84, 3), bcr!(98, 0), bcr!(110, 3)],
        ], [
            [bcr!(58, 0), bcr!(65, 3), bcr!(75, 3)],
            [bcr!(62, 3), bcr!(76, 0), bcr!(86, 3)],
            [bcr!(64, 3), bcr!(82, 0), bcr!(96, 0)],
        ]
    ], [
        [
            [bcr!(85,  0), bcr!(87,  0), bcr!(83,  3)],
            [bcr!(101, 0), bcr!(102, 3), bcr!(100, 3)],
            [bcr!(107, 3), bcr!(112, 3), bcr!(114, 3)],
        ], [
            [bcr!(66, 0), bcr!(67,  3), bcr!(70,  3)],
            [bcr!(81, 0), bcr!(85,  0), bcr!(87,  0)],
            [bcr!(94, 3), bcr!(101, 0), bcr!(102, 3)],
        ], [
            [bcr!(49, 0), bcr!(48, 3), bcr!(50, 3)],
            [bcr!(61, 3), bcr!(66, 0), bcr!(67, 3)],
            [bcr!(75, 3), bcr!(81, 0), bcr!(85, 0)],
        ]
    ], [
        [
            [bcr!(95, 0), bcr!(92, 0), bcr!(83, 0)],
            [bcr!(79, 0), bcr!(78, 0), bcr!(74, 3)],
            [bcr!(63, 1), bcr!(59, 3), bcr!(57, 3)],
        ], [
            [bcr!(109, 0), bcr!(108, 0), bcr!(100, 5)],
            [bcr!(93,  1), bcr!(95,  0), bcr!(92,  0)],
            [bcr!(77,  1), bcr!(79,  0), bcr!(78,  0)],
        ], [
            [bcr!(117, 4), bcr!(118, 5), bcr!(114, 5)],
            [bcr!(106, 1), bcr!(109, 0), bcr!(108, 0)],
            [bcr!(90,  1), bcr!(93,  1), bcr!(95,  0)],
        ]
    ], [
        [
            [bcr!(90, 0), bcr!(77, 0), bcr!(63, 0)],
            [bcr!(80, 0), bcr!(68, 0), bcr!(56, 3)],
            [bcr!(72, 1), bcr!(60, 3), bcr!(46, 3)],
        ], [
            [bcr!(106, 0), bcr!(93, 0), bcr!(79, 5)],
            [bcr!(99,  1), bcr!(90, 0), bcr!(77, 0)],
            [bcr!(88,  1), bcr!(80, 0), bcr!(68, 0)],
        ], [
            [bcr!(117, 3), bcr!(109, 5), bcr!(95, 5)],
            [bcr!(113, 1), bcr!(106, 0), bcr!(93, 0)],
            [bcr!(105, 1), bcr!(99,  1), bcr!(90, 0)],
        ]
    ], [
        [
            [bcr!(105, 0), bcr!(88, 0), bcr!(72, 0)],
            [bcr!(103, 0), bcr!(91, 0), bcr!(73, 3)],
            [bcr!(97,  1), bcr!(89, 3), bcr!(71, 3)],
        ], [
            [bcr!(113, 0), bcr!(99,  0), bcr!(80, 5)],
            [bcr!(116, 1), bcr!(105, 0), bcr!(88, 0)],
            [bcr!(111, 1), bcr!(103, 0), bcr!(91, 0)],
        ], [
            [bcr!(117, 2), bcr!(106, 5), bcr!(90, 5)],
            [bcr!(121, 1), bcr!(113, 0), bcr!(99, 0)],
            [bcr!(119, 1), bcr!(116, 1), bcr!(105, 0)],
        ]
    ], [
        [
            [bcr!(119, 0), bcr!(111, 0), bcr!(97, 0)],
            [bcr!(115, 0), bcr!(110, 0), bcr!(98, 3)],
            [bcr!(107, 1), bcr!(104, 3), bcr!(96, 3)],
        ], [
            [bcr!(121, 0), bcr!(116, 0), bcr!(103, 5)],
            [bcr!(120, 1), bcr!(119, 0), bcr!(111, 0)],
            [bcr!(112, 1), bcr!(115, 0), bcr!(110, 0)],
        ], [
            [bcr!(117, 1), bcr!(113, 5), bcr!(105, 5)],
            [bcr!(118, 1), bcr!(121, 0), bcr!(116, 0)],
            [bcr!(114, 1), bcr!(120, 1), bcr!(119, 0)],
        ]
    ], [
        [
            [bcr!(114, 0), bcr!(112, 0), bcr!(107, 0)],
            [bcr!(100, 0), bcr!(102, 0), bcr!(101, 3)],
            [bcr!(83,  1), bcr!(87,  3), bcr!(85,  3)],
        ], [
            [bcr!(118, 0), bcr!(120, 0), bcr!(115, 5)],
            [bcr!(108, 1), bcr!(114, 0), bcr!(112, 0)],
            [bcr!(92,  1), bcr!(100, 0), bcr!(102, 0)],
        ], [
            [bcr!(117, 0), bcr!(121, 5), bcr!(119, 5)],
            [bcr!(109, 1), bcr!(118, 0), bcr!(120, 0)],
            [bcr!(95,  1), bcr!(108, 1), bcr!(114, 0)],
        ]
    ]
];
