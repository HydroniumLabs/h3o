use criterion::{criterion_group, criterion_main};

mod are_neighbor_cells;
mod cell_area;
mod cell_range;
mod cell_to_boundary;
mod cell_to_center_child;
mod cell_to_child_pos;
mod cell_to_children;
mod cell_to_children_size;
mod cell_to_latlng;
mod cell_to_local_ij;
mod cell_to_parent;
mod cell_to_vertex;
mod cell_to_vertexes;
mod cells_to_directed_edge;
mod child_pos_to_cell;
mod compact_cells;
mod constants;
mod degs_to_rads;
mod directed_edge_to_boundary;
mod directed_edge_to_cells;
mod edge_length;
mod get_base_cell_number;
mod get_directed_edge_destination;
mod get_directed_edge_origin;
mod get_hexagon_area_avg;
mod get_hexagon_edge_length_avg;
mod get_icosahedron_faces;
mod get_num_cells;
mod get_pentagons;
mod get_res0_cells;
mod get_resolution;
mod great_circle_distance;
mod grid_disk;
mod grid_disk_distances;
mod grid_disk_distances_safe;
mod grid_disk_distances_unsafe;
mod grid_disk_unsafe;
mod grid_disks_unsafe;
mod grid_distance;
mod grid_path_cells;
mod grid_path_cells_size;
mod grid_ring_unsafe;
mod h3_to_string;
mod is_pentagon;
mod is_res_class3;
mod is_valid_cell;
mod is_valid_directed_edge;
mod is_valid_vertex;
mod latlng_to_cell;
mod local_ij_to_cell;
mod max_face_count;
mod max_grid_disk_size;
mod origin_to_directed_edges;
mod pentagon_count;
mod rads_to_degs;
mod res0_cell_count;
mod string_to_h3;
mod uncompact_cells;
mod uncompact_cells_size;
mod vertex_to_latlng;

#[cfg(feature = "geo")]
mod h3_set_lo_linked_geo;
#[cfg(feature = "geo")]
mod max_polygon_to_cells_size;
#[cfg(feature = "geo")]
mod polygon_to_cells;
#[cfg(feature = "geo")]
mod utils;

criterion_group!(
    benches,
    are_neighbor_cells::bench,
    cell_area::bench_km2,
    cell_area::bench_m2,
    cell_area::bench_rads2,
    cell_range::bench_succ,
    cell_range::bench_pred,
    cell_to_boundary::bench,
    cell_to_center_child::bench,
    cell_to_children_size::bench,
    cell_to_child_pos::bench,
    cell_to_children::bench,
    cell_to_latlng::bench,
    cell_to_local_ij::bench,
    cell_to_parent::bench,
    cell_to_vertex::bench,
    cell_to_vertexes::bench,
    cells_to_directed_edge::bench,
    child_pos_to_cell::bench,
    compact_cells::bench,
    degs_to_rads::bench,
    directed_edge_to_boundary::bench,
    directed_edge_to_cells::bench,
    edge_length::bench_km,
    edge_length::bench_m,
    edge_length::bench_rads,
    get_base_cell_number::bench,
    get_directed_edge_destination::bench,
    get_directed_edge_origin::bench,
    get_hexagon_area_avg::bench_km2,
    get_hexagon_area_avg::bench_m2,
    get_hexagon_edge_length_avg::bench_km,
    get_hexagon_edge_length_avg::bench_m,
    great_circle_distance::bench_km,
    great_circle_distance::bench_m,
    great_circle_distance::bench_rads,
    grid_disk::bench,
    grid_disk_distances::bench,
    grid_disk_distances_safe::bench,
    grid_disk_distances_unsafe::bench,
    grid_disk_unsafe::bench,
    grid_disks_unsafe::bench,
    grid_distance::bench,
    grid_path_cells_size::bench,
    grid_path_cells::bench,
    grid_ring_unsafe::bench,
    get_icosahedron_faces::bench,
    get_num_cells::bench,
    get_pentagons::bench,
    get_res0_cells::bench,
    get_resolution::bench,
    h3_to_string::bench,
    is_pentagon::bench_hexagons,
    is_pentagon::bench_pentagons,
    is_res_class3::bench,
    is_valid_cell::bench,
    is_valid_directed_edge::bench,
    is_valid_vertex::bench,
    latlng_to_cell::bench,
    local_ij_to_cell::bench,
    max_face_count::bench,
    max_grid_disk_size::bench,
    origin_to_directed_edges::bench,
    pentagon_count::bench,
    rads_to_degs::bench,
    res0_cell_count::bench,
    string_to_h3::bench_cell,
    string_to_h3::bench_edge,
    string_to_h3::bench_vertex,
    uncompact_cells::bench,
    uncompact_cells_size::bench,
    vertex_to_latlng::bench,
);

#[cfg(not(feature = "geo"))]
criterion_main!(benches);

#[cfg(feature = "geo")]
criterion_group!(
    benches_geom,
    h3_set_lo_linked_geo::bench_full,
    h3_set_lo_linked_geo::bench_holes,
    h3_set_lo_linked_geo::bench_rings,
    max_polygon_to_cells_size::bench,
    polygon_to_cells::bench_full,
    polygon_to_cells::bench_transmeridian,
    polygon_to_cells::bench_polyfill_mode,
);

#[cfg(feature = "geo")]
criterion_main!(benches, benches_geom);
