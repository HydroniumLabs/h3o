use geo::{Contains, Coord, LineString, MultiPolygon, Polygon};
use std::iter::Peekable;

/// A rings hierarchy.
pub struct RingHierarchy {
    /// Rings geometry.
    rings: Vec<LineString<f64>>,

    /// Bitmap keeping track of which ring have already been assigned to a
    /// polygon.
    is_assigned: Vec<bool>,

    /// Hierarchy matrix.
    ///
    /// Rows represent the `contained by` relation and columns represent the
    /// `contains` one.
    ///
    /// # Example
    ///
    /// The following rings:
    ///
    /// ```text
    // ┏━A━━━━━┓
    // ┃┏━B━━━┓┃
    // ┃┃┏━━C┓┃┃
    // ┃┃┃┏━┓┃┃┃
    // ┃┃┃┗D┛┃┃┃
    // ┃┃┗━━━┛┃┃
    // ┃┗━━━━━┛┃
    // ┗━━━━━━━┛
    /// ```
    ///
    /// Will generate this matrix
    ///
    /// ```text
    /// ┌───┬───┬───┬───┬───┐
    /// │   │ A │ B │ C │ D │
    /// ├───┼───┼───┼───┼───┤
    /// │ A │   │   │   │   │
    /// ├───┼───┼───┼───┼───┤
    /// │ B │ 1 │   │   │   │
    /// ├───┼───┼───┼───┼───┤
    /// │ C │ 1 │ 1 │   │   │
    /// ├───┼───┼───┼───┼───┤
    /// │ D │ 1 │ 1 │ 1 │   │
    /// └───┴───┴───┴───┴───┘
    /// ```
    matrix: Vec<bool>,
}

impl RingHierarchy {
    /// Builds a new hierarchy of rings.
    pub fn new(rings: Vec<LineString<f64>>) -> Self {
        let is_assigned = vec![false; rings.len()];

        // Compute the hierarchy matrix.
        let mut matrix = vec![false; rings.len() * rings.len()];
        for (i, r1) in rings.iter().enumerate() {
            for (j, r2) in rings.iter().enumerate() {
                // One cannot contains itself.
                if i == j {
                    continue;
                }

                let r1 = LineString::new(
                    r1.coords().map(adjust_coordinate).collect(),
                );
                // We are guaranteed not to overlap, so just test the first
                // point.
                let r2 = adjust_coordinate(&r2.0[0]);
                // Need to convert to Polygon to have the right `contains`
                // algorithm.
                if Polygon::new(r1, vec![]).contains(&r2) {
                    matrix[j * rings.len() + i] = true;
                }
            }
        }

        Self {
            rings,
            is_assigned,
            matrix,
        }
    }

    /// Consumes the hierarchy into a stream of Polygon.
    pub fn into_iter(mut self) -> impl Iterator<Item = Polygon<f64>> {
        type OuterRingIterator =
            Peekable<std::vec::IntoIter<(usize, LineString<f64>)>>;

        // Outers ring at the current nesting level.
        let mut outers: Option<OuterRingIterator> = None;

        std::iter::from_fn(move || {
            // If the current layer is exhausted, peel the next one.
            if outers.as_mut().map_or(true, |rings| rings.peek().is_none()) {
                outers = self
                    .peel_outers()
                    .map(|rings| rings.into_iter().peekable());
            }

            outers.as_mut().map(|outers| {
                let (id, outer) = outers.next().expect("peeked above");
                let inners = self.inners(id);

                // Mark the outer as assigned.
                self.is_assigned[id] = true;

                Polygon::new(outer, inners)
            })
        })
    }

    /// Peels one layer of outer rings and return them.
    ///
    /// An outer ring is a ring that is not contained by any other ring.
    ///
    /// Note that in case of nested polygons, you may need to call this function
    /// several time to extract outer rings at every nesting level.
    fn peel_outers(&mut self) -> Option<Vec<(usize, LineString<f64>)>> {
        #[allow(clippy::filter_map_bool_then)] // Borrow issue if filter+map
        let outers = (0..self.rings.len())
            .filter_map(|i| {
                // Skip assigned rings and non-outer ones.
                (!self.is_assigned[i] && self.is_outer(i)).then(|| {
                    // Extract the ring in place to preserve `rings` size and
                    // ordering.
                    let ring = std::mem::replace(
                        &mut self.rings[i],
                        LineString(vec![]),
                    );
                    (i, ring)
                })
            })
            .collect::<Vec<_>>();

        (!outers.is_empty()).then_some(outers)
    }

    /// Returns the inners of the given outer ring.
    ///
    /// An inner ring belongs to an outer ring if it's only contained by this
    /// outer ring.
    fn inners(&mut self, outer_id: usize) -> Vec<LineString<f64>> {
        // Walk by column to find candidate and then check their parents using
        // the row-order
        #[allow(clippy::filter_map_bool_then)] // Borrow issue if filter+map
        let (ids, rings) = (0..self.rings.len())
            .filter_map(|inner_id| {
                (!self.is_assigned[inner_id]
                    && self.belongs_to(inner_id, outer_id))
                .then(|| {
                    // Extract the ring in place to preserve `rings` size and
                    // ordering.
                    let ring = std::mem::replace(
                        &mut self.rings[inner_id],
                        LineString(vec![]),
                    );
                    (inner_id, ring)
                })
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();

        // Mark the ring as assigned.
        for inner_id in ids {
            self.is_assigned[inner_id] = true;
        }

        rings
    }

    /// Tests if `outer` contains `inner`.
    fn contains(&self, outer: usize, inner: usize) -> bool {
        self.matrix[inner * self.rings.len() + outer]
    }

    /// Tests if the given ring is an outer ring (e.g. row `id` is all false).
    fn is_outer(&self, id: usize) -> bool {
        self.count_parents(id) == 0
    }

    /// Tests if `inner_id` belongs to to `outer_id`.
    ///
    /// A ring belong to another one if it's a direct child of it.
    fn belongs_to(&self, inner_id: usize, outer_id: usize) -> bool {
        self.contains(outer_id, inner_id) && self.count_parents(inner_id) == 1
    }

    /// Counts how many ring contains the ring `id`.
    fn count_parents(&self, id: usize) -> usize {
        (0..self.rings.len()).fold(0, |acc, i| {
            if self.is_assigned[i] {
                return acc;
            }
            acc + usize::from(self.matrix[id * self.rings.len() + i])
        })
    }
}

impl From<RingHierarchy> for MultiPolygon<f64> {
    fn from(value: RingHierarchy) -> Self {
        Self(value.into_iter().collect())
    }
}

// -----------------------------------------------------------------------------

// Adjusts coordinates to handle transmeridian crossing.
fn adjust_coordinate(coord: &Coord) -> Coord {
    Coord {
        x: f64::from(u8::from(coord.x < 0.) * 2).mul_add(180., coord.x),
        y: coord.y,
    }
}

#[cfg(test)]
#[path = "./ring_hierarchy_tests.rs"]
mod tests;
