# phantom-coerce

Safe, zero-cost coercion between types differing only in `PhantomData` parameters.

## Overview

When working with phantom types (types that use `PhantomData<T>` for compile-time type tracking), you often want to coerce from specific types to more generic ones. This crate provides a `#[derive(Coerce)]` macro that generates safe coercion methods with compile-time guarantees.

## Why?

`PhantomData<T>` is zero-sized and has no runtime representation, making coercions between types that differ only in their phantom parameters safe. However, writing these coercions manually is tedious and error-prone. This crate automates the process while maintaining strong safety guarantees.

## Design Philosophy

This library is designed to complement, not replace, strongly-typed marker patterns like typestate. When you have a typestate or similar pattern with specific state transitions (e.g., `Draft -> Review -> Published`), you should continue using those strongly-typed transitions for your core logic.

**This library solves a specific orthogonal problem**: sometimes you need "generic" marker types that are reachable from any specific state. For example:

- Storing heterogeneous collections of values with different marker types
- Passing values across API boundaries that don't care about specific states
- Implementing generic handlers that work regardless of the specific marker type
- Converting to a "known unknown" state when precise type information is no longer needed

By requiring explicit `#[coerce(...)]` annotations, the library ensures that:
- You explicitly define which generic types are safe and meaningful for your domain
- Coercion points are auditable and intentional, not automatic
- Your strongly-typed transitions remain the primary way to change states
- Generic coercions serve as explicit "escape hatches" when type erasure to a more general type is appropriate

**Example**: In a request validation pipeline, you might have strongly-typed states like `Unvalidated -> HeadersValidated -> FullyValidated`. But you might also want an `AnyStatus` generic marker for when you need to store mixed validation states in a collection or pass requests through generic middleware. This library helps you define those specific "any state → generic state" coercions safely.

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
struct UnknownBase;  // Generic (subsumes Absolute and Relative)

struct File;
struct Directory;
struct UnknownType;  // Generic (subsumes File and Directory)

#[derive(Coerce)]
#[coerce(borrowed = "TypedPath<UnknownBase, File>")]
#[coerce(borrowed = "TypedPath<Absolute, UnknownType>")]
#[coerce(borrowed = "TypedPath<UnknownBase, UnknownType>")]
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

    // Coerce to more generic phantom types
    let coerced: &TypedPath<UnknownBase, File> = path.coerce();
}
```

#### Turbofish Syntax

All coercion methods support turbofish syntax for explicit type specification:

```rust
let path = TypedPath::<Absolute, File> { /* ... */ };

// With type inference:
let coerced: &TypedPath<UnknownBase, File> = path.coerce();

// With turbofish (no left-hand type annotation needed):
let coerced = path.coerce::<TypedPath<UnknownBase, File>>();
```

Similarly, `into_coerced::<T>()` and `to_coerced::<T>()` support turbofish for owned and cloned coercions.

#### Optional `AsRef` Integration

Add the `asref` marker to also generate `AsRef` implementations:

```rust
#[derive(Coerce)]
#[coerce(borrowed = "TypedPath<UnknownBase, File>", asref)]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

fn takes_asref(path: &impl AsRef<TypedPath<UnknownBase, File>>) {
    let p: &TypedPath<UnknownBase, File> = path.as_ref();
    // Use p...
}

fn main() {
    let path = TypedPath::<Absolute, File> { /* ... */ };
    takes_asref(&path); // Works: Absolute coerces to UnknownBase
}
```

### Owned Coercion

Owned coercions allow you to convert `T` to `U`, consuming the original value:

```rust
use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Validated;
struct Unvalidated;
struct AnyStatus;  // Generic (subsumes Validated and Unvalidated)

#[derive(Coerce)]
#[coerce(owned = "Request<AnyStatus>")]
struct Request<Status> {
    marker: PhantomData<Status>,
    url: String,
    headers: Vec<(String, String)>,
}

fn main() {
    let validated_req = Request::<Validated> {
        marker: PhantomData,
        url: "https://api.example.com/users".to_string(),
        headers: vec![("Authorization".to_string(), "Bearer token".to_string())],
    };

    // Convert to more generic phantom type (owned, consumes original)
    let any_req: Request<AnyStatus> = validated_req.into_coerced();
}
```

### Cloned Coercion

Cloned coercions allow you to convert `&T` to `U` by cloning, requiring the source type to implement `Clone`:

```rust
use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Json;
struct Xml;
struct AnyFormat;  // Generic (subsumes Json and Xml)

#[derive(Coerce, Clone)]
#[coerce(cloned = "Message<AnyFormat>")]
struct Message<Format> {
    marker: PhantomData<Format>,
    content: String,
    metadata: Vec<String>,
}

fn main() {
    let json_msg = Message::<Json> {
        marker: PhantomData,
        content: r#"{"status": "ok"}"#.to_string(),
        metadata: vec!["v1".to_string()],
    };

    // Clone and coerce to more generic phantom type (source remains usable)
    let any_msg: Message<AnyFormat> = json_msg.to_coerced();

    // Original is still available
    println!("{}", json_msg.content);
}
```

## How It Works

The `#[derive(Coerce)]` macro generates:

1. **For borrowed coercions**: A trait `CoerceRef{TypeName}<Output>` with a `coerce(&self) -> &Output` method
2. **For owned coercions**: A trait `CoerceOwned{TypeName}<Output>` with an `into_coerced(self) -> Output` method
3. **For cloned coercions**: A trait `CoerceCloned{TypeName}<Output>` with a `to_coerced(&self) -> Output` method
4. Implementations for each target type specified in attributes

### Generated Code (Borrowed)

For borrowed coercions:

```rust
trait CoerceRefTypedPath<Output: ?Sized> {
    fn coerce(&self) -> &Output;
}

impl<Base, Type> CoerceRefTypedPath<TypedPath<UnknownBase, File>> for TypedPath<Base, Type> {
    fn coerce(&self) -> &TypedPath<UnknownBase, File> {
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
trait CoerceOwnedRequest<Output> {
    fn into_coerced(self) -> Output;
}

impl<Status> CoerceOwnedRequest<Request<AnyStatus>> for Request<Status> {
    fn into_coerced(self) -> Request<AnyStatus> {
        // Compile-time safety guard: ensure all fields are accounted for
        let Request { marker: _, url: _, headers: _ } = &self;

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

impl<Format> CoerceClonedMessage<Message<AnyFormat>> for Message<Format>
where
    Message<Format>: Clone,
{
    fn to_coerced(&self) -> Message<AnyFormat> {
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
impl<Base, Type> AsRef<TypedPath<UnknownBase, File>> for TypedPath<Base, Type> {
    fn as_ref(&self) -> &TypedPath<UnknownBase, File> {
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
