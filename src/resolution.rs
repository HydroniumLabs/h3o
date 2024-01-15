use crate::{error, index::bits, BaseCell, CellIndex, NUM_PENTAGONS};
use std::{ffi::c_int, fmt, iter::DoubleEndedIterator, str::FromStr};

/// Cell resolution, from 0 to 15.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
#[allow(clippy::exhaustive_enums)] // Not gonna change any time soon.
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
pub enum Resolution {
    /// Resolution 0.
    Zero = 0,
    /// Resolution 1.
    One = 1,
    /// Resolution 2.
    Two = 2,
    /// Resolution 3.
    Three = 3,
    /// Resolution 4.
    Four = 4,
    /// Resolution 5.
    Five = 5,
    /// Resolution 6.
    Six = 6,
    /// Resolution 7.
    Seven = 7,
    /// Resolution 8.
    Eight = 8,
    /// Resolution 9.
    Nine = 9,
    /// Resolution 10.
    Ten = 10,
    /// Resolution 11.
    Eleven = 11,
    /// Resolution 12.
    Twelve = 12,
    /// Resolution 13.
    Thirteen = 13,
    /// Resolution 14.
    Fourteen = 14,
    /// Resolution 15.
    Fifteen = 15,
}

impl Resolution {
    /// Returns true if the resolution is a Class III resolution.
    ///
    /// Cells in a class III resolution are rotated versus the icosahedron and
    /// subject to shape distortion adding extra points on icosahedron edges,
    /// making them not true hexagons.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::Resolution;
    ///
    /// assert!(Resolution::Eleven.is_class3());
    /// assert!(!Resolution::Two.is_class3());
    /// ```
    #[must_use]
    pub const fn is_class3(self) -> bool {
        (self as u8) % 2 == 1
    }

    /// Return the next resolution, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::Resolution;
    ///
    /// assert_eq!(Resolution::Eleven.succ(), Some(Resolution::Twelve));
    /// assert!(Resolution::Fifteen.succ().is_none());
    /// ```
    #[must_use]
    pub fn succ(self) -> Option<Self> {
        // SAFETY: Every resolution but 15 have a finer one.
        (self != Self::Fifteen).then(|| Self::new_unchecked(u8::from(self) + 1))
    }

    /// Return the previous resolution, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::Resolution;
    ///
    /// assert_eq!(Resolution::Eleven.pred(), Some(Resolution::Ten));
    /// assert!(Resolution::Zero.pred().is_none());
    /// ```
    #[must_use]
    pub fn pred(self) -> Option<Self> {
        // SAFETY: Every resolution but 0 have a coarser one.
        (self != Self::Zero).then(|| Self::new_unchecked(u8::from(self) - 1))
    }

    /// Iterates over the resolution in `[start, end]` (inclusive bounds).
    ///
    /// # Arguments
    ///
    /// * `start` - The lower bound of the range (inclusive).
    /// * `end` - The upper bound of the range (inclusive).
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::Resolution;
    ///
    /// // From 5 to 7.
    /// let res = Resolution::range(
    ///     Resolution::Five, Resolution::Seven
    /// ).collect::<Vec<_>>();
    /// assert_eq!(res, vec![Resolution::Five, Resolution::Six, Resolution::Seven]);
    ///
    /// // From 2 to 0: wrong way to do it.
    /// let res = Resolution::range(Resolution::Two, Resolution::Zero)
    ///               .collect::<Vec<_>>();
    /// assert!(res.is_empty());
    ///
    /// // Use `rev` instead!
    /// let res = Resolution::range(Resolution::Zero, Resolution::Two)
    ///               .rev()
    ///               .collect::<Vec<_>>();
    /// assert_eq!(res, vec![Resolution::Two, Resolution::One, Resolution::Zero]);
    /// ```
    #[allow(unsafe_code)]
    pub fn range(
        start: Self,
        end: Self,
    ) -> impl Iterator<Item = Self> + DoubleEndedIterator {
        (u8::from(start)..=u8::from(end))
            // SAFETY: values between two resolutions are valid resolutions.
            .map(|value| unsafe { std::mem::transmute::<u8, Self>(value) })
    }

    /// Returns the average hexagon area, in square radians, at this
    /// resolution (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_area = h3o::Resolution::Three.area_rads2();
    /// ```
    #[must_use]
    pub const fn area_rads2(self) -> f64 {
        match self {
            Self::Zero => 0.10735348936216678,
            Self::One => 0.015023219032163554,
            Self::Two => 0.002138515704691145,
            Self::Three => 0.00030533422843566127,
            Self::Four => 4.361565217314979e-5,
            Self::Five => 6.230734784621071e-6,
            Self::Six => 8.90103480357314e-7,
            Self::Seven => 1.2715760961973138e-7,
            Self::Eight => 1.8165372181422116e-8,
            Self::Nine => 2.5950531560902726e-9,
            Self::Ten => 3.707218791825834e-10,
            Self::Eleven => 5.2960268449371254e-11,
            Self::Twelve => 7.565752635516638e-12,
            Self::Thirteen => 1.0808218050716045e-12,
            Self::Fourteen => 1.5440311501018426e-13,
            Self::Fifteen => 2.205758785859684e-14,
        }
    }

    /// Returns the average hexagon area, in square kilometers, at this
    /// resolution (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_area = h3o::Resolution::Three.area_km2();
    /// ```
    #[must_use]
    pub const fn area_km2(self) -> f64 {
        match self {
            Self::Zero => 4.357449416078383e6,
            Self::One => 6.097884417941332e5,
            Self::Two => 8.68017803989972e4,
            Self::Three => 1.239343465508816e4,
            Self::Four => 1.770347654491307e3,
            Self::Five => 2.529038581819449e2,
            Self::Six => 3.612906216441245e1,
            Self::Seven => 5.161293359717191,
            Self::Eight => 7.373275975944177e-1,
            Self::Nine => 1.053325134272067e-1,
            Self::Ten => 1.504750190766435e-2,
            Self::Eleven => 2.149643129451879e-3,
            Self::Twelve => 3.07091875631606e-4,
            Self::Thirteen => 4.387026794728296e-5,
            Self::Fourteen => 6.267181135324313e-6,
            Self::Fifteen => 8.95311590760579e-7,
        }
    }

    /// Returns the average hexagon area, in square meters, at this resolution
    /// (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_area = h3o::Resolution::Three.area_m2();
    /// ```
    #[must_use]
    pub const fn area_m2(self) -> f64 {
        match self {
            Self::Zero => 4.35744941607839e12,
            Self::One => 6.097884417941339e11,
            Self::Two => 8.680178039899731e10,
            Self::Three => 1.239343465508818e10,
            Self::Four => 1.770347654491309e9,
            Self::Five => 2.529038581819452e8,
            Self::Six => 3.61290621644125e7,
            Self::Seven => 5.161293359717198e6,
            Self::Eight => 7.373275975944188e5,
            Self::Nine => 1.053325134272069e5,
            Self::Ten => 1.504750190766437e4,
            Self::Eleven => 2.149643129451882e3,
            Self::Twelve => 3.070918756316063e2,
            Self::Thirteen => 4.387026794728301e1,
            Self::Fourteen => 6.267181135324322,
            Self::Fifteen => 8.953115907605802e-1,
        }
    }

    /// Returns the average hexagon edge length, in radians, at this
    /// resolution (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_edge_len = h3o::Resolution::Three.edge_length_rads();
    /// ```
    #[must_use]
    pub const fn edge_length_rads(self) -> f64 {
        match self {
            Self::Zero => 0.20110729345570325,
            Self::One => 0.07582111043884765,
            Self::Two => 0.02864742595735945,
            Self::Three => 0.01082705133253538,
            Self::Four => 0.00409225087020209,
            Self::Five => 0.0015467085046591636,
            Self::Six => 0.0005846065718298453,
            Self::Seven => 0.0002207619176534713,
            Self::Eight => 0.0000834113029497395,
            Self::Nine => 0.00003151560528974152,
            Self::Ten => 0.000011907659293816194,
            Self::Eleven => 0.000004499115550979479,
            Self::Twelve => 0.000001699917695123959,
            Self::Thirteen => 0.0000006422862754806202,
            Self::Fourteen => 0.0000002426774312980401,
            Self::Fifteen => 0.00000009169172362798202,
        }
    }

    /// Returns the average hexagon edge length, in kilometers, at this
    /// resolution (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_edge_len = h3o::Resolution::Three.edge_length_km();
    /// ```
    #[must_use]
    pub const fn edge_length_km(self) -> f64 {
        match self {
            Self::Zero => 1281.2560107413644,
            Self::One => 483.0568390711111,
            Self::Two => 182.51295648916735,
            Self::Three => 68.97922178775585,
            Self::Four => 26.07175968017739,
            Self::Five => 9.854090989971207,
            Self::Six => 3.7245326671400765,
            Self::Seven => 1.4064757626435986,
            Self::Eight => 0.5314140100625567,
            Self::Nine => 0.20078614761193547,
            Self::Ten => 0.07586378286883358,
            Self::Eleven => 0.02866389748307224,
            Self::Twelve => 0.010830187842605124,
            Self::Thirteen => 0.004092010473292413,
            Self::Fourteen => 0.001546099657446663,
            Self::Fifteen => 0.0005841686296646656,
        }
    }

    /// Returns the average hexagon edge length, in meters, at this resolution
    /// (excludes pentagons).
    ///
    /// # Example
    ///
    /// ```
    /// let avg_edge_len = h3o::Resolution::Three.edge_length_m();
    /// ```
    #[must_use]
    // I don't want to group digits of the decimal part.
    #[allow(clippy::inconsistent_digit_grouping)]
    pub const fn edge_length_m(self) -> f64 {
        match self {
            Self::Zero => 1281256.0107413644,
            Self::One => 483056.8390711111,
            Self::Two => 182512.95648916735,
            Self::Three => 68979.22178775584,
            Self::Four => 26071.75968017739,
            Self::Five => 9854.090989971206,
            Self::Six => 3724.5326671400767,
            Self::Seven => 1406.4757626435987,
            Self::Eight => 531.4140100625567,
            Self::Nine => 200.78614761193546,
            Self::Ten => 75.86378286883358,
            Self::Eleven => 28.66389748307224,
            Self::Twelve => 10.830187842605124,
            Self::Thirteen => 4.092010473292413,
            Self::Fourteen => 1.546099657446663,
            Self::Fifteen => 0.5841686296646657,
        }
    }

    /// Returns the number of unique H3 indexes at the given resolution.
    ///
    /// # Example
    ///
    /// ```
    /// let nb_cells = h3o::Resolution::Three.cell_count();
    /// ```
    #[must_use]
    pub const fn cell_count(self) -> u64 {
        // 2 + 120 * pow(7, resolution)
        match self {
            Self::Zero => 122,
            Self::One => 842,
            Self::Two => 5882,
            Self::Three => 41_162,
            Self::Four => 288_122,
            Self::Five => 2_016_842,
            Self::Six => 14_117_882,
            Self::Seven => 98_825_162,
            Self::Eight => 691_776_122,
            Self::Nine => 4_842_432_842,
            Self::Ten => 33_897_029_882,
            Self::Eleven => 237_279_209_162,
            Self::Twelve => 1_660_954_464_122,
            Self::Thirteen => 11_626_681_248_842,
            Self::Fourteen => 81_386_768_741_882,
            Self::Fifteen => 569_707_381_193_162,
        }
    }

    /// Returns the number of pentagons (same at any resolution).
    ///
    /// # Example
    ///
    /// ```
    /// assert_eq!(h3o::Resolution::pentagon_count(), 12);
    /// ```
    #[must_use]
    pub const fn pentagon_count() -> u8 {
        NUM_PENTAGONS
    }

    /// Generates all pentagons at this resolution.
    ///
    /// # Example
    ///
    /// ```
    /// let pentagons = h3o::Resolution::Two.pentagons().collect::<Vec<_>>();
    /// ```
    pub fn pentagons(self) -> impl Iterator<Item = CellIndex> {
        // Template for a resolution 0 index:
        // mode = CELL, resolution = 0, all children at 0.
        const TEMPLATE: u64 = 0x0800_0000_0000_0000;

        BaseCell::iter()
            .filter(move |&base_cell| base_cell.is_pentagon())
            .map(move |base_cell| {
                let bits = h3o_bit::set_base_cell(TEMPLATE, base_cell.into());
                let bits = bits::set_resolution(bits, self);

                CellIndex::new_unchecked(bits::set_unused(bits, self))
            })
    }

    /// Initializes a new `Resolution` using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid resolution.
    #[allow(unsafe_code)]
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        assert!(value <= h3o_bit::MAX_RESOLUTION, "resolution out of range");
        // SAFETY: range is checked above!
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }

    /// Returns the bit offset of the direction at this resolution in an H3
    /// index.
    pub(crate) fn direction_offset(self) -> usize {
        h3o_bit::direction_offset(self.into())
    }
}

impl From<Resolution> for usize {
    fn from(value: Resolution) -> Self {
        u8::from(value).into()
    }
}

impl From<Resolution> for u64 {
    fn from(value: Resolution) -> Self {
        u8::from(value).into()
    }
}

impl From<Resolution> for i16 {
    fn from(value: Resolution) -> Self {
        u8::from(value).into()
    }
}

impl From<Resolution> for u8 {
    fn from(value: Resolution) -> Self {
        value as Self
    }
}

impl TryFrom<c_int> for Resolution {
    type Error = error::InvalidResolution;

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| Self::Error::new(None, "c_int out of range"))
            .and_then(TryInto::try_into)
    }
}

impl TryFrom<u8> for Resolution {
    type Error = error::InvalidResolution;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            10 => Ok(Self::Ten),
            11 => Ok(Self::Eleven),
            12 => Ok(Self::Twelve),
            13 => Ok(Self::Thirteen),
            14 => Ok(Self::Fourteen),
            15 => Ok(Self::Fifteen),
            _ => Err(Self::Error::new(Some(value), "out of range")),
        }
    }
}

impl FromStr for Resolution {
    type Err = error::InvalidResolution;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u8::from_str(s)
            .map_err(|_| Self::Err::new(None, "invalid 8-bit number"))
            .and_then(Self::try_from)
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", u8::from(*self))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Resolution {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        u8::arbitrary(data).and_then(|byte| {
            Self::try_from(byte).map_err(|_| arbitrary::Error::IncorrectFormat)
        })
    }
}

// -----------------------------------------------------------------------------

/// Same as an H3 index resolution, but can goes up to 16.
///
/// This extended range is required for some intermediate calculation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ExtendedResolution(u8);

impl ExtendedResolution {
    /// Returns true if the resolution is a Class III resolution.
    pub const fn is_class3(self) -> bool {
        self.0 % 2 == 1
    }

    /// Move to the next finer resolution, below `current`.
    pub fn down(current: Resolution) -> Self {
        // Max value of `Resolution` is 15: we're always in bound (max 16).
        Self(u8::from(current) + 1)
    }
}

impl From<ExtendedResolution> for usize {
    fn from(value: ExtendedResolution) -> Self {
        value.0.into()
    }
}

impl From<Resolution> for ExtendedResolution {
    fn from(value: Resolution) -> Self {
        Self(value.into())
    }
}
