use phantom_coerce::Coerce;
/// Example demonstrating phantom-coerce with a TypedPath system
///
/// This example shows how to use PhantomData to track path types at compile-time,
/// and how to use the Coerce derive macro to safely coerce between different
/// phantom type parameters.
use std::marker::PhantomData;
use std::path::PathBuf;

// Base path types (tracking whether path is absolute or relative)
struct Absolute;
struct Relative;
struct UnknownBase;

// Path content types (tracking what the path points to)
struct File;
struct Directory;
struct UnknownType;

/// A strongly-typed path that tracks both base type and content type
#[derive(Debug, Coerce)]
#[coerce(
    borrowed_from = "TypedPath<Absolute | Relative, File>",
    borrowed_to = "TypedPath<UnknownBase, File>",
    asref
)]
#[coerce(
    borrowed_from = "TypedPath<Absolute, File>",
    borrowed_to = "TypedPath<Absolute, UnknownType>"
)]
#[coerce(
    borrowed_from = "TypedPath<Absolute | Relative, File | Directory>",
    borrowed_to = "TypedPath<UnknownBase, UnknownType>"
)]
struct TypedPath<Base, Type> {
    _base: PhantomData<Base>,
    _type: PhantomData<Type>,
    path: PathBuf,
}

impl<Base, Type> TypedPath<Base, Type> {
    fn as_path(&self) -> &std::path::Path {
        &self.path
    }
}

impl TypedPath<Absolute, File> {
    fn new_absolute_file(path: PathBuf) -> Self {
        Self {
            _base: PhantomData,
            _type: PhantomData,
            path,
        }
    }
}

impl TypedPath<Relative, Directory> {
    fn new_relative_dir(path: PathBuf) -> Self {
        Self {
            _base: PhantomData,
            _type: PhantomData,
            path,
        }
    }
}

// Example function that accepts any file path regardless of base type
fn print_file_path<Base>(path: &TypedPath<Base, File>) {
    println!("File path: {}", path.as_path().display());
}

// Example function that accepts any absolute path regardless of type
fn print_absolute_path<Type>(path: &TypedPath<Absolute, Type>) {
    println!("Absolute path: {}", path.as_path().display());
}

// Example function that accepts any path (maximally generic)
fn print_any_path<Base, Type>(path: &TypedPath<Base, Type>) {
    println!("Path: {}", path.as_path().display());
}

// Example function using AsRef for ergonomic API design
fn process_file_path(path: &impl AsRef<TypedPath<UnknownBase, File>>) {
    let p: &TypedPath<UnknownBase, File> = path.as_ref();
    println!("Processing file: {}", p.as_path().display());
}

fn main() {
    // Create a specific typed path
    let file_path =
        TypedPath::<Absolute, File>::new_absolute_file(PathBuf::from("/home/user/document.txt"));

    println!("=== Demonstrating type coercion ===\n");

    // We can use it directly with the most specific type
    println!("Original type: TypedPath<Absolute, File>");
    println!("Path: {}\n", file_path.as_path().display());

    // Coerce to UnknownBase while keeping File type
    println!("Coercing to TypedPath<UnknownBase, File>:");
    let coerced1: &TypedPath<UnknownBase, File> = file_path.coerce();
    print_file_path(coerced1);
    println!();

    // Coerce to UnknownType while keeping Absolute base
    println!("Coercing to TypedPath<Absolute, UnknownType>:");
    let coerced2: &TypedPath<Absolute, UnknownType> = file_path.coerce();
    print_absolute_path(coerced2);
    println!();

    // Coerce to fully generic (both parameters unknown)
    println!("Coercing to TypedPath<UnknownBase, UnknownType>:");
    let coerced3: &TypedPath<UnknownBase, UnknownType> = file_path.coerce();
    print_any_path(coerced3);
    println!();

    // Demonstrate AsRef integration
    println!("Using AsRef integration:");
    process_file_path(&file_path); // Works because of AsRef<TypedPath<UnknownBase, File>> impl
    println!();

    // Create another path with different types
    let dir_path = TypedPath::<Relative, Directory>::new_relative_dir(PathBuf::from("./my_folder"));

    println!("=== Another example ===\n");
    println!("Original type: TypedPath<Relative, Directory>");
    println!("Path: {}\n", dir_path.as_path().display());

    // This path can also be coerced to UnknownBase, UnknownType
    let coerced4: &TypedPath<UnknownBase, UnknownType> = dir_path.coerce();
    print_any_path(coerced4);

    println!("\n=== Benefits ===");
    println!("1. Type safety: Cannot accidentally pass a File path to a Directory-only function");
    println!("2. Flexibility: Can coerce specific types to more generic ones when needed");
    println!("3. Zero cost: All coercions are compile-time only, no runtime overhead");
    println!("4. Safety: Compile-time checks ensure coercions are valid");
    println!("5. AsRef integration: Works seamlessly with APIs expecting AsRef<T>");
}
