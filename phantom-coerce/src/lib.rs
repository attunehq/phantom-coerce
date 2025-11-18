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
//! # struct SomeBase;
//! # struct File;
//! # struct SomeType;
//! #
//! #[derive(Coerce)]
//! #[coerce(borrowed = "TypedPath<SomeBase, File>")]
//! #[coerce(borrowed = "TypedPath<Absolute, SomeType>")]
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
//! let coerced: &TypedPath<SomeBase, File> = path.coerce();
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
//! # struct Source;
//! # #[derive(Clone)]
//! # struct Target;
//! #
//! #[derive(Coerce, Clone)]
//! #[coerce(cloned = "Message<Target>")]
//! struct Message<M> {
//!     marker: PhantomData<M>,
//!     content: String,
//! }
//!
//! # fn main() {
//! let msg = Message::<Source> {
//!     marker: PhantomData,
//!     content: "Hello".to_string(),
//! };
//! // Clone and coerce (source remains usable)
//! let coerced: Message<Target> = msg.to_coerced();
//! assert_eq!(msg.content, "Hello"); // Original still available
//! # }
//! ```

pub use phantom_coerce_derive::Coerce;
