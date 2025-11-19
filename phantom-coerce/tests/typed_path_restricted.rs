use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct Relative;
struct Absolute;
struct SomeBase;

struct Directory;
struct File;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "TypedPathRestricted<Absolute | Relative, _>",
    borrowed_to = "TypedPathRestricted<SomeBase, _>"
)]
struct TypedPathRestricted<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

impl<Base, Type> TypedPathRestricted<Base, Type> {
    fn new(path: &str) -> Self {
        Self {
            base: PhantomData,
            ty: PhantomData,
            path: path.to_string(),
        }
    }

    fn as_str(&self) -> &str {
        &self.path
    }
}

#[test]
fn single_param_coercion_restricted() {
    let path = TypedPathRestricted::<Absolute, File>::new("/home/user/file.txt");

    // Coerce Base parameter only (Type stays as File)
    let coerced: &TypedPathRestricted<SomeBase, File> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");

    // TypeHole ensures Type parameter is preserved
    // This CANNOT compile (File doesn't change to Directory):
    // let coerced: &TypedPathRestricted<SomeBase, Directory> = path.coerce();

    let path = TypedPathRestricted::<Relative, Directory>::new("user/dir");

    // Coerce Base parameter only (Type stays as Directory)
    let coerced: &TypedPathRestricted<SomeBase, Directory> = path.coerce();
    assert_eq!(coerced.as_str(), "user/dir");

    // TypeHole ensures Type parameter is preserved
    // This CANNOT compile (Directory doesn't change to File):
    // let coerced: &TypedPathRestricted<SomeBase, File> = path.coerce();
}
