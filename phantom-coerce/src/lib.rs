//! Safe, zero-cost coercion between types differing only in PhantomData parameters.
//!
//! This crate provides a `#[derive(Coerce)]` macro that generates safe coercion methods
//! for types that differ only in their `PhantomData` type parameters.
//!
//! # Borrowed Coercion
//!
//! Use `#[coerce(borrowed_from = "...", borrowed_to = "...")]` to generate borrowed coercions (`&T -> &U`):
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # struct Absolute;
//! # struct Relative;
//! # struct UnknownBase;  // Generic (subsumes Absolute, Relative)
//! # struct File;
//! # struct UnknownType;  // Generic (subsumes File)
//! #
//! #[derive(Coerce)]
//! #[coerce(borrowed_from = "TypedPath<Absolute | Relative, File>", borrowed_to = "TypedPath<UnknownBase, File>")]
//! #[coerce(borrowed_from = "TypedPath<Absolute, File>", borrowed_to = "TypedPath<Absolute, UnknownType>")]
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
//! # struct Relative;
//! # struct UnknownBase;  // Generic (subsumes Absolute, Relative)
//! # struct File;
//! #
//! #[derive(Coerce)]
//! #[coerce(borrowed_from = "TypedPath<Absolute | Relative, File>", borrowed_to = "TypedPath<UnknownBase, File>", asref)]
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
//! Use `#[coerce(owned_from = "...", owned_to = "...")]` to generate owned coercions (`T -> U`):
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # struct Validated;
//! # struct Unvalidated;
//! # struct AnyStatus;  // Generic (subsumes Validated, Unvalidated)
//! #
//! #[derive(Coerce)]
//! #[coerce(owned_from = "Request<Validated | Unvalidated>", owned_to = "Request<AnyStatus>")]
//! struct Request<Status> {
//!     marker: PhantomData<Status>,
//!     url: String,
//! }
//!
//! # fn main() {
//! let validated_req = Request::<Validated> {
//!     marker: PhantomData,
//!     url: "https://api.example.com".to_string(),
//! };
//! // Coerce to more generic type (owned, consumes original)
//! let any_req: Request<AnyStatus> = validated_req.into_coerced();
//! # }
//! ```
//!
//! # Cloned Coercion
//!
//! Use `#[coerce(cloned_from = "...", cloned_to = "...")]` to generate cloned coercions (`&T -> U`), requiring `Clone`:
//!
//! ```rust
//! use std::marker::PhantomData;
//! use phantom_coerce::Coerce;
//!
//! # #[derive(Clone)]
//! # struct Json;
//! # #[derive(Clone)]
//! # struct Xml;
//! # #[derive(Clone)]
//! # struct AnyFormat;  // Generic format (subsumes Json, Xml, etc.)
//! #
//! #[derive(Coerce, Clone)]
//! #[coerce(cloned_from = "Message<Json | Xml>", cloned_to = "Message<AnyFormat>")]
//! struct Message<Format> {
//!     marker: PhantomData<Format>,
//!     content: String,
//! }
//!
//! # fn main() {
//! let json_msg = Message::<Json> {
//!     marker: PhantomData,
//!     content: r#"{"status": "ok"}"#.to_string(),
//! };
//! // Clone and coerce to more generic type (source remains usable)
//! let any_msg: Message<AnyFormat> = json_msg.to_coerced();
//! assert_eq!(json_msg.content, r#"{"status": "ok"}"#); // Original still available
//! # }
//! ```

pub use phantom_coerce_derive::Coerce;
