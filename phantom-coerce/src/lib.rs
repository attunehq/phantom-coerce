//! Safe, zero-cost coercion between types differing only in PhantomData parameters.
//!
//! This crate provides a `#[derive(Coerce)]` macro that generates safe coercion methods
//! for types that differ only in their `PhantomData` type parameters.
//!
//! # Borrowed Coercion
//!
//! Use `#[coerce(borrowed = "...")]` to generate borrowed coercions (`&T -> &U`):
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # struct Absolute;
//! # struct UnknownBase;  // Generic (subsumes Absolute)
//! # struct File;
//! # struct UnknownType;  // Generic (subsumes File)
//! #
//! #[derive(Coerce)]
//! #[coerce(borrowed = "TypedPath<UnknownBase, File>")]
//! #[coerce(borrowed = "TypedPath<Absolute, UnknownType>")]
//! struct TypedPath<Base, Type> {
//!     base: PhantomData<Base>,
//!     ty: PhantomData<Type>,
//!     path: std::path::PathBuf,
//! }
//!
//! # fn main() {
//! let path = TypedPath::<Absolute, File> {
//!     base: PhantomData,
//!     ty: PhantomData,
//!     path: std::path::PathBuf::from("/test"),
//! };
//! // Coerce to more generic type (with type inference):
//! let coerced: &TypedPath<UnknownBase, File> = path.coerce();
//! // Or with turbofish:
//! let coerced2 = path.coerce::<TypedPath<UnknownBase, File>>();
//! # }
//! ```
//!
//! ## Optional AsRef Integration
//!
//! Add the `asref` marker to also generate `AsRef` implementations:
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # struct Absolute;
//! # struct UnknownBase;  // Generic (subsumes Absolute)
//! # struct File;
//! #
//! #[derive(Coerce)]
//! #[coerce(borrowed = "TypedPath<UnknownBase, File>", asref)]
//! struct TypedPath<Base, Type> {
//!     base: PhantomData<Base>,
//!     ty: PhantomData<Type>,
//!     path: std::path::PathBuf,
//! }
//!
//! fn takes_asref(path: &impl AsRef<TypedPath<UnknownBase, File>>) {
//!     let p: &TypedPath<UnknownBase, File> = path.as_ref();
//!     // Use p...
//! }
//!
//! # fn main() {
//! let path = TypedPath::<Absolute, File> {
//!     base: PhantomData,
//!     ty: PhantomData,
//!     path: std::path::PathBuf::from("/test"),
//! };
//! takes_asref(&path); // Works: Absolute coerces to UnknownBase
//! # }
//! ```
//!
//! # Owned Coercion
//!
//! Use `#[coerce(owned = "...")]` to generate owned coercions (`T -> U`):
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # struct Initial;
//! # struct Final;
//! #
//! #[derive(Coerce)]
//! #[coerce(owned = "State<Final>")]
//! struct State<S> {
//!     marker: PhantomData<S>,
//!     data: Vec<i32>,
//! }
//!
//! # fn main() {
//! let state = State::<Initial> {
//!     marker: PhantomData,
//!     data: vec![1, 2, 3],
//! };
//! let final_state: State<Final> = state.into_coerced();
//! # }
//! ```
//!
//! # Cloned Coercion
//!
//! Use `#[coerce(cloned = "...")]` to generate cloned coercions (`&T -> U`), requiring `Clone`:
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # #[derive(Clone)]
//! # struct Validated;  // Specific marker
//! # #[derive(Clone)]
//! # struct AnyStatus;  // Generic marker (subsumes Validated, etc.)
//! #
//! #[derive(Coerce, Clone)]
//! #[coerce(cloned = "Message<AnyStatus>")]
//! struct Message<M> {
//!     marker: PhantomData<M>,
//!     content: String,
//! }
//!
//! # fn main() {
//! let msg = Message::<Validated> {
//!     marker: PhantomData,
//!     content: "Hello".to_string(),
//! };
//! // Clone and coerce to more generic type (source remains usable)
//! let coerced: Message<AnyStatus> = msg.to_coerced();
//! assert_eq!(msg.content, "Hello"); // Original still available
//! # }
//! ```

pub use phantom_coerce_derive::Coerce;
