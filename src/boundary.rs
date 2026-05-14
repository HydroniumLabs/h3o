use crate::LatLng;
use core::{fmt, ops::Deref};

/// Maximum number of cell boundary vertices.
///
/// Worst case is pentagon: 5 original verts + 5 edge crossings.
const MAX_BNDRY_VERTS: usize = 10;

/// Boundary in latitude/longitude.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct Boundary {
    /// Vertices in CCW order.
    points: [LatLng; MAX_BNDRY_VERTS],
    /// Number of vertices.
    count: u8,
}

impl Boundary {
    /// Initializes a new empty cell boundary (test only)
    #[must_use]
    #[doc(hidden)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertices to the boundary (test only).
    #[doc(hidden)]
    pub fn push(&mut self, ll: LatLng) {
        self.points[usize::from(self.count)] = ll;
        self.count += 1;
    }
}

impl Deref for Boundary {
    type Target = [LatLng];

    /// Dereference to the slice of filled elements.
    fn deref(&self) -> &Self::Target {
        &self.points[..self.count.into()]
    }
}

impl fmt::Display for Boundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, ll) in self.iter().enumerate() {
            if i != 0 {
                write!(f, "-")?;
            }
            write!(f, "{ll}")?;
        }
        write!(f, "]")
    }
}

#[cfg(feature = "geo")]
impl From<Boundary> for geo::LineString {
    fn from(value: Boundary) -> Self {
        Self::new(value.iter().copied().map(geo::Coord::from).collect())
    }
}

#[cfg(feature = "geo")]
impl geo_traits::GeometryTrait for Boundary {
    type GeometryCollectionType<'b>
        = geo_traits::UnimplementedGeometryCollection<f64>
    where
        Self: 'b;
    type LineStringType<'b>
        = Self
    where
        Self: 'b;
    type LineType<'b>
        = geo_traits::UnimplementedLine<f64>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = geo_traits::UnimplementedMultiLineString<f64>
    where
        Self: 'b;
    type MultiPointType<'b>
        = geo_traits::UnimplementedMultiPoint<f64>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = geo_traits::UnimplementedMultiPolygon<f64>
    where
        Self: 'b;
    type PointType<'b>
        = geo_traits::UnimplementedPoint<f64>
    where
        Self: 'b;
    type PolygonType<'b>
        = Self
    where
        Self: 'b;
    type RectType<'b>
        = geo_traits::UnimplementedRect<f64>
    where
        Self: 'b;
    type T = f64;
    type TriangleType<'b>
        = geo_traits::UnimplementedTriangle<f64>
    where
        Self: 'b;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        if self.count >= crate::NUM_PENT_VERTS {
            geo_traits::GeometryType::Polygon(self)
        } else {
            geo_traits::GeometryType::LineString(self)
        }
    }
}

#[cfg(feature = "geo")]
impl geo_traits::LineStringTrait for Boundary {
    type CoordType<'a> = LatLng;

    fn num_coords(&self) -> usize {
        self.count.into()
    }

    /// Access to a specified coordinate in this boundary.
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    #[expect(unsafe_code, reason = "precondition must be held by the caller")]
    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        // SAFETY: precondition must be held by the caller.
        unsafe { *self.get_unchecked(i) }
    }
}

#[cfg(feature = "geo")]
impl geo_traits::PolygonTrait for Boundary {
    type RingType<'a>
        = geo::LineString
    where
        Self: 'a;

    /// Returns the exsterior ring for pentagons and hexagons.
    /// For edges, `None` is returned.
    fn exterior(&self) -> Option<Self::RingType<'_>> {
        (self.count >= crate::NUM_PENT_VERTS).then(|| {
            let mut coords = Vec::with_capacity((self.count + 1).into());
            coords.extend(self.iter().copied().map(geo::Coord::from));
            coords.push(coords[0]);

            geo::LineString::new(coords)
        })
    }

    /// H3 cell have no interrior rings.
    fn num_interiors(&self) -> usize {
        0
    }

    /// # Safety
    ///
    /// This method always panic, H3 cells have no interior rings.
    #[expect(unsafe_code, reason = "always panic, it's safe")]
    unsafe fn interior_unchecked(&self, _i: usize) -> Self::RingType<'_> {
        unreachable!("Boundary has no interior rings")
    }
}
