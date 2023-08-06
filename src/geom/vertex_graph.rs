use super::RingHierarchy;
use crate::{error::OutlinerError, CellIndex, LatLng, Resolution, VertexIndex};
use ahash::{HashMap, HashMapExt};
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
    /// Initializes a new `VertexGraph` from the given set of cells.
    pub fn from_cells(
        cells: impl IntoIterator<Item = CellIndex>,
    ) -> Result<Self, OutlinerError> {
        // Detect duplicates in the input.
        // (sort + dedup is slower than HashSet but use less memory, especially
        // for large input).
        let mut cells = cells.into_iter().collect::<Vec<_>>();
        let old_len = cells.len();
        cells.sort_unstable();
        cells.dedup();
        if cells.len() < old_len {
            // Dups were removed, not good.
            return Err(OutlinerError::DuplicateInput);
        }

        let resolution = cells
            .first()
            .copied()
            .map_or_else(|| Resolution::Zero, CellIndex::resolution);
        let mut graph = Self {
            nodes: HashMap::new(),
            distortions: HashMap::new(),
            is_class3: resolution.is_class3(),
        };

        let mut vertexes = Vec::with_capacity(6);
        for cell in cells {
            if cell.resolution() != resolution {
                return Err(OutlinerError::HeterogeneousResolution);
            }

            for vertex in cell.vertexes() {
                vertexes.push(vertex);
            }

            // Iterate through every edge.
            for i in 0..vertexes.len() {
                let from = vertexes[i];
                let to = vertexes[(i + 1) % vertexes.len()];

                graph.insert(&Node { from, to });
            }

            // Keep track of distortions vertices when necessary.
            if graph.is_class3 && cell.icosahedron_faces().len() > 1 {
                graph.index_distorsions(cell, &vertexes);
            }

            vertexes.clear();
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
        if let Entry::Occupied(mut entry) = self.nodes.entry(node.from) {
            if let Some(pos) =
                entry.get().iter().position(|&vertex| vertex == node.to)
            {
                entry.get_mut().swap_remove(pos);
                if entry.get().is_empty() {
                    entry.remove_entry();
                }
                // XXX: distortions deletion is handled when injected.
            }
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

    /// Index distortions vertices that exists between topological ones.
    fn index_distorsions(&mut self, cell: CellIndex, vertexes: &[VertexIndex]) {
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
                if value.is_class3 {
                    if let Some(distortion) = value.distortions.remove(&node) {
                        coords.push(distortion.into());
                    }
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
