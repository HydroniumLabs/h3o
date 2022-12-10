use criterion::{criterion_group, criterion_main};

mod cell_to_center_child;
mod cell_to_children_size;
mod cell_to_parent;
mod constants;
mod degs_to_rads;
mod get_base_cell_number;
mod get_directed_edge_origin;
mod get_hexagon_area_avg;
mod get_hexagon_edge_length_avg;
mod get_num_cells;
mod get_pentagons;
mod get_res0_cells;
mod get_resolution;
mod h3_to_string;
mod is_pentagon;
mod is_res_class3;
mod is_valid_cell;
mod is_valid_directed_edge;
mod pentagon_count;
mod rads_to_degs;
mod res0_cell_count;
mod string_to_h3;

criterion_group!(
    benches,
    cell_to_center_child::bench,
    cell_to_children_size::bench,
    cell_to_parent::bench,
    degs_to_rads::bench,
    get_base_cell_number::bench,
    get_directed_edge_origin::bench,
    get_hexagon_area_avg::bench_km2,
    get_hexagon_area_avg::bench_m2,
    get_hexagon_edge_length_avg::bench_km,
    get_hexagon_edge_length_avg::bench_m,
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
    pentagon_count::bench,
    rads_to_degs::bench,
    res0_cell_count::bench,
    string_to_h3::bench_cell,
    string_to_h3::bench_edge,
);

criterion_main!(benches);
