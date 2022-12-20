//! Various precomputed data about each of the 20 icosahedron face.

use crate::{
    coord::{CoordIJK, LatLng, Vec3d},
    error, NUM_ICOSA_FACES,
};
use std::fmt;

// -----------------------------------------------------------------------------

/// An icosahedron face.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Face(u8);

impl Face {
    /// Initializes a new `Face` using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid face.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn new_unchecked(value: usize) -> Self {
        debug_assert!(value < NUM_ICOSA_FACES, "face out of range");
        Self(value as u8)
    }
}

impl From<Face> for usize {
    fn from(value: Face) -> Self {
        Self::from(value.0)
    }
}

impl From<Face> for u8 {
    fn from(value: Face) -> Self {
        value.0
    }
}

impl TryFrom<u8> for Face {
    type Error = error::InvalidFace;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if usize::from(value) >= NUM_ICOSA_FACES {
            return Err(Self::Error::new(value, "out of range"));
        }

        Ok(Self(value))
    }
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------

/// A set of icosahedron faces.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct FaceSet(u32);

impl FaceSet {
    /// Initializes a new empty set of faces.
    pub(crate) const fn new() -> Self {
        Self(0)
    }

    /// Adds a new face into the set.
    pub(crate) fn insert(&mut self, face: Face) {
        let offset = u8::from(face);
        self.0 |= 1 << u32::from(offset);
    }

    /// Returns the number of faces in the set.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x089283470803ffff)?;
    /// let faces = index.icosahedron_faces();
    /// assert_eq!(faces.len(), 1);
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns whether the set is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x089283470803ffff)?;
    /// let faces = index.icosahedron_faces();
    /// assert!(!faces.is_empty());
    /// # Ok::<(), h3o::error::InvalidCellIndex>(())
    /// ```
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the specified face is present in the set.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x089283470803ffff)?;
    /// let faces = index.icosahedron_faces();
    /// assert!(faces.contains(h3o::Face::try_from(7)?));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn contains(self, face: Face) -> bool {
        let offset = u8::from(face);
        self.0 & 1 << u32::from(offset) != 0
    }

    /// Returns the contained faces.
    ///
    /// # Example
    ///
    /// ```
    /// let index = h3o::CellIndex::try_from(0x089283470803ffff)?;
    /// let faces = index.icosahedron_faces().iter().collect::<Vec<_>>();
    /// assert_eq!(faces, vec![h3o::Face::try_from(7)?]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn iter(self) -> impl Iterator<Item = Face> {
        (0..NUM_ICOSA_FACES).filter_map(move |offset| {
            #[allow(clippy::cast_possible_truncation)]
            // bounded by NUM_ICOSA_FACES.
            (self.0 >> offset & 1 == 1).then_some(Face(offset as u8))
        })
    }
}

impl fmt::Display for FaceSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.iter()
                .map(|face| face.to_string())
                .collect::<Vec<_>>()
                .join("-")
        )
    }
}

// -----------------------------------------------------------------------------

/// Icosahedron face centers in x/y/z on the unit sphere.
#[rustfmt::skip]
pub static CENTER_POINT: [Vec3d; NUM_ICOSA_FACES] = [
    Vec3d {x:  0.2199307791404606, y:  0.6583691780274996, z:  0.7198475378926182},
    Vec3d {x: -0.2139234834501421, y:  0.1478171829550703, z:  0.9656017935214205},
    Vec3d {x:  0.1092625278784797, y: -0.481195157287321,  z:  0.8697775121287253},
    Vec3d {x:  0.7428567301586791, y: -0.3593941678278028, z:  0.5648005936517033},
    Vec3d {x:  0.8112534709140969, y:  0.3448953237639384, z:  0.472138773641393},
    Vec3d {x: -0.1055498149613921, y:  0.9794457296411413, z:  0.1718874610009365},
    Vec3d {x: -0.8075407579970092, y:  0.1533552485898818, z:  0.5695261994882688},
    Vec3d {x: -0.2846148069787907, y: -0.8644080972654206, z:  0.4144792552473539},
    Vec3d {x:  0.7405621473854482, y: -0.6673299564565524, z: -0.0789837646326737},
    Vec3d {x:  0.8512303986474293, y:  0.4722343788582681, z: -0.2289137388687808},
    Vec3d {x: -0.7405621473854481, y:  0.6673299564565524, z:  0.0789837646326737},
    Vec3d {x: -0.8512303986474292, y: -0.4722343788582682, z:  0.2289137388687808},
    Vec3d {x:  0.1055498149613919, y: -0.9794457296411413, z: -0.1718874610009365},
    Vec3d {x:  0.8075407579970092, y: -0.1533552485898819, z: -0.5695261994882688},
    Vec3d {x:  0.2846148069787908, y:  0.8644080972654204, z: -0.4144792552473539},
    Vec3d {x: -0.7428567301586791, y:  0.3593941678278027, z: -0.5648005936517033},
    Vec3d {x: -0.811253470914097,  y: -0.3448953237639382, z: -0.472138773641393},
    Vec3d {x: -0.2199307791404607, y: -0.6583691780274996, z: -0.7198475378926182},
    Vec3d {x:  0.213923483450142,  y: -0.1478171829550704, z: -0.9656017935214205},
    Vec3d {x: -0.1092625278784796, y:  0.481195157287321,  z: -0.8697775121287253},
];

/// Icosahedron face centers in lat/lng radians.
#[rustfmt::skip]
pub static CENTER_GEO: [LatLng; NUM_ICOSA_FACES] = [
    LatLng::new_unchecked( 0.80358264971899,     1.2483974196173961),
    LatLng::new_unchecked( 1.3077478834556382,   2.5369450098779214),
    LatLng::new_unchecked( 1.054751253523952,   -1.3475173589003966),
    LatLng::new_unchecked( 0.6001915955381868,  -0.45060390946975576),
    LatLng::new_unchecked( 0.49171542819877384,  0.40198820291130694),
    LatLng::new_unchecked( 0.1727453274156187,   1.6781468852804338),
    LatLng::new_unchecked( 0.6059293215713507,   2.9539233298124117),
    LatLng::new_unchecked( 0.42737051832897965, -1.8888762003362853),
    LatLng::new_unchecked(-0.07906611854921283, -0.7334295133808677),
    LatLng::new_unchecked(-0.23096164445538364,  0.506495587332349),
    LatLng::new_unchecked( 0.07906611854921283,  2.4081631402089254),
    LatLng::new_unchecked( 0.23096164445538364, -2.635097066257444),
    LatLng::new_unchecked(-0.1727453274156187,  -1.4634457683093596),
    LatLng::new_unchecked(-0.6059293215713507,  -0.18766932377738163),
    LatLng::new_unchecked(-0.42737051832897965,  1.2527164532535078),
    LatLng::new_unchecked(-0.6001915955381868,   2.6909887441200375),
    LatLng::new_unchecked(-0.49171542819877384, -2.7396044506784865),
    LatLng::new_unchecked(-0.80358264971899,    -1.8931952339723972),
    LatLng::new_unchecked(-1.3077478834556382,  -0.6046476437118721),
    LatLng::new_unchecked(-1.054751253523952,    1.7940752946893965),
];

/// Icosahedron face `ijk` axes as azimuth in radians from face center to vertex
/// `0`/`1`/`2` respectively.
#[rustfmt::skip]
pub static AXES_AZ_RADS_CII : [[f64; 3]; NUM_ICOSA_FACES] = [
    [5.6199582685239395,  3.5255631661307447, 1.4311680637375488],
    [5.7603390817141875,  3.665943979320992,  1.571548876927796],
    [0.78021365439343,    4.969003859179821,  2.8746087567866256],
    [0.4304693639799999,  4.619259568766391,  2.5248644663731956],
    [6.130269123335111,   4.0358740209419155, 1.9414789185487202],
    [2.692877706530643,   0.5984826041374471, 4.787272808923838],
    [2.982963003477244,   0.8885679010840484, 5.07735810587044],
    [3.532912002790141,   1.4385169003969456, 5.627307105183337],
    [3.494305004259568,   1.3999099018663728, 5.588700106652764],
    [3.0032141694995382,  0.908819067106343,  5.0976092718927335],
    [5.930472956509812,   3.836077854116616,  1.7416827517234204],
    [0.13837848409025486, 4.327168688876646,  2.23277358648345],
    [0.4487149470591504,  4.6375051518455415, 2.543110049452346],
    [0.15862965011254937, 4.3474198548989405, 2.2530247525057447],
    [5.891865957979238,   3.797470855586043,  1.7030757531928475],
    [2.711123289609793,   0.6167281872165977, 4.8055183920029885],
    [3.294508837434268,   1.2001137350410729, 5.388903939827464],
    [3.80481969224544,    1.7104245898522445, 5.8992147946386355],
    [3.6644388790551923,  1.570043776661997,  5.758833981448388],
    [2.361378999196363,   0.2669838968031676, 4.455774101589559],
];

/// Information to transform into an adjacent face IJK system.
#[derive(Debug, Clone, Copy)]
pub struct FaceOrientIJK {
    /// Face number.
    pub face: Face,
    /// Resolution 0 translation relative to primary face.
    pub translate: CoordIJK,
    /// Number of 60 degree ccw rotations relative to primary face.
    pub ccw_rot60: u8,
}

// indexes for NEIGHBORS table.
/// IJ quadrant NEIGHBORS table direction.
pub const IJ: usize = 1;
/// KI quadrant NEIGHBORS table direction.
pub const KI: usize = 2;
/// JK quadrant NEIGHBORS table direction.
pub const JK: usize = 3;

macro_rules! face_orient_ijk {
    [$face:literal, ($i:literal, $j: literal, $k: literal), $ccw_rot60:literal] => {
        FaceOrientIJK { face: Face($face), translate: CoordIJK::new($i, $j, $k), ccw_rot60: $ccw_rot60 }
    }
}

/// Definition of which faces neighbor each other.
#[rustfmt::skip]
pub static NEIGHBORS: [[FaceOrientIJK; 4]; NUM_ICOSA_FACES] = [
    [
        // Face 0.
        face_orient_ijk!(0, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(4, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(1, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(5, (0, 2, 2), 3), // jk quadrant.
    ], [
        // Face 1.
        face_orient_ijk!(1, (0, 0, 0), 0), // Central face
        face_orient_ijk!(0, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(2, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(6, (0, 2, 2), 3), // jk quadrant.
    ], [
        // Face 2.
        face_orient_ijk!(2, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(1, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(3, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(7, (0, 2, 2), 3), // jk quadrant.
    ], [
        // Face 3.
        face_orient_ijk!(3, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(2, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(4, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(8, (0, 2, 2), 3), // jk quadrant.
    ], [
        // Face 4.
        face_orient_ijk!(4, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(3, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(0, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(9, (0, 2, 2), 3), // jk quadrant.
    ], [
        // Face 5.
        face_orient_ijk!(5,  (0, 0, 0), 0),  // Central face.
        face_orient_ijk!(10, (2, 2, 0), 3),  // ij quadrant.
        face_orient_ijk!(14, (2, 0, 2), 3),  // ki quadrant.
        face_orient_ijk!(0,  (0, 2, 2), 3),  // jk quadrant.
    ], [
        // Face 6.
        face_orient_ijk!(6,  (0, 0, 0), 0), // Central face.
        face_orient_ijk!(11, (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(10, (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(1,  (0, 2, 2), 3)  // jk quadrant.
    ], [
        // Face 7.
        face_orient_ijk!(7,  (0, 0, 0), 0), // Central face.
        face_orient_ijk!(12, (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(11, (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(2,  (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 8.
        face_orient_ijk!(8,  (0, 0, 0), 0), // Central face.
        face_orient_ijk!(13, (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(12, (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(3,  (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 9.
        face_orient_ijk!(9,  (0, 0, 0), 0), // Central face.
        face_orient_ijk!(14, (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(13, (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(4,  (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 10.
        face_orient_ijk!(10, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(5,  (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(6,  (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(15, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 11.
        face_orient_ijk!(11, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(6,  (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(7,  (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(16, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 12.
        face_orient_ijk!(12, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(7,  (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(8,  (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(17, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 13.
        face_orient_ijk!(13, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(8,  (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(9,  (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(18, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 14.
        face_orient_ijk!(14, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(9,  (2, 2, 0), 3), // ij quadrant.
        face_orient_ijk!(5,  (2, 0, 2), 3), // ki quadrant.
        face_orient_ijk!(19, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 15.
        face_orient_ijk!(15, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(16, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(19, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(10, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 16.
        face_orient_ijk!(16, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(17, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(15, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(11, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 17.
        face_orient_ijk!(17, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(18, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(16, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(12, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 18.
        face_orient_ijk!(18, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(19, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(17, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(13, (0, 2, 2), 3), // jk quadrant.
     ], [
        // Face 19.
        face_orient_ijk!(19, (0, 0, 0), 0), // Central face.
        face_orient_ijk!(15, (2, 0, 2), 1), // ij quadrant.
        face_orient_ijk!(18, (2, 2, 0), 5), // ki quadrant.
        face_orient_ijk!(14, (0, 2, 2), 3), // jk quadrant.
    ]
];
