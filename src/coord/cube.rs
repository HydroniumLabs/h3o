use super::CoordIJK;

/// Cube coordinates.
///
/// Cube coordinates are more suitable than `IJK` for linear interpolation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CoordCube {
    /// `i` component.
    pub i: i32,
    /// `j` component.
    pub j: i32,
    /// `k` component.
    pub k: i32,
}

impl CoordCube {
    /// Initializes a new cube coordinate with the specified component values.
    pub const fn new(i: i32, j: i32, k: i32) -> Self {
        Self { i, j, k }
    }

    /// Translate the coordinates by the specified offset.
    ///
    /// Algorithm from <https://www.redblobgames.com/grids/hexagons/#rounding/>
    pub fn translate(&self, offsets: (f64, f64, f64)) -> Self {
        let i = f64::from(self.i) + offsets.0;
        let j = f64::from(self.j) + offsets.1;
        let k = f64::from(self.k) + offsets.2;

        #[allow(clippy::cast_possible_truncation)] // on purpose
        let (mut ri, mut rj, mut rk) =
            { (i.round() as i32, j.round() as i32, k.round() as i32) };

        let i_diff = (f64::from(ri) - i).abs();
        let j_diff = (f64::from(rj) - j).abs();
        let k_diff = (f64::from(rk) - k).abs();

        // Round, maintaining valid cube coords.
        if i_diff > j_diff && i_diff > k_diff {
            ri = -rj - rk;
        } else if j_diff > k_diff {
            rj = -ri - rk;
        } else {
            rk = -ri - rj;
        }

        Self::new(ri, rj, rk)
    }
}

impl From<CoordCube> for CoordIJK {
    fn from(value: CoordCube) -> Self {
        Self::new(-value.i, value.j, 0).normalize()
    }
}
