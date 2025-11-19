//! Example demonstrating type holes for partial coercion
//!
//! This example addresses the challenge of coercing one type parameter
//! while preserving another, preventing unintended cross-parameter coercions
//! like accidentally turning a File into a Directory.

use phantom_coerce::Coerce;
use std::marker::PhantomData;

// Base path types (absolute vs relative paths)
struct Absolute;
struct Relative;
struct SomeBase; // Generic base type (subsumes Absolute and Relative)

// Path content types (what the path points to)
struct File;
struct Directory;
struct SomeType; // Generic type (subsumes File and Directory)

/// A typed path that tracks both base type and content type
///
/// The type hole syntax `_` allows us to specify exactly which parameters
/// can be coerced while preserving others.
#[derive(Debug, Clone, Coerce)]
// Coerce Base parameter only: Absolute OR Relative → SomeBase, while Type stays unchanged
#[coerce(
    borrowed_from = "TypedPath<Absolute | Relative, _>",
    borrowed_to = "TypedPath<SomeBase, _>"
)]
// Coerce Type parameter only: File OR Directory → SomeType, while Base stays unchanged
#[coerce(
    borrowed_from = "TypedPath<_, File | Directory>",
    borrowed_to = "TypedPath<_, SomeType>"
)]
// Coerce both parameters: any combination → fully generic
#[coerce(
    borrowed_from = "TypedPath<Absolute | Relative, File | Directory>",
    borrowed_to = "TypedPath<SomeBase, SomeType>"
)]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

impl<Base, Type> TypedPath<Base, Type> {
    fn new(path: &str) -> Self {
        Self {
            base: PhantomData,
            ty: PhantomData,
            path: path.to_string(),
        }
    }

    fn path(&self) -> &str {
        &self.path
    }
}

/// Takes any path with File type, regardless of base
fn process_file<Base>(path: &TypedPath<Base, File>) {
    println!("Processing file: {}", path.path());
}

/// Takes any path with Directory type, regardless of base
fn process_directory<Base>(path: &TypedPath<Base, Directory>) {
    println!("Processing directory: {}", path.path());
}

/// Takes any absolute path, regardless of content type
fn process_absolute<Type>(path: &TypedPath<Absolute, Type>) {
    println!("Processing absolute path: {}", path.path());
}

/// Takes any path with SomeBase, regardless of content type
fn process_generic_base<Type>(path: &TypedPath<SomeBase, Type>) {
    println!("Processing path with generic base: {}", path.path());
}

/// Takes any path with SomeType, regardless of base
fn process_generic_type<Base>(path: &TypedPath<Base, SomeType>) {
    println!("Processing path with generic type: {}", path.path());
}

/// Takes a fully generic path
fn process_fully_generic(path: &TypedPath<SomeBase, SomeType>) {
    println!("Processing fully generic path: {}", path.path());
}

fn main() {
    println!("=== Partial Coercion Example ===\n");

    // Create some specific typed paths
    let abs_file = TypedPath::<Absolute, File>::new("/home/user/document.txt");
    let rel_file = TypedPath::<Relative, File>::new("../config.toml");
    let abs_dir = TypedPath::<Absolute, Directory>::new("/home/user/projects");
    let rel_dir = TypedPath::<Relative, Directory>::new("./src");

    println!("Created paths:");
    println!(
        "  abs_file: TypedPath<Absolute, File> = {}",
        abs_file.path()
    );
    println!(
        "  rel_file: TypedPath<Relative, File> = {}",
        rel_file.path()
    );
    println!(
        "  abs_dir: TypedPath<Absolute, Directory> = {}",
        abs_dir.path()
    );
    println!(
        "  rel_dir: TypedPath<Relative, Directory> = {}",
        rel_dir.path()
    );
    println!();

    println!("--- Coercing Base parameter only (Type preserved) ---\n");

    // Coerce Absolute → SomeBase while preserving File
    // Need turbofish here because multiple target types are possible
    process_generic_base(abs_file.coerce::<TypedPath<SomeBase, File>>());
    // The Type parameter is still File, so we can pass it to process_file
    process_file(abs_file.coerce::<TypedPath<SomeBase, File>>());

    // Coerce Relative → SomeBase while preserving File
    process_generic_base(rel_file.coerce::<TypedPath<SomeBase, File>>());
    process_file(rel_file.coerce::<TypedPath<SomeBase, File>>());

    // Coerce Absolute → SomeBase while preserving Directory
    process_generic_base(abs_dir.coerce::<TypedPath<SomeBase, Directory>>());
    // The Type parameter is still Directory, so we can pass it to process_directory
    process_directory(abs_dir.coerce::<TypedPath<SomeBase, Directory>>());

    println!();
    println!("--- Coercing Type parameter only (Base preserved) ---\n");

    // Coerce File → SomeType while preserving Absolute
    // Need turbofish here because multiple target types are possible
    process_generic_type(abs_file.coerce::<TypedPath<Absolute, SomeType>>());
    // The Base parameter is still Absolute, so we can pass it to process_absolute
    process_absolute(abs_file.coerce::<TypedPath<Absolute, SomeType>>());

    // Coerce Directory → SomeType while preserving Relative
    process_generic_type(rel_dir.coerce::<TypedPath<Relative, SomeType>>());

    println!();
    println!("--- Coercing both parameters to fully generic ---\n");

    // All paths can be coerced to the fully generic type
    process_fully_generic(abs_file.coerce());
    process_fully_generic(rel_file.coerce());
    process_fully_generic(abs_dir.coerce());
    process_fully_generic(rel_dir.coerce());

    println!();
    println!("=== Key Safety Guarantees ===\n");
    println!("✓ Type holes prevent unintended cross-parameter coercions");
    println!("✓ A File can NEVER accidentally become a Directory");
    println!("✓ An Absolute path can NEVER accidentally become Relative");
    println!("✓ Each coercion is explicit and type-checked at compile time");
    println!();

    println!("The following would NOT compile:");
    println!("  // abs_file.coerce::<TypedPath<SomeBase, Directory>>();");
    println!("  // ERROR: abs_file is File, cannot coerce to Directory");
    println!();
    println!("  // abs_file.coerce::<TypedPath<Relative, SomeType>>();");
    println!("  // ERROR: abs_file is Absolute, cannot coerce to Relative");
}
