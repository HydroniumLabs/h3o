use super::RingHierarchy;
use crate::{error::OutlinerError, CellIndex, LatLng, Resolution};
use geo::{LineString, MultiPolygon, Polygon};
use std::collections::{btree_map::Entry, BTreeMap};

/// A single node in a vertex graph.
#[derive(Debug)]
pub struct Node {
    from: LatLng,
    to: LatLng,
}

/// A `BTreeMap` value.
#[derive(Debug)]
pub struct Value {
    /// End of the edge.
    vertex: LatLng,
    /// Is the edge part of the outline.
    is_outline: bool,
}

// -----------------------------------------------------------------------------

/// A data structure to store a graph of vertices.
#[derive(Default)]
pub struct VertexGraph {
    nodes: BTreeMap<LatLng, Vec<Value>>,
}

impl VertexGraph {
    /// Initializes a new `VertexGraph` from the given set of cells.
    pub fn from_cells(
        cells: impl IntoIterator<Item = CellIndex>,
    ) -> Result<Self, OutlinerError> {
        let mut cells = cells.into_iter();
        let mut graph = Self::default();
        let mut item = cells.next();
        let resolution =
            item.map_or_else(|| Resolution::Zero, CellIndex::resolution);

        while let Some(cell) = item {
            if cell.resolution() != resolution {
                return Err(OutlinerError::HeterogeneousResolution);
            }

            let boundary = cell.boundary();

            // Iterate through every edge.
            for i in 0..boundary.len() {
                let from = boundary[i];
                let to = boundary[(i + 1) % boundary.len()];

                graph.insert(&Node { from, to })?;
            }

            item = cells.next();
        }

        graph.prune();

        Ok(graph)
    }

    /// Adds an edge to the graph.
    pub fn insert(&mut self, node: &Node) -> Result<(), OutlinerError> {
        // First lookup the reversed edge.
        // If we've seen this edge already, it will be reversed.
        if let Entry::Occupied(mut entry) = self.nodes.entry(node.to) {
            if let Some(pos) = entry
                .get()
                .iter()
                .position(|value| value.vertex == node.from)
            {
                // Flag the edge as `not part of the outline`.
                if entry.get()[pos].is_outline {
                    entry.get_mut()[pos].is_outline = false;
                    return Ok(());
                }
                // If the edge was already flagged, we have a dup!
                return Err(OutlinerError::DuplicateInput);
            }
        }

        // New edge, insert it.
        let value = Value {
            vertex: node.to,
            is_outline: true,
        };
        match self.nodes.entry(node.from) {
            Entry::Occupied(mut entry) => {
                // Check if the node already exists.
                if entry.get().iter().any(|value| value.vertex == node.to) {
                    return Err(OutlinerError::DuplicateInput);
                }
                entry.get_mut().push(value);
            }
            Entry::Vacant(entry) => {
                // A vertex is shared by at most 3 edges.
                let values = entry.insert(Vec::with_capacity(3));
                values.push(value);
            }
        }

        Ok(())
    }

    /// Removes a node from the graph.
    pub fn remove(&mut self, node: &Node) {
        if let Entry::Occupied(mut entry) = self.nodes.entry(node.from) {
            if let Some(pos) =
                entry.get().iter().position(|value| value.vertex == node.to)
            {
                entry.get_mut().swap_remove(pos);
            }
            if entry.get().is_empty() {
                entry.remove_entry();
            }
        }
    }

    /// Remove the edges that are not part of an outline.
    pub fn prune(&mut self) {
        self.nodes.retain(|_, value| {
            value.retain(|value| value.is_outline);
            !value.is_empty()
        });
    }

    /// Finds a vertex node starting at the given vertex, if it exists.
    pub fn get_from_vertex(&self, from: LatLng) -> Option<Node> {
        self.nodes.get(&from).map(|to| Node {
            from,
            to: to[0].vertex,
        })
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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
            let mut node = Node {
                from,
                to: to[0].vertex,
            };
            loop {
                coords.push(node.from.into());
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
