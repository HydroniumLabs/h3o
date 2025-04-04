# Changelog

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Possible sections are:

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.8.0] - 2025-03-31

### Added

- `h3o::geom::Solvent` to compute the shape of a set of H3 cells.
    - opt-in duplicate detection
    - can works with heterogeneous set of cells

### Changed

- `CellIndex::compact` now works in-place instead of using iterators.

### Removed

- `h3o::geom::dissolve` is removed.
    - you can use `SolventBuilder::new().build().dissolve(cells)` instead.

## [0.7.1] - 2024-12-24

### Fixed

- Fix a bug in the `Tiler` when dealing with transmeridian cells.

## [0.7.0] - 2024-11-15

### Added

- `h3o::geom::Plotter` to compute the cells along lines.
- `h3o::geom::Tiler` to compute the cell coverage of plane figures.

### Changed

- small performance enhancement for aarch64, should not affect other platforms
- error types now derive `Error`, even in no-std mode.

### Removed

- remove the geometry types wrapper (you can now use `geo` types directly).
- remove the `ToCells` trait, now you can use:
    - `h3o::LatLng` for `Point` & `MultiPoint`.
    - `h3o::geom::Plotter` for `Line`, `LineString` and `MultiLineString`.
    - `h3o::geom::Tiler` for `Polygon`, `MultiPolygon`, `Rect` and `Triangle`.
- `Geometry` and `GeometryCollection` are not directly supported
- remove the support for `geoJSON`.
- remove the `ToGeo` trait, now you can use:
    - `From` trait for `CellIndex`, `DirectedEdgeIndex` and `VertexIndex`
    ` `geom::dissolve` for a set of cell indexes.

## [0.6.4] - 2024-05-10

### Fixed

- fix `to_geom` when dealing with shapes crossing the prime meridian.

## [0.6.3] - 2024-05-09

### Fixed

- fix `to_cells` when dealing with transmeridian cells

## [0.6.2] - 2024-03-31

### Changed

- bump dependencies

## [0.6.1] - 2024-03-25

### Fixed

- fix a bug in the antimeridian heuristic of `to_cells`

## [0.6.0] - 2024-02-23

### Removed

- remove public constants VERSION_MAJOR, VERSION_MINOR and VERSION_PATCH

## [0.5.2] - 2024-02-22

### Added

- add `no_std` support (`std` is still enabled by default though)
- add `typed_floats` support (disabled by default)

## [0.5.1] - 2024-01-27

### Fixed

- fix `to_cells` when dealing with transmeridian cells

## [0.5.0] - 2024-01-15

### Added

- new containment mode, `Covers`, to handle small geometries within a cell.
- clarification on the `IntersectsBoundary` mode.

### Removed

- `LocalIJ::anchor`, `LocalIJ::i` and `LocalIJ::j` no longer exists (fields are
  public now).

### Changed

- `CoordIJ` is now public
- `LocalIJ::new_unchecked` is replaced by `LocalIJ::new`

## [0.4.0] - 2023-08-09

### Changed

- `to_cells` and `max_cells_count` now takes a `PolyfillConfig` as input.
- `from_radians` constructors for geometry now take ownership of the input.

## [0.3.5] - 2023-08-05

### Fixed

- update precomputed average edge lengths (they were underestimated).
- HUGE improvements, both in term of speed and memory usage, for
  `Polygon::to_cells`.
- HUGE improvements, in term of memory usage, for `to_geom` on cells.

## [0.3.4] - 2023-07-28

### Fixed

- fix `Point` to `LatLng` conversion

## [0.3.3] - 2023-07-21

### Fixed

- compilation issue on Rust 1.71+ due to an outdated version of `geo`

## [0.3.2] - 2023-05-30

### Fixed

- fix the `to_geom` implementation

### Added

- `CellIndex::succ`
- `CellIndex::pred`
- `CellIndex::first`
- `CellIndex::last`

### Changed

- `Debug` impl for `LatLng` now prints both radians and degrees.

## [0.3.1] - 2023-03-17

### Fixed

- fix `LatLng` ordering
- fix compilation to WASM by using compile-time RNG for ahash

## [0.3.0] - 2023-01-31

### Added

- `Resolution::area_rads2`
- `Resolution::edge_length_rads`
- `CellIndex::child_position`
- `CellIndex::child_at`

### Changed

- geometry functions relies on radians only, making the planet-independant

## [0.2.0] - 2023-01-15

### Added

- `LatLng::from_radians` (replace the old `LatLng::new`)
- `LatLng::lat_radians` (replace the old `LatLng::lat`)
- `LatLng::lng_radians` (replace the old `LatLng::lng`)

### Changed

- Conversion from/to `geo` types now assumes degrees (instead of radians)
- `LatLng::new` now expects degrees as input.
- `LatLng::lat` and `LatLng::lng` now return degrees.

### Removed

- `LatLng::from_degrees` (use the new `LatLng::new` instead)
- `LatLng::lat_degrees` (use the new `LatLng::lat` instead)
- `LatLng::lng_degrees` (use the new `LatLng::lng` instead)

### Fixed

- fix link to CHANGELOG in CONTRIBUTING
- fix various typos
- escape backslash in README
- fix LocalIJ doc

## [0.1.0] - 2023-01-09

- initial release, full coverage of the H3 API
