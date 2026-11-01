use super::{FaceIJK, FaceProjection, LatLng, Vec3d};
use crate::{CellIndex, Resolution};
use h3o_bit::MAX_RESOLUTION;

/// Indexes `LatLng` coordinates at one or several resolutions.
///
/// Converting a coordinates into an H3 cell index can be split into two parts:
/// - converting the `LatLng` into a `FaceIJK`
/// - converting the `FaceIJK` into a `CellIndex` at the given resolution.
///
/// The first step is independent of the resolution, and coordinates close
/// together usually map to the same `FaceIJK`.
/// Thus, by caching the result of this first step (using a small bounded cache)
/// we get a significant speedup on some use cases, namely:
/// - indexing a single coordinates at different resolutions
/// - indexing a sequence of coordinates (eg pixels of a raster scanline)
#[derive(Debug, Clone, Default)]
pub struct LatLngIndexer {
    cache: [Option<(FaceIJK, CellIndex)>; MAX_RESOLUTION as usize + 1],
}

impl LatLngIndexer {
    /// Indexes the location at the specified resolution, returning the index of
    /// the cell containing the location.
    ///
    /// # Example
    ///
    /// Indexing a sequence of coordinates (eg pixels of a raster scanline).
    ///
    /// ```
    /// let mut indexer = h3o::LatLngIndexer::default();
    /// let cells = [2.3490, 2.3491, 2.3492].iter().map(|&lng| {
    ///     let ll = h3o::LatLng::new(48.8647, lng)?;
    ///     Ok(indexer.to_cell(ll, h3o::Resolution::Nine))
    /// })
    /// .collect::<Result<Vec<_>, _>>()?;
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    pub fn to_cell(&mut self, ll: LatLng, resolution: Resolution) -> CellIndex {
        self.lookup(FaceIJK::from_vec3d(ll.into(), resolution), resolution)
    }

    /// Indexes the location at each of the specified resolutions.
    ///
    /// # Example
    ///
    /// Indexing a single coordinates at different resolutions.
    ///
    /// ```
    /// use h3o::{LatLng, LatLngIndexer, Resolution};
    ///
    /// let cells = LatLngIndexer::default().to_cells(
    ///     LatLng::new(48.864716, 2.349014)?,
    ///     Resolution::range(Resolution::Zero, Resolution::Four)
    /// ).collect::<Vec<_>>();
    /// # Ok::<(), h3o::error::InvalidLatLng>(())
    /// ```
    pub fn to_cells(
        &mut self,
        ll: LatLng,
        resolutions: impl IntoIterator<Item = Resolution>,
    ) -> impl Iterator<Item = CellIndex> {
        let vec3d = Vec3d::from(ll);
        let (face, distance) = vec3d.closest_face();
        // Compute the resolution-independent projection.
        let mut projection = FaceProjection::new(vec3d, face, distance);

        resolutions
            .into_iter()
            .map(move |resolution| {
                let coord = projection.compute_vec2d(resolution);

                (FaceIJK::new(face, coord.into()), resolution)
            })
            .map(|(fijk, resolution)| self.lookup(fijk, resolution))
    }

    // Check if there is a previously computed result we can reuse.
    fn lookup(&mut self, fijk: FaceIJK, resolution: Resolution) -> CellIndex {
        let entry = &mut self.cache[usize::from(resolution)];

        // Cache hit (same `FaceIJK`): reuse the already computed cell.
        if let Some((prev_fijk, cell)) = entry
            && *prev_fijk == fijk
        {
            return *cell;
        }

        // Cache miss (different/too far coordinate): update the cache.
        let cell = fijk.to_cell(resolution);
        *entry = Some((fijk, cell));
        cell
    }
}
