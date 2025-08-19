use super::{RingHierarchy, neighbors};
use crate::{
    CellIndex, LatLng, Resolution, VertexIndex, error::DissolutionError,
};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use either::Either;
use geo::{LineString, MultiPolygon, Polygon};
use std::collections::hash_map::Entry;

/// A single node in a vertex graph.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Node {
    from: VertexIndex,
    to: VertexIndex,
}

/// A data structure to store a graph of vertices.
#[derive(Default)]
pub struct VertexGraph {
    nodes: HashMap<VertexIndex, Vec<VertexIndex>>,
    distortions: HashMap<Node, LatLng>,
    is_class3: bool,
}

impl VertexGraph {
    /// Initializes a new `VertexGraph` from a set of homogeneous cells.
    ///
    /// # Notes
    ///
    /// If `check_duplicate` is set to true, a duplicates detection is
    /// performed, which implies an eager consumption of the iterator upfront,
    /// incurring memory overhead and losing the lazyness of the iterator-based
    /// approach.
    pub fn from_homogeneous(
        cells: impl IntoIterator<Item = CellIndex>,
        check_duplicate: bool,
    ) -> Result<Self, DissolutionError> {
        let mut cells = if check_duplicate {
            Either::Left(check_duplicates(cells)?.into_iter())
        } else {
            Either::Right(cells.into_iter())
        };

        // Infer the resolution from the first cell (since its homogeneous).
        let first = cells.next();
        let resolution = first.map_or(Resolution::Zero, CellIndex::resolution);
        let cells = first.into_iter().chain(cells);

        let mut graph = Self {
            nodes: HashMap::new(),
            distortions: HashMap::new(),
            is_class3: resolution.is_class3(),
        };

        // Scratchpad to reuse memory allocations.
        let mut scratchpad = Scratchpad::new();
        for cell in cells {
            if cell.resolution() != resolution {
                return Err(DissolutionError::UnsupportedResolution);
            }

            scratchpad.compute_vertexes(cell);
            for pair in scratchpad.vertexes.windows(2) {
                graph.insert(&Node {
                    from: pair[0],
                    to: pair[1],
                });
            }

            // Keep track of distortions vertices when necessary.
            if graph.is_class3 && cell.icosahedron_faces().len() > 1 {
                graph.index_distortions(cell, &scratchpad.vertexes);
            }
        }

        Ok(graph)
    }

    /// Initializes a new `VertexGraph` from a set of heterogeneous cells.
    ///
    /// # Notes
    ///
    /// If `check_duplicate` is set to true, a duplicates detection is
    /// performed, which implies an eager consumption of the iterator upfront,
    /// incurring memory overhead and losing the lazyness of the iterator-based
    /// approach.
    pub fn from_heterogeneous(
        cells: impl IntoIterator<Item = CellIndex>,
        resolution: Resolution,
        check_duplicate: bool,
    ) -> Result<Self, DissolutionError> {
        let cells = if check_duplicate {
            let cells = cells.into_iter().collect::<Vec<_>>();
            if cells.iter().any(|cell| cell.resolution() > resolution) {
                return Err(DissolutionError::UnsupportedResolution);
            }
            check_duplicates(
                cells.iter().flat_map(|cell| cell.children(resolution)),
            )?;
            Either::Left(cells.into_iter())
        } else {
            Either::Right(cells.into_iter())
        };

        let mut graph = Self {
            nodes: HashMap::new(),
            distortions: HashMap::new(),
            is_class3: resolution.is_class3(),
        };

        // Scratchpad to reuse memory allocations.
        let mut scratchpad = Scratchpad::new();
        for cell in cells {
            match cell.resolution().cmp(&resolution) {
                std::cmp::Ordering::Less => {
                    graph.insert_large_cell(cell, resolution, &mut scratchpad);
                }
                std::cmp::Ordering::Equal => {
                    scratchpad.compute_vertexes(cell);
                    for pair in scratchpad.vertexes.windows(2) {
                        graph.insert(&Node {
                            from: pair[0],
                            to: pair[1],
                        });
                    }

                    // Keep track of distortions vertices when necessary.
                    if graph.is_class3 && cell.icosahedron_faces().len() > 1 {
                        graph.index_distortions(cell, &scratchpad.vertexes);
                    }
                }
                std::cmp::Ordering::Greater => {
                    return Err(DissolutionError::UnsupportedResolution);
                }
            }
        }

        Ok(graph)
    }

    /// Adds an edge to the graph.
    pub fn insert(&mut self, node: &Node) {
        // First lookup the reversed edge.
        // If we've seen this edge already, it will be reversed.
        if let Entry::Occupied(mut entry) = self.nodes.entry(node.to) {
            // Edge share by two cells: not part of the outline!
            if let Some(pos) =
                entry.get().iter().position(|&vertex| vertex == node.from)
            {
                entry.get_mut().swap_remove(pos);
                if entry.get().is_empty() {
                    entry.remove_entry();
                }
                self.distortions.remove(&Node {
                    from: node.to,
                    to: node.from,
                });
                return;
            }
        }

        // New edge, insert it.
        let nodes = self
            .nodes
            .entry(node.from)
            // A vertex is shared by at most 3 edges.
            .or_insert_with(|| Vec::with_capacity(3));
        nodes.push(node.to);
    }

    /// Removes a node from the graph.
    pub fn remove(&mut self, node: &Node) {
        if let Entry::Occupied(mut entry) = self.nodes.entry(node.from)
            && let Some(pos) =
                entry.get().iter().position(|&vertex| vertex == node.to)
        {
            entry.get_mut().swap_remove(pos);
            if entry.get().is_empty() {
                entry.remove_entry();
            }
            // XXX: distortions deletion is handled when injected.
        }
    }

    /// Finds a vertex node starting at the given vertex, if it exists.
    pub fn get_from_vertex(&self, from: VertexIndex) -> Option<Node> {
        self.nodes.get(&from).map(|to| Node { from, to: to[0] })
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Efficient insertion of cells larger than the target resolution.
    fn insert_large_cell(
        &mut self,
        cell: CellIndex,
        resolution: Resolution,
        scratchpad: &mut Scratchpad,
    ) {
        let boundary = compute_large_cell_boundary(
            cell,
            resolution,
            &mut scratchpad.neighbors,
        );

        for (cell, neighbors) in boundary {
            // Build a blacklist of edge by storing the reverse edge of the
            // neighbors to cancel the shared edges of the cell being examined.
            for candidate in neighbors {
                // Skip the cell being examined of course.
                if candidate == cell {
                    continue;
                }
                scratchpad.compute_vertexes(candidate);
                scratchpad.blacklist.extend(
                    scratchpad.vertexes.windows(2).map(|pair| Node {
                        to: pair[0],
                        from: pair[1],
                    }),
                );
            }

            scratchpad.compute_vertexes(cell);
            for pair in scratchpad.vertexes.windows(2) {
                let node = Node {
                    from: pair[0],
                    to: pair[1],
                };
                if !scratchpad.blacklist.contains(&node) {
                    self.insert(&node);
                }
            }

            // Keep track of distortions vertices when necessary.
            if self.is_class3 && cell.icosahedron_faces().len() > 1 {
                self.index_distortions(cell, &scratchpad.vertexes);
            }

            scratchpad.blacklist.clear();
        }
    }

    /// Index distortions vertices that exists between topological ones.
    fn index_distortions(&mut self, cell: CellIndex, vertexes: &[VertexIndex]) {
        // Boundary contains the every vertex (topological and distortions).
        let boundary = cell.boundary();
        let mut topological_idx = 0;

        for i in 0..boundary.len() {
            let vertex = boundary[i];

            if vertex == LatLng::from(vertexes[topological_idx]) {
                topological_idx = (topological_idx + 1) % vertexes.len();
            } else {
                let from = topological_idx
                    .checked_sub(1)
                    .unwrap_or(vertexes.len() - 1);
                self.distortions.insert(
                    Node {
                        from: vertexes[from],
                        to: vertexes[topological_idx],
                    },
                    vertex,
                );
            }
        }
    }
}

impl From<VertexGraph> for MultiPolygon<f64> {
    fn from(mut value: VertexGraph) -> Self {
        // No vertex, no shape.
        if value.is_empty() {
            return Self::new(Vec::new());
        }

        let mut rings = Vec::new();
        let mut coords = Vec::new();

        while !value.is_empty() {
            let (&from, to) =
                value.nodes.iter().next().expect("non-empty graph");
            let mut node = Node { from, to: to[0] };
            loop {
                coords.push(LatLng::from(node.from).into());
                // Inject distortion vertex, if any.
                if value.is_class3
                    && let Some(distortion) = value.distortions.remove(&node)
                {
                    coords.push(distortion.into());
                }

                let to = node.to;
                value.remove(&node);
                match value.get_from_vertex(to) {
                    Some(next_node) => node = next_node,
                    None => break,
                }
            }
            assert!(coords.len() >= 4);
            rings.push(LineString::new(coords.clone()));
            coords.clear();
        }

        // If we have a single ring, the resulting shape is obvious.
        if rings.len() == 1 {
            return Self::new(vec![Polygon::new(
                rings.swap_remove(0),
                Vec::new(),
            )]);
        }

        RingHierarchy::new(rings).into()
    }
}

// -----------------------------------------------------------------------------

struct Scratchpad {
    neighbors: [u64; 7],
    vertexes: Vec<VertexIndex>,
    blacklist: HashSet<Node>,
}

impl Scratchpad {
    fn new() -> Self {
        Self {
            // 6 neighbors + self.
            neighbors: [0; 7],
            // 6 vertexes + 1 to close the loop.
            vertexes: Vec::with_capacity(7),
            // 5 neighbors * 6 vertexes.
            blacklist: HashSet::with_capacity(30),
        }
    }

    fn compute_vertexes(&mut self, cell: CellIndex) {
        self.vertexes.clear();
        self.vertexes.extend(cell.vertexes());
        // Close the loop.
        // This simplify the iteration over edges by using `windows(2)`.
        self.vertexes.push(self.vertexes[0]);
    }
}

fn compute_large_cell_boundary(
    cell: CellIndex,
    resolution: Resolution,
    scratchpad: &mut [u64],
) -> HashMap<CellIndex, Vec<CellIndex>> {
    let cells = cell.children(resolution).collect::<HashSet<_>>();

    cells
        .iter()
        .copied()
        .filter_map(|cell| {
            let count = neighbors(cell, scratchpad);
            let is_boundary = scratchpad[0..count].iter().any(|neighbor| {
                !cells.contains(&CellIndex::new_unchecked(*neighbor))
            });

            is_boundary.then(|| {
                let neighbors = scratchpad[0..count]
                    .iter()
                    .filter_map(|&neighbor| {
                        let index = CellIndex::new_unchecked(neighbor);
                        cells.contains(&index).then_some(index)
                    })
                    .collect::<Vec<_>>();
                (cell, neighbors)
            })
        })
        .collect()
}

fn check_duplicates(
    cells: impl IntoIterator<Item = CellIndex>,
) -> Result<HashSet<CellIndex>, DissolutionError> {
    cells
        .into_iter()
        .try_fold(HashSet::default(), |mut acc, cell| {
            if acc.insert(cell) {
                Ok(acc)
            } else {
                Err(DissolutionError::DuplicateInput)
            }
        })
}
