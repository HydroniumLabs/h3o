# Architecture

This document describes the high-level architecture of `h3o`.
If you want to familiarize yourself with the code base, you are just in the
right place!

## Code Map

### Index types (src/index)

This is were the H3 indexes type lives.

They are the main entry points to the H3 API, with `CellIndex` being the most
prominent one (as demonstrated by its large public API).

### Error types (src/errors)

There is a balance to find between the "single crate-wide catch-all error type"
approach and the "per-function error type" one.
Here we're lean toward the latter since it gives a better experience to the
consumer of the library when matching/handling error from a function.

### Coordinate systems (src/coord)

This module implements various coordinate systems that are used as bridge
between public types (e.g. converting a `CellIndex` into a `LatLng` goes through
the `IJK` coordinate system).

Except for the two public ones (`LatLng` and `LocalIJ`) they can be treated as
an implementation detail used for internal computations.

### Grid API (src/grid)

This module implements algorithms related to grid traversal.

They provides way to find cell indexes in the vicinity of an origin cell index,
and to determine how to traverse the grid from one cell index to another.

This module is not public: its features are exposed through the `CellIndex`
type.

### Geometry API (src/geom)

This module is feature-gated by the `geom` feature.

It provides two traits:
- `ToGeo`, to convert H3 objects to geometries (e.g. convert a set of H3 cell
  indexes into a multi-polygon).
- `ToCells`, to convert geometries into H3 objects (e.g. compute the set of H3
  cell indexes that cover a given polygon).

It also contains wrapper types around `RustGeo` types (to enforce H3-specific
constraints) and `From/Into` implementations to work with GeoJSON.

## Cross-Cutting Concerns

### Testing

Unit tests are used to test internal functions that are complex enough.
They reside in their own file, to limit the amount of recompilation when working
on the tests.

Integration tests are used to test the public API.
They are divided into two tests suite:
- `api` contains freestanding tests
- `h3` contains tests that are run against the H3 reference implementation
  (differential testing).

doctest are disabled because they are way slower than regular tests (one binary
per example, and since we have one example per public function...)

### Fuzzing

Fuzz targets for `cargo-fuzz` can be found under `fuzz`.

Targets are grouped by input type (e.g. every function taking a single
`CellIndex` as input will be in `cell_index.rs`) in order to fuzz as many
function as possible at each round of fuzzing.

Not as comprehensive as the test suite for now.

### Benchmarking

A comprehensive benchmark suite lives under `benches`.

Each public H3 function from the reference implementation is benched (through
`h3ron-sys`) against its h3o equivalent.
