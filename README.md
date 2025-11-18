# phantom-coerce

Safe, zero-cost coercion between types differing only in `PhantomData` parameters.

## Overview

When working with phantom types (types that use `PhantomData<T>` for compile-time type tracking), you often want to coerce from specific types to more generic ones. This crate provides a `#[derive(Coerce)]` macro that generates safe coercion methods with compile-time guarantees.

## Why?

`PhantomData<T>` is zero-sized and has no runtime representation, making coercions between types that differ only in their phantom parameters safe. However, writing these coercions manually is tedious and error-prone. This crate automates the process while maintaining strong safety guarantees.

## Features

- **Type-safe**: Compile-time checks ensure only `PhantomData` fields can vary
- **Zero-cost**: All coercions are compile-time only with no runtime overhead
- **Ergonomic**: Simple attribute syntax for declaring valid coercions
- **Safe**: Generated code includes exhaustive field destructuring and type annotations
- **Flexible**: Supports both borrowed (`&T -> &U`) and owned (`T -> U`) coercions

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
phantom-coerce = "0.1"
```

### Borrowed Coercion

Borrowed coercions allow you to coerce `&T` to `&U`:

```rust
use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Absolute;
struct Relative;
struct File;
struct Directory;

#[derive(Coerce)]
#[coerce(borrowed = "TypedPath<Relative, File>")]
#[coerce(borrowed = "TypedPath<Absolute, Directory>")]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

fn main() {
    let path = TypedPath::<Absolute, File> {
        base: PhantomData,
        ty: PhantomData,
        path: "/home/user/file.txt".to_string(),
    };

    // Coerce to different phantom types (borrowed)
    let coerced: &TypedPath<Relative, File> = path.coerce();
}
```

### Owned Coercion

Owned coercions allow you to convert `T` to `U`, consuming the original value:

```rust
use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Initial;
struct Final;

#[derive(Coerce)]
#[coerce(owned = "State<Final>")]
struct State<S> {
    marker: PhantomData<S>,
    data: Vec<String>,
}

fn main() {
    let state = State::<Initial> {
        marker: PhantomData,
        data: vec!["item1".to_string(), "item2".to_string()],
    };

    // Convert to different phantom type (owned, consumes original)
    let final_state: State<Final> = state.into_coerced();
}
```

## How It Works

The `#[derive(Coerce)]` macro generates:

1. **For borrowed coercions**: A trait `Coerce{TypeName}<Output>` with a `coerce(&self) -> &Output` method
2. **For owned coercions**: A trait `CoerceOwned{TypeName}<Output>` with an `into_coerced(self) -> Output` method
3. Implementations for each target type specified in attributes

### Generated Code (Borrowed)

For borrowed coercions:

```rust
trait CoerceTypedPath<Output: ?Sized> {
    fn coerce(&self) -> &Output;
}

impl<Base, Type> CoerceTypedPath<TypedPath<Relative, File>> for TypedPath<Base, Type> {
    fn coerce(&self) -> &TypedPath<Relative, File> {
        // Compile-time safety guards: ensure all fields are accounted for
        let TypedPath { base: _, ty: _, path: _ } = self;
        let _: &PhantomData<Base> = &self.base;
        let _: &PhantomData<Type> = &self.ty;
        let _: &String = &self.path;

        // SAFETY: Types differ only in PhantomData type parameters.
        // The destructuring pattern and type annotations above ensure this at compile time.
        unsafe { std::mem::transmute(self) }
    }
}
```

### Generated Code (Owned)

For owned coercions:

```rust
trait CoerceOwnedState<Output> {
    fn into_coerced(self) -> Output;
}

impl<S> CoerceOwnedState<State<Final>> for State<S> {
    fn into_coerced(self) -> State<Final> {
        // Compile-time safety guard: ensure all fields are accounted for
        let State { marker: _, data: _ } = &self;

        // SAFETY: Types differ only in PhantomData type parameters.
        // The destructuring pattern above ensures this at compile time.
        unsafe { std::mem::transmute(self) }
    }
}
```

## Safety Guarantees

The macro includes multiple compile-time safety checks:

1. **Field exhaustiveness**: Destructuring ensures all fields are accounted for. Adding or removing fields breaks compilation.
2. **Type stability** (borrowed only): Type annotations ensure field types haven't changed.
3. **PhantomData-only changes**: Only types differing in `PhantomData` parameters can be coerced.
4. **Documented safety**: Generated `SAFETY` comments explain why each transmute is sound.

## Examples

See the [`examples/`](examples/) directory:

- [`typed_path.rs`](examples/typed_path.rs) - Complete example with typed filesystem paths

Run with:

```bash
cargo run --example typed_path
```

## Limitations

- Requires named struct fields
- Target types must be specified as literal strings in attributes

## Future Enhancements

- Support for `#[coerce(mutable = "...")]` for mutable references
- Attribute to customize generated trait name
- Tuple struct support
- Better error messages with span information
- Optional `AsRef`/`Into` integration via attribute

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Work

- **`ghost`**: Variance control for phantom types, but no coercion
- **`CoercePointee`**: For unsizing coercion (`T` → `dyn Trait`), not phantom marker changes
- **`derive-where`**: For deriving standard traits with phantom types, not coercion

This crate fills a gap: safe, ergonomic coercion for types differing only in phantom parameters.
