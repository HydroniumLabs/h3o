use super::{
    faceijk::FaceIJK, to_positive_angle, Vec2d, Vec3d, AP7_ROT_RADS, EPSILON,
    RES0_U_GNOMONIC, SQRT7_POWERS,
};
use crate::{
    error::InvalidLatLng, face, CellIndex, Face, Resolution, EARTH_RADIUS_KM,
    TWO_PI,
};
use float_eq::float_eq;
use std::{
    f64::consts::{FRAC_PI_2, PI},
    fmt,
};

/// Epsilon of ~0.1mm in degrees.
const EPSILON_DEG: f64 = 0.000000001;

/// Same as `EPSILON_DEG`, but in radians.
const EPSILON_RAD: f64 = EPSILON_DEG * PI / 180.0;

/// Latitude/longitude.
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
    pub fn new(lat: f64, lng: f64) -> Result<Self, InvalidLatLng> {
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
    pub fn from_radians(lat: f64, lng: f64) -> Result<Self, InvalidLatLng> {
        if !lat.is_finite() {
            return Err(InvalidLatLng::new(lat, "infinite latitude"));
        }
        if !lng.is_finite() {
            return Err(InvalidLatLng::new(lng, "infinite longitude"));
        }

        Ok(Self { lat, lng })
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
    pub fn lat(self) -> f64 {
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
    pub fn lng(self) -> f64 {
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
        let sin_lat = ((other.lat - self.lat) / 2.).sin();
        let sin_lng = ((other.lng - self.lng) / 2.).sin();

        let a = sin_lat.mul_add(
            sin_lat,
            self.lat.cos() * other.lat.cos() * sin_lng * sin_lng,
        );

        2. * a.sqrt().atan2((1. - a).sqrt())
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
        self.to_face_ijk(resolution).to_cell(resolution)
    }

    /// Encodes a coordinate on the sphere to the `FaceIJK` address of the
    /// containing cell at the specified resolution.
    ///
    /// # Arguments
    ///
    /// * `ll` - The spherical coordinates to encode.
    /// * `resolution` -  The desired H3 resolution for the encoding.
    /// * `face` -  The icosahedral face of the coordinate.
    /// * `distance` - The squared euclidean distance from the face center.
    pub(super) fn to_vec2d(
        self,
        resolution: Resolution,
        face: Face,
        distance: f64,
    ) -> Vec2d {
        let face = usize::from(face);

        let r = {
            // cos(r) = 1 - 2 * sin^2(r/2) = 1 - 2 * (sqd / 4) = 1 - sqd/2
            let r = (1. - distance / 2.).acos();

            if r < EPSILON {
                return Vec2d::new(0., 0.);
            }

            // Perform gnomonic scaling of `r` (`tan(r)`) and scale for current
            // resolution length `u`.
            (r.tan() / RES0_U_GNOMONIC) * SQRT7_POWERS[usize::from(resolution)]
        };

        let theta = {
            // Compute counter-clockwise `theta` from Class II i-axis.
            let mut theta = face::AXES_AZ_RADS_CII[face][0]
                - face::CENTER_GEO[face].azimuth(&self);

            // Adjust `theta` for Class III.
            if resolution.is_class3() {
                theta -= AP7_ROT_RADS;
            }
            theta
        };

        // Convert to local x, y.
        Vec2d::new(r * theta.cos(), r * theta.sin())
    }

    /// Finds the closest icosahedral face from the current coordinate.
    ///
    /// Returns both the face and the squared euclidean distance to that face
    /// center.
    #[must_use]
    pub(crate) fn closest_face(self) -> (Face, f64) {
        // The distance between two farthest points is 2.0, therefore the square
        // of the distance between two points should always be less or equal
        // than 4.
        const MAX_DIST: f64 = 5.0;

        let v3d = Vec3d::from(self);

        face::CENTER_POINT.iter().enumerate().fold(
            (Face::new_unchecked(0), MAX_DIST),
            |(face, distance), (i, center)| {
                let dist = v3d.distance(center);

                if dist < distance {
                    // SAFETY: `face` is always in range because it's a index of
                    // `CENTER_POINT`.
                    (Face::new_unchecked(i), dist)
                } else {
                    (face, distance)
                }
            },
        )
    }

    /// Computes the azimuth to `other` from `self`, in radians.
    #[must_use]
    pub(crate) fn azimuth(self, other: &Self) -> f64 {
        (other.lat.cos() * (other.lng - self.lng).sin()).atan2(
            self.lat.cos().mul_add(
                other.lat.sin(),
                -self.lat.sin()
                    * other.lat.cos()
                    * (other.lng - self.lng).cos(),
            ),
        )
    }

    /// Computes the point on the sphere a specified azimuth and distance from
    /// `self`.
    #[must_use]
    pub(crate) fn coord_at(self, azimuth: f64, distance: f64) -> Self {
        if distance < EPSILON {
            return self;
        }
        let azimuth = to_positive_angle(azimuth);
        let is_due_north_south = float_eq!(azimuth, 0.0, abs <= EPSILON)
            || float_eq!(azimuth, PI, abs <= EPSILON);

        // Compute latitude.
        let lat = if is_due_north_south {
            if float_eq!(azimuth, 0.0, abs <= EPSILON) {
                self.lat + distance // Due North.
            } else {
                self.lat - distance // Due South.
            }
        } else {
            self.lat
                .sin()
                .mul_add(
                    distance.cos(),
                    self.lat.cos() * distance.sin() * azimuth.cos(),
                )
                .clamp(-1., 1.)
                .asin()
        };

        // Handle poles.
        if float_eq!(lat, FRAC_PI_2, abs <= EPSILON) {
            return Self::new_unchecked(FRAC_PI_2, 0.0); // North pole.
        } else if float_eq!(lat, -FRAC_PI_2, abs <= EPSILON) {
            return Self::new_unchecked(-FRAC_PI_2, 0.0); // South pole.
        }

        // Compute longitude.
        let mut lng = if is_due_north_south {
            self.lng
        } else {
            let sinlng =
                (azimuth.sin() * distance.sin() / lat.cos()).clamp(-1., 1.);
            let coslng = self.lat.sin().mul_add(-lat.sin(), distance.cos())
                / self.lat.cos()
                / lat.cos();
            self.lng + sinlng.atan2(coslng)
        };

        // XXX: make sure longitudes are in the proper bounds.
        while lng > PI {
            lng -= TWO_PI;
        }
        while lng < -PI {
            lng += TWO_PI;
        }

        Self::new_unchecked(lat, lng)
    }

    /// Initializes a new coordinate with the specified, possibly invalid,
    /// values.
    ///
    /// # Safety
    ///
    /// The values must be finite numbers.
    #[must_use]
    pub(crate) const fn new_unchecked(lat: f64, lng: f64) -> Self {
        // TODO: wait for `is_finite` to be `const` in stable.
        // debug_assert!(lat.is_finite() && lng.is_finite());

        Self { lat, lng }
    }

    /// Encodes a coordinate on the sphere to the `FaceIJK` address of the
    /// containing cell at the specified resolution.
    ///
    /// # Arguments
    ///
    /// * `ll` - The spherical coordinates to encode.
    /// * `resolution` - The desired H3 resolution for the encoding.
    fn to_face_ijk(self, resolution: Resolution) -> FaceIJK {
        let (face, distance) = self.closest_face();
        let coord = self.to_vec2d(resolution, face, distance).into();

        FaceIJK::new(face, coord)
    }
}

impl PartialEq for LatLng {
    fn eq(&self, other: &Self) -> bool {
        float_eq!(self.lat, other.lat, abs <= EPSILON_RAD)
            && float_eq!(self.lng, other.lng, abs <= EPSILON_RAD)
    }
}

impl Eq for LatLng {}

impl From<LatLng> for Vec3d {
    /// Computes the 3D coordinate on unit sphere from the latitude and
    /// longitude.
    fn from(value: LatLng) -> Self {
        let r = value.lat.cos();

        let z = value.lat.sin();
        let x = value.lng.cos() * r;
        let y = value.lng.sin() * r;

        Self::new(x, y, z)
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

#[cfg(feature = "geo")]
impl From<LatLng> for geo::Coord {
    fn from(value: LatLng) -> Self {
        Self {
            x: value.lng(),
            y: value.lat(),
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

#[cfg(test)]
#[path = "./latlng_tests.rs"]
mod tests;
