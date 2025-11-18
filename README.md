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

#### Optional `AsRef` Integration

Add the `asref` marker to also generate `AsRef` implementations:

```rust
#[derive(Coerce)]
#[coerce(borrowed = "TypedPath<Relative, File>", asref)]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

fn takes_asref(path: &impl AsRef<TypedPath<Relative, File>>) {
    let p: &TypedPath<Relative, File> = path.as_ref();
    // Use p...
}

fn main() {
    let path = TypedPath::<Absolute, File> { /* ... */ };
    takes_asref(&path); // Works thanks to AsRef impl
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

### Cloned Coercion

Cloned coercions allow you to convert `&T` to `U` by cloning, requiring the source type to implement `Clone`:

```rust
use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Source;
struct Target;

#[derive(Coerce, Clone)]
#[coerce(cloned = "Message<Target>")]
struct Message<M> {
    marker: PhantomData<M>,
    content: String,
    metadata: Vec<String>,
}

fn main() {
    let msg = Message::<Source> {
        marker: PhantomData,
        content: "Hello".to_string(),
        metadata: vec!["tag1".to_string()],
    };

    // Clone and coerce to different phantom type (source remains usable)
    let coerced: Message<Target> = msg.to_coerced();

    // Original is still available
    println!("{}", msg.content);
}
```

## How It Works

The `#[derive(Coerce)]` macro generates:

1. **For borrowed coercions**: A trait `Coerce{TypeName}<Output>` with a `coerce(&self) -> &Output` method
2. **For owned coercions**: A trait `CoerceOwned{TypeName}<Output>` with an `into_coerced(self) -> Output` method
3. **For cloned coercions**: A trait `CoerceCloned{TypeName}<Output>` with a `to_coerced(&self) -> Output` method
4. Implementations for each target type specified in attributes

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

### Generated Code (Cloned)

For cloned coercions:

```rust
trait CoerceClonedMessage<Output> {
    fn to_coerced(&self) -> Output;
}

impl<M> CoerceClonedMessage<Message<Target>> for Message<M>
where
    Message<M>: Clone,
{
    fn to_coerced(&self) -> Message<Target> {
        // Compile-time safety guard: ensure all fields are accounted for
        let Message { marker: _, content: _, metadata: _ } = self;

        // SAFETY: Types differ only in PhantomData type parameters.
        // The destructuring pattern above ensures this at compile time.
        // The source type is cloned and then transmuted.
        unsafe { std::mem::transmute(self.clone()) }
    }
}
```

### Generated Code (AsRef)

When using the `asref` marker with borrowed coercions:

```rust
impl<Base, Type> AsRef<TypedPath<Relative, File>> for TypedPath<Base, Type> {
    fn as_ref(&self) -> &TypedPath<Relative, File> {
        self.coerce()
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

- [`typed_path.rs`](examples/typed_path.rs) - Complete example with typed filesystem paths and AsRef integration

Run with:

```bash
cargo run --example typed_path
```

## Testing

The crate includes comprehensive tests:

- **Integration tests** (`tests/integration_test.rs`): 11 tests covering borrowed, owned, and cloned coercions
- **Compile-fail tests** (`tests/ui/`): Demonstrates compile-time safety guarantees:
  - `asref_on_non_borrowed.rs`: AsRef marker only works with borrowed coercions
  - `missing_clone.rs`: Cloned coercion requires Clone trait
  - `no_coerce_attrs.rs`: At least one coerce attribute required
  - `on_enum.rs`: Derive only works on structs

Run tests with:

```bash
cargo test --all
```

## Limitations

- Requires named struct fields
- Target types must be specified as literal strings in attributes
- Cannot generate `Into` impls due to conflicting blanket impl in `core` (use trait methods directly instead)

## Future Enhancements

- Support for `#[coerce(mutable = "...")]` for mutable references
- Attribute to customize generated trait name
- Tuple struct support
- Better error messages with span information

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).

## Related Work

- **`ghost`**: Variance control for phantom types, but no coercion
- **`CoercePointee`**: For unsizing coercion (`T` → `dyn Trait`), not phantom marker changes
- **`derive-where`**: For deriving standard traits with phantom types, not coercion

This crate fills a gap: safe, ergonomic coercion for types differing only in phantom parameters.
