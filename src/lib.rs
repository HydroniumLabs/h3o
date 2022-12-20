//! The `h3o` library implements the H3 geospatial indexing system.
//!
//! H3 is a geospatial indexing system using a hexagonal grid that can be
//! (approximately) subdivided into finer and finer hexagonal grids, combining
//! the benefits of a hexagonal grid with S2's hierarchical subdivisions.

// Lints {{{

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    rustdoc::all,
    rustdoc::missing_crate_level_docs,
    missing_docs,
    unsafe_code,
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    variant_size_differences,
    warnings,
    clippy::all,
    clippy::cargo,
    clippy::pedantic,
    clippy::allow_attributes_without_reason,
    clippy::as_underscore,
    clippy::branches_sharing_code,
    clippy::clone_on_ref_ptr,
    clippy::cognitive_complexity,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::decimal_literal_representation,
    clippy::default_union_representation,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_drop,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::exhaustive_enums,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::future_not_send,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::imprecise_flops,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_include_file,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::missing_const_for_fn,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_inherent_impl,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::needless_collect,
    clippy::non_send_fields_in_send_ty,
    clippy::nonstandard_macro_braces,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::panic,
    clippy::path_buf_push_overwrite,
    clippy::pattern_type_mismatch,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_pub_crate,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::significant_drop_in_scrutinee,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::todo,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::try_err,
    clippy::type_repetition_in_bounds,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_file_reads
)]
#![allow(
    // The 90’s called and wanted their charset back.
    clippy::non_ascii_literal,
    // "It requires the user to type the module name twice."
    // => not true here since internal modules are hidden from the users.
    clippy::module_name_repetitions,
    // Usually yes, but not really applicable for most literals in this crate.
    clippy::unreadable_literal,
)]

// }}}

mod base_cell;
mod boundary;
mod coord;
mod direction;
pub mod error;
mod face;
#[cfg(feature = "geo")]
pub mod geom;
mod grid;
mod index;
mod resolution;

pub use base_cell::BaseCell;
pub use boundary::Boundary;
pub use coord::{LatLng, LocalIJ};
pub use direction::Direction;
pub use face::{Face, FaceSet};
pub use index::{
    CellIndex, DirectedEdgeIndex, Edge, IndexMode, Vertex, VertexIndex,
};
pub use resolution::Resolution;

use resolution::ExtendedResolution;

// -----------------------------------------------------------------------------

/// Size, in bits, of a direction (range [0; 6].
const DIRECTION_BITSIZE: usize = 3;

/// An icosahedron has 20 faces.
const NUM_ICOSA_FACES: usize = 20;
// The number of vertices in a hexagon.
const NUM_HEX_VERTS: u8 = 6;
// The number of vertices in a pentagon.
const NUM_PENT_VERTS: u8 = 5;

/// Direction: counterclockwise
const CCW: bool = true;
/// Direction: clockwise
const CW: bool = false;

/// Earth radius in kilometers using WGS84 authalic radius.
const EARTH_RADIUS_KM: f64 = 6371.007180918475_f64;

/// Number of pentagon per resolution.
const NUM_PENTAGONS: u8 = 12;

/// Default cell index (resolution 0, base cell 0).
const DEFAULT_CELL_INDEX: u64 = 0x0800_1fff_ffff_ffff;

// 2π
const TWO_PI: f64 = 2. * std::f64::consts::PI;

// -----------------------------------------------------------------------------

/// Maximum number of indices produced by the grid disk algorithm with the given
/// `k`.
///
/// # Example
///
/// ```
/// let count = h3o::max_grid_disk_size(3);
/// ```
#[must_use]
pub const fn max_grid_disk_size(k: u32) -> u64 {
    // k value which will encompass all cells at resolution 15.
    // This is the largest possible k in the H3 grid system.
    const K_MAX: u32 = 13_780_510;

    if k >= K_MAX {
        return Resolution::Fifteen.cell_count();
    }

    let k = k as u64;
    // Formula source and proof: https://oeis.org/A003215
    3 * k * (k + 1) + 1
}
