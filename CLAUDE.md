# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`phantom-coerce` is a Rust library providing safe, zero-cost coercion between types differing only in `PhantomData` parameters. It's a workspace with two crates:

- `phantom-coerce`: User-facing library that re-exports the derive macro
- `phantom-coerce-derive`: Procedural macro implementation

## Core Design Philosophy

**Critical**: This library is NOT for general type coercion or state machine transitions. It specifically solves the problem of coercing from **specific marker types to more generic marker types** (e.g., `TypedPath<Absolute, File>` → `TypedPath<UnknownBase, File>`).

The library complements, not replaces, strongly-typed patterns like typestate. Users should continue using strongly-typed transitions for core logic. This library provides explicit "escape hatches" for when type erasure to a more general type is appropriate (heterogeneous collections, API boundaries, generic handlers).

Examples should always demonstrate:
- Coercing to MORE GENERIC types (never arbitrary state transitions)
- Realistic use cases (typed paths, validation states, format types)
- Never use state machine patterns like `Initial -> Final` or `Draft -> Published`

## Development Commands

### Building
```bash
# Build workspace
cargo build

# Build with documentation
cargo doc --no-deps
```

### Testing
```bash
# Run all tests (integration + compile-fail UI tests)
cargo test --all

# Run only doc tests
cargo test --doc

# Run a specific test
cargo test --test integration_test test_single_param_coercion

# Run compile-fail tests specifically
cargo test --test compile_fail
```

### Examples
```bash
cargo run --example typed_path
```

## Architecture

### Code Generation Pattern

The derive macro generates three types of traits per struct:

1. **`CoerceRef{TypeName}<Output>`** for borrowed coercions (`&T -> &U`)
   - Method: `coerce(&self) -> &Output`
   - Optional: `AsRef<Output>` impl when `asref` marker present

2. **`CoerceOwned{TypeName}<Output>`** for owned coercions (`T -> U`)
   - Method: `into_coerced(self) -> Output`

3. **`CoerceCloned{TypeName}<Output>`** for cloned coercions (`&T -> U`)
   - Method: `to_coerced(&self) -> Output`
   - Requires `Clone` bound on source type

Each trait gets:
- Trait definition with generic `Output` parameter
- Impl blocks for each target type specified in `#[coerce(...)]` attributes
- Inherent method with turbofish support (e.g., `.coerce::<Target>()`)

### Safety Model

All coercions use `unsafe { std::mem::transmute(...) }` but are made safe through compile-time checks:

1. **Field exhaustiveness**: Destructuring pattern ensures all fields accounted for
2. **Type stability** (borrowed only): Type annotations verify field types unchanged
3. **PhantomData detection**: Only fields with `PhantomData<T>` can vary between source/target

The generated code includes `SAFETY` comments explaining why each transmute is sound.

### Key Implementation Files

- `phantom-coerce-derive/src/lib.rs`: Single-file proc macro implementation
  - `parse_coerce_attr()`: Parses `#[coerce(...)]` attributes
  - `generate_borrowed_impl()`, `generate_owned_impl()`, `generate_cloned_impl()`: Code generators
  - `is_phantom_data()`: Identifies PhantomData fields

- `phantom-coerce/tests/ui/`: Compile-fail tests using `trybuild`
  - Ensures correct error messages for misuse

## Testing Strategy

### Integration Tests
Located in `tests/integration_test.rs`:
- 11 tests covering all coercion modes
- Tests single-param, multi-param, and turbofish syntax
- Verifies AsRef integration

### Compile-Fail Tests
Located in `tests/ui/`:
- `asref_on_non_borrowed.rs`: AsRef only works with borrowed coercions
- `missing_clone.rs`: Cloned coercion requires Clone
- `no_coerce_attrs.rs`: At least one coerce attribute required
- `on_enum.rs`: Derive only works on structs

Run with `cargo test --test compile_fail` - uses `trybuild` to verify expected compilation errors.

## Publishing Notes

Both crates must be published to crates.io in order:
1. `cargo publish -p phantom-coerce-derive` (the proc macro)
2. `cargo publish -p phantom-coerce` (depends on derive crate)

The crates are configured with proper metadata for crates.io publication.
