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
