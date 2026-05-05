mod avg_edge_len;
mod base_cell;
mod boundary;
mod cell_index;
mod directed_edge_index;
mod direction;
mod edge;
mod face;
mod face_set;
#[cfg(feature = "geo")]
mod geom;
mod index_mode;
mod latlng;
mod localij;
mod resolution;
mod vertex;
mod vertex_index;

#[test]
fn max_grid_disk_size_overflow() {
    assert_eq!(h3o::max_grid_disk_size(4294967295), 569_707_381_193_162);
}

#[test]
fn is_valid_index() {
    assert!(h3o::is_valid_index(0x85754e67fffffff), "CellIndex");
    assert!(h3o::is_valid_index(0x145754e67fffffff), "DirectedEdgeIndex");
    assert!(h3o::is_valid_index(0x225754a93fffffff), "VertexIndex");

    assert!(!h3o::is_valid_index(0), "invalid cell");
    assert!(!h3o::is_valid_index(0x885754e67fffffff), "corrupted cell");
}
