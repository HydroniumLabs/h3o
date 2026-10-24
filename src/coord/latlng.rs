use super::{EPSILON_RAD, FaceIJK, FaceProjection, Vec3d};
use crate::{
    CellIndex, EARTH_RADIUS_KM, Resolution,
    error::InvalidLatLng,
    math::{Coord2d, asin, atan2, cos, mul_add, sin, sin_cos, sqrt},
};
use core::fmt;
use float_eq::float_eq;

/// Latitude/longitude.
///
/// The coordinate reference system (CRS) is sphere coordinates with the
/// WGS84/EPSG:4326 authalic radius.
///
/// Note that the `Display` impl prints the values as degrees (10 decimals at
/// most), while the `Debug` impl prints both degrees and radians.
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LatLng {
    /// Latitude, in radians.
    lat: f64,
    /// Longitude, in radians.
    lng: f64,
}

impl LatLng {
    /// Initializes a new coordinate from degrees.
    ///
    /// # Errors
    ///
    /// [`InvalidLatLng`] when one (or both) components is not a finite number.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    ///
    /// assert!(h3o::LatLng::new(f64::NAN, 10.).is_err());
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    pub const fn new(lat: f64, lng: f64) -> Result<Self, InvalidLatLng> {
        Self::from_radians(lat.to_radians(), lng.to_radians())
    }

    /// Initializes a new coordinate from radians.
    ///
    /// # Errors
    ///
    /// [`InvalidLatLng`] when one (or both) components is not a finite number.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::from_radians(0.852850182, 0.0409980285)?;
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    pub const fn from_radians(
        lat: f64,
        lng: f64,
    ) -> Result<Self, InvalidLatLng> {
        if !lat.is_finite() {
            return Err(InvalidLatLng::new(lat, "infinite latitude"));
        }
        if !lng.is_finite() {
            return Err(InvalidLatLng::new(lng, "infinite longitude"));
        }

        Ok(Self { lat, lng })
    }

    /// Initializes from a geo coordinate (in degrees).
    ///
    /// # Errors
    ///
    /// [`InvalidLatLng`] when one (or both) components is not a finite number.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::from_coord(&(181.2, 51.79))?;
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[cfg(feature = "geo")]
    pub fn from_coord<T>(value: &T) -> Result<Self, InvalidLatLng>
    where
        T: geo_traits::CoordTrait<T = f64>,
    {
        Self::new(value.y(), value.x())
    }

    /// Latitude, in degrees.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    ///
    /// assert_eq!(ll.lat(), 48.864716);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub const fn lat(self) -> f64 {
        self.lat.to_degrees()
    }

    /// Longitude, in degrees.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    ///
    /// assert_eq!(ll.lng(), 2.349014);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub const fn lng(self) -> f64 {
        self.lng.to_degrees()
    }

    /// Latitude, in radians.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    ///
    /// assert_eq!(ll.lat_radians(), 0.8528501822519535);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub const fn lat_radians(self) -> f64 {
        self.lat
    }

    /// Longitude, in degrees.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    ///
    /// assert_eq!(ll.lng_radians(), 0.04099802847544208);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub const fn lng_radians(self) -> f64 {
        self.lng
    }

    /// The great circle distance, in radians, between two spherical
    /// coordinates.
    ///
    /// This function uses the Haversine formula.
    ///
    /// For math details, see:
    /// - <https://en.wikipedia.org/wiki/Haversine_formula/>
    /// - <https://www.movable-type.co.uk/scripts/latlong.html/>
    ///
    /// # Example
    ///
    /// ```
    /// let src = h3o::LatLng::new(48.864716, 2.349014)?;
    /// let dst = h3o::LatLng::new(31.224361, 121.469170)?;
    ///
    /// assert_eq!(src.distance_rads(dst), 1.453859220532047);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub fn distance_rads(self, other: Self) -> f64 {
        let sin_lat = sin((other.lat - self.lat) * 0.5);
        let sin_lng = sin((other.lng - self.lng) * 0.5);

        let a = mul_add(
            sin_lat,
            sin_lat,
            cos(self.lat) * cos(other.lat) * sin_lng * sin_lng,
        );

        2. * atan2(sqrt(a), sqrt(1. - a))
    }

    /// The great circle distance, in kilometers, between two spherical
    /// coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// let src = h3o::LatLng::new(48.864716, 2.349014)?;
    /// let dst = h3o::LatLng::new(31.224361, 121.469170)?;
    ///
    /// assert_eq!(src.distance_km(dst), 9262.547534054209);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub fn distance_km(self, other: Self) -> f64 {
        self.distance_rads(other) * EARTH_RADIUS_KM
    }

    /// The great circle distance, in meters, between two spherical coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// let src = h3o::LatLng::new(48.864716, 2.349014)?;
    /// let dst = h3o::LatLng::new(31.224361, 121.469170)?;
    ///
    /// assert_eq!(src.distance_m(dst), 9262547.534054209);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub fn distance_m(self, other: Self) -> f64 {
        self.distance_km(other) * 1000.
    }

    /// Indexes the location at the specified resolution, returning the index of
    /// the cell containing the location.
    ///
    /// # Example
    ///
    /// ```
    /// let ll = h3o::LatLng::new(48.864716, 2.349014)?;
    /// let cell = ll.to_cell(h3o::Resolution::Five);
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    #[must_use]
    pub fn to_cell(self, resolution: Resolution) -> CellIndex {
        Vec3d::from(self).to_cell(resolution)
    }

    /// Returns the `FaceIJK` address of the location at each resolution,
    /// sharing the resolution-independent projection work.
    fn to_face_ijks(
        self,
        resolutions: impl IntoIterator<Item = Resolution>,
    ) -> impl Iterator<Item = (FaceIJK, Resolution)> {
        let value = Vec3d::from(self);
        let (face, distance) = value.closest_face();
        let projection = FaceProjection::new(value, face, distance);
        // `sin_cos` of the Class II and Class III angles, computed on demand.
        let mut sin_cos_by_class = [None; 2];

        resolutions.into_iter().map(move |resolution| {
            let class = usize::from(resolution.is_class3());
            let coord = projection.to_vec2d_with(resolution, |theta| {
                *sin_cos_by_class[class].get_or_insert_with(|| sin_cos(theta))
            });
            (FaceIJK::new(face, coord.into()), resolution)
        })
    }

    #[cfg(any(test, feature = "typed_floats"))]
    pub(crate) const fn new_unchecked(lat: f64, lng: f64) -> Self {
        debug_assert!(lat.is_finite() && lng.is_finite());

        Self { lat, lng }
    }
}

/// Indexes locations at one or several resolutions, reusing work between
/// resolutions and between nearby locations.
///
/// Every returned cell is identical to [`LatLng::to_cell`], but indexing is
/// faster when:
///
/// - indexing a location at several resolutions with [`Self::to_cells`]: the
///   resolution-independent work (face selection and gnomonic projection) is
///   done once for all of them;
/// - indexing a sequence of nearby locations, such as the pixels of a raster
///   scanline, with the same indexer: the cell index is a function of the
///   face-centered `IJK` coordinates of the containing cell, which consecutive
///   nearby locations usually share, so the indexer remembers the last
///   coordinates and cell at each resolution and only encodes a new cell
///   index when they change.
///
/// # Example
///
/// ```
/// use h3o::{LatLng, LatLngIndexer, Resolution};
///
/// let resolutions = [Resolution::Six, Resolution::Nine];
/// let mut indexer = LatLngIndexer::new();
/// for lng in [2.3490, 2.3491, 2.3492] {
///     let ll = LatLng::new(48.8647, lng)?;
///     for (cell, res) in indexer.to_cells(ll, resolutions).zip(resolutions) {
///         assert_eq!(cell, ll.to_cell(res));
///     }
///     assert_eq!(
///         indexer.to_cell(ll, Resolution::Ten),
///         ll.to_cell(Resolution::Ten)
///     );
/// }
/// # Ok::<(), h3o::error::InvalidLatLng>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct LatLngIndexer {
    /// Last `FaceIJK` address and cell at each resolution.
    last: [Option<(FaceIJK, CellIndex)>; h3o_bit::MAX_RESOLUTION as usize + 1],
}

impl LatLngIndexer {
    /// Initializes an indexer with no remembered cells.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes the location at the specified resolution.
    ///
    /// Equivalent to [`LatLng::to_cell`].
    pub fn to_cell(&mut self, ll: LatLng, resolution: Resolution) -> CellIndex {
        let fijk = FaceIJK::from_vec3d(Vec3d::from(ll), resolution);
        self.encode(fijk, resolution)
    }

    /// Indexes the location at each of the specified resolutions.
    ///
    /// Equivalent to calling [`LatLng::to_cell`] for each resolution, in
    /// order.
    pub fn to_cells(
        &mut self,
        ll: LatLng,
        resolutions: impl IntoIterator<Item = Resolution>,
    ) -> impl Iterator<Item = CellIndex> {
        ll.to_face_ijks(resolutions)
            .map(|(fijk, resolution)| self.encode(fijk, resolution))
    }

    fn encode(&mut self, fijk: FaceIJK, resolution: Resolution) -> CellIndex {
        let last = &mut self.last[usize::from(resolution)];
        match *last {
            Some((last_fijk, cell)) if last_fijk == fijk => cell,
            _ => {
                let cell = fijk.to_cell(resolution);
                *last = Some((fijk, cell));
                cell
            }
        }
    }
}

impl PartialEq for LatLng {
    fn eq(&self, other: &Self) -> bool {
        float_eq!(self.lat, other.lat, abs <= EPSILON_RAD)
            && float_eq!(self.lng, other.lng, abs <= EPSILON_RAD)
    }
}

impl Eq for LatLng {}

impl From<Vec3d> for LatLng {
    #[inline]
    fn from(value: Vec3d) -> Self {
        Self {
            lat: asin(value.z),
            lng: atan2(value.y, value.x),
        }
    }
}

impl fmt::Display for LatLng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For display purpose, 10 decimals be more than enough.
        // See https://gis.stackexchange.com/a/8674
        write!(f, "({:.10}, {:.10})", self.lat(), self.lng())
    }
}

impl fmt::Debug for LatLng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LatLng")
            .field("lat_rad", &self.lat)
            .field("lat_deg", &self.lat())
            .field("lng_rad", &self.lng)
            .field("lng_deg", &self.lng())
            .finish()
    }
}

impl Coord2d for LatLng {
    fn xy(self) -> (f64, f64) {
        (self.lng, self.lat)
    }
}

#[cfg(feature = "geo")]
impl From<LatLng> for geo::Coord {
    fn from(value: LatLng) -> Self {
        Self {
            x: value.lng(),
            y: value.lat(),
        }
    }
}

#[cfg(feature = "typed_floats")]
mod typed_floats {
    // Types for readability
    type TFCoord = typed_floats::NonNaNFinite<f64>;
    type TFLatlng = (TFCoord, TFCoord);

    impl From<TFLatlng> for crate::LatLng {
        fn from(latlng: TFLatlng) -> Self {
            // SAFETY: `NonNaNFinite` guarantees that the values are finite.
            Self::new_unchecked(latlng.0.into(), latlng.1.into())
        }
    }
}

#[cfg(feature = "geo")]
impl TryFrom<geo::Coord> for LatLng {
    type Error = InvalidLatLng;

    fn try_from(value: geo::Coord) -> Result<Self, Self::Error> {
        Self::new(value.y, value.x)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for LatLng {
    fn arbitrary(
        data: &mut arbitrary::Unstructured<'a>,
    ) -> arbitrary::Result<Self> {
        let lat = f64::arbitrary(data)?;
        let lng = f64::arbitrary(data)?;

        Self::from_radians(lat, lng)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(feature = "geo")]
impl geo_traits::CoordTrait for LatLng {
    type T = f64;

    #[expect(clippy::panic, reason = "panic by design")]
    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("LatLng only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.lng()
    }

    fn y(&self) -> Self::T {
        self.lat()
    }
}
