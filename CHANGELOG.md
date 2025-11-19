# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-18

### Added

- Initial release of `phantom-coerce` and `phantom-coerce-derive`
- `#[derive(Coerce)]` macro for safe phantom type coercion
- Three coercion modes with explicit `from/to` syntax:
  - **Borrowed coercion** (`#[coerce(borrowed_from = "Source", borrowed_to = "Target")]`): `&T -> &U` via `.coerce()` method
  - **Owned coercion** (`#[coerce(owned_from = "Source", owned_to = "Target")]`): `T -> U` via `.into_coerced()` method
  - **Cloned coercion** (`#[coerce(cloned_from = "Source", cloned_to = "Target")]`): `&T -> U` via `.to_coerced()` method (requires `Clone`)
- Pipe syntax (`|`) for specifying multiple source type alternatives: `Type<A | B, X | Y>`
- Type hole syntax (`_`) in type parameters to preserve specific parameters during coercion
- Optional `AsRef` implementation generation for borrowed coercions via `asref` marker
- Turbofish syntax support for all coercion methods
- Compile-time safety guarantees:
  - Field exhaustiveness checking via destructuring patterns
  - Type stability verification (borrowed only)
  - PhantomData-only change validation
- Comprehensive test suite:
  - 11 integration tests covering all coercion modes
  - Compile-fail UI tests ensuring proper error messages
- Examples demonstrating all coercion modes:
  - `typed_path.rs` - Borrowed coercion with filesystem paths
  - `request_validation.rs` - Owned coercion for validation pipelines
  - `message_formats.rs` - Cloned coercion for message handling
  - `heterogeneous_collections.rs` - Storing mixed types in collections
- CI workflow with cargo-nextest, clippy, and formatting checks
- Documentation with design philosophy and usage examples

### Technical Details

- Minimum Supported Rust Version (MSRV): 1.85
- Edition: 2024
- Zero-cost abstractions using `std::mem::transmute` with compile-time safety
- Works with named struct fields only
- No runtime overhead

[0.1.0]: https://github.com/attunehq/phantom-coerce/releases/tag/v0.1.0
