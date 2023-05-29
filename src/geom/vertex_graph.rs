use super::RingHierarchy;
use crate::{error::OutlinerError, CellIndex, LatLng, Resolution};
use geo::{LineString, MultiPolygon, Polygon};
use std::{cmp::Ordering, collections::BTreeMap, ops::Bound};

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

#[derive(Copy, Clone)]
struct OrderedCoord(LatLng);

impl PartialEq for OrderedCoord {
    fn eq(&self, other: &Self) -> bool {
        (self.0.lat_radians(), self.0.lng_radians())
            == (other.0.lat_radians(), other.0.lng_radians())
    }
}

impl Eq for OrderedCoord {}

impl PartialOrd for OrderedCoord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.0.lat_radians(), self.0.lng_radians())
            .partial_cmp(&(other.0.lat_radians(), other.0.lng_radians()))
    }
}

impl Ord for OrderedCoord {
    fn cmp(&self, other: &Self) -> Ordering {
        // LatLng are guaranteed to be finite number.
        self.partial_cmp(other)
            .expect("LatLng number can be ordered")
    }
}

/// A data structure to store a graph of vertices.
#[derive(Default)]
pub struct VertexGraph {
    nodes: BTreeMap<OrderedCoord, Vec<Value>>,
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
        if let Some((_, entry)) = self.get_mut(node.to) {
            if let Some(pos) =
                entry.iter().position(|value| value.vertex == node.from)
            {
                // Flag the edge as `not part of the outline`.
                if entry[pos].is_outline {
                    entry[pos].is_outline = false;
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
        if let Some((_, entry)) = self.get_mut(node.from) {
            // Check if the node already exists.
            if entry.iter().any(|value| value.vertex == node.to) {
                return Err(OutlinerError::DuplicateInput);
            }
            entry.push(value);
        } else {
            // A vertex is shared by at most 3 edges.
            let mut values = Vec::with_capacity(3);
            values.push(value);
            self.nodes.insert(OrderedCoord(node.from), values);
        }

        Ok(())
    }

    /// Removes a node from the graph.
    pub fn remove(&mut self, node: &Node) {
        let del_key = if let Some((key, entry)) = self.get_mut(node.from) {
            if let Some(pos) =
                entry.iter().position(|value| value.vertex == node.to)
            {
                entry.swap_remove(pos);
            }
            entry.is_empty().then_some(*key)
        } else {
            None
        };
        if let Some(key) = del_key {
            self.nodes.remove(&key);
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
        self.get(from).map(|(_, to)| Node {
            from,
            to: to[0].vertex,
        })
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Lookup edges starting from `coord`.
    fn get_mut(
        &mut self,
        coord: LatLng,
    ) -> Option<(&OrderedCoord, &mut Vec<Value>)> {
        // Since LatLng relies on approximate equality, we have to first select a
        // range and then perform an equality test.
        let (lower, higher) = coord.bounds();
        let bounds = (
            Bound::Included(OrderedCoord(lower)),
            Bound::Included(OrderedCoord(higher)),
        );

        let mut values = self
            .nodes
            .range_mut(bounds)
            .filter(|&(key, _)| key.0 == coord);

        let nodes = values.next();
        assert!(values.next().is_none(), "expect a single match");

        nodes
    }

    fn get(&self, coord: LatLng) -> Option<(&OrderedCoord, &Vec<Value>)> {
        // Since LatLng relies on approximate equality, we have to first select a
        // range and then perform an equality test.
        let (lower, higher) = coord.bounds();
        let bounds = (
            Bound::Included(OrderedCoord(lower)),
            Bound::Included(OrderedCoord(higher)),
        );

        let mut values =
            self.nodes.range(bounds).filter(|&(key, _)| key.0 == coord);

        let nodes = values.next();
        assert!(values.next().is_none(), "expect a single match");

        nodes
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
                from: from.0,
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
