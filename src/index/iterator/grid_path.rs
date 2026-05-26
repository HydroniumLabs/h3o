use crate::{
    CellIndex,
    coord::{CoordCube, CoordIJK, LocalIJK},
    error::LocalIjError,
};
use core::{cmp::max, iter::FusedIterator};

/// Iterator over a children cell index at a given resolution.
#[derive(Debug, Clone)]
pub struct GridPathCells {
    /// Starting cell .
    anchor: CellIndex,
    /// Starting coordinate.
    start: CoordCube,
    // Path length.
    distance: i32,
    // Current position in the path.
    n: i32,

    /// Translation offset for the i component.
    i_step: f64,
    /// Translation offset for the j component.
    j_step: f64,
    /// Translation offset for the k component.
    k_step: f64,
}

impl GridPathCells {
    /// Returns an iterator over the children cell index at the given
    /// resolution.
    pub fn new(start: CellIndex, end: CellIndex) -> Result<Self, LocalIjError> {
        let anchor = start;

        // Get IJK coords for the start and end.
        let src = start.to_local_ijk(start)?;
        let dst = end.to_local_ijk(start)?;
        let distance = src.coord().distance(dst.coord());

        // Convert IJK to cube coordinates suitable for linear interpolation
        let start = CoordCube::from(*src.coord());
        let end = CoordCube::from(*dst.coord());

        let (i_step, j_step, k_step) = if distance == 0 {
            (0., 0., 0.)
        } else {
            let inv_distance = 1.0 / f64::from(distance);
            (
                f64::from(end.i - start.i) * inv_distance,
                f64::from(end.j - start.j) * inv_distance,
                f64::from(end.k - start.k) * inv_distance,
            )
        };

        Ok(Self {
            anchor,
            start,
            distance,
            n: 0,
            i_step,
            j_step,
            k_step,
        })
    }
}

impl Iterator for GridPathCells {
    type Item = Result<CellIndex, LocalIjError>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.n <= self.distance).then(|| {
            let coord = self.start.translate((
                self.i_step * f64::from(self.n),
                self.j_step * f64::from(self.n),
                self.k_step * f64::from(self.n),
            ));
            self.n += 1;

            // Convert cube -> ijk -> h3 index
            let local_ijk = LocalIJK {
                anchor: self.anchor,
                coord: CoordIJK::from(coord),
            };
            CellIndex::try_from(local_ijk)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = usize::try_from(max(self.distance - self.n, 0))
            .unwrap_or(usize::MAX);
        (count, Some(count))
    }
}

impl ExactSizeIterator for GridPathCells {}
impl FusedIterator for GridPathCells {}

#[cfg(test)]
#[path = "./grid_path_tests.rs"]
mod tests;
