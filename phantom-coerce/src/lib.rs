//! Safe, zero-cost coercion between types differing only in PhantomData parameters.
//!
//! This crate provides a `#[derive(Coerce)]` macro that generates safe coercion methods
//! for types that differ only in their `PhantomData` type parameters.
//!
//! # Example
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
//! #[coerce(borrowed = "TypedPath<SomeBase, SomeType>")]
//! struct TypedPath<Base, Type> {
//!     base: PhantomData<Base>,
//!     ty: PhantomData<Type>,
//!     path: std::path::PathBuf,
//! }
//! ```

pub use phantom_coerce_derive::Coerce;
