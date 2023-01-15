# h3o

[![Crates.io](https://img.shields.io/crates/v/h3o.svg)](https://crates.io/crates/h3o)
[![Docs.rs](https://docs.rs/h3o/badge.svg)](https://docs.rs/h3o)
[![CI Status](https://github.com/HydroniumLabs/h3o/actions/workflows/ci.yml/badge.svg)](https://github.com/HydroniumLabs/h3o/actions)
[![Coverage](https://img.shields.io/codecov/c/github/HydroniumLabs/h3o)](https://app.codecov.io/gh/HydroniumLabs/h3o)
[![License](https://img.shields.io/badge/license-BSD-green)](https://opensource.org/licenses/BSD-3-Clause)

[Rust](https://rustlang.org) implementation of the [H3](https://h3geo.org)
geospatial indexing system.

## Design

This is not a binding of the reference implementation, but a reimplementation
from scratch.

The goals are:
- To be safer/harder to misuse by leveraging the strong typing of Rust.
- To be 100% Rust (no C deps): painless compilation to WASM, easier LTO, …
- To be as fast (or even faster when possible) than the reference library.

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install h3o`

## Usage

```rust
use h3o::{LatLng, Resolution};

let coord = LatLng::new(37.769377, -122.388903).expect("valid coord");
let cell = coord.to_cell(Resolution::Nine);
```

## Why this name?

Rust is an iron oxide.
A Rust version of H3 is an H3 oxide, in other word $H_3O$ (a.k.a hydronium).
Chemically speaking this is wrong ( $H_3O$ is produced by protonation of
$H_2O$, not oxidation of $H_3$), but ¯\\_(ツ)_/¯

## License

[BSD 3-Clause](./LICENSE)
