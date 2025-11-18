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

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
phantom-coerce = "0.1"
```

### Basic Example

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

    // Coerce to different phantom types
    let coerced: &TypedPath<Relative, File> = path.coerce();
}
```

## How It Works

The `#[derive(Coerce)]` macro generates:

1. A trait `Coerce{TypeName}<Output>` with a `coerce(&self) -> &Output` method
2. Implementations for each `#[coerce(borrowed = "...")]` target

### Generated Code

For the example above, the macro generates:

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

## Safety Guarantees

The macro includes multiple compile-time safety checks:

1. **Field exhaustiveness**: Destructuring ensures all fields are accounted for. Adding or removing fields breaks compilation.
2. **Type stability**: Type annotations ensure field types haven't changed.
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

- Currently only supports borrowed coercions (`&T -> &U`)
- Requires named struct fields
- Target types must be specified as literal strings in attributes

## Future Enhancements

- Support for `#[coerce(owned = "...")]` for `Into` implementations
- Support for `#[coerce(mutable = "...")]` for mutable references
- Tuple struct support
- Better error messages with span information

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
