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
