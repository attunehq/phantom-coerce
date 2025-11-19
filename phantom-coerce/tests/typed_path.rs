use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct Absolute;
struct SomeBase;

struct File;
struct SomeType;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "TypedPath<Absolute, File>",
    borrowed_to = "TypedPath<SomeBase | Absolute, File | SomeType>"
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

    fn as_str(&self) -> &str {
        &self.path
    }
}

#[test]
fn single_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt");

    // Coerce Base parameter
    let coerced: &TypedPath<SomeBase, File> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");

    // Coerce Type parameter
    let coerced2: &TypedPath<Absolute, SomeType> = path.coerce();
    assert_eq!(coerced2.as_str(), "/home/user/file.txt");
}

#[test]
fn multi_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt");

    // Coerce both parameters at once
    let coerced: &TypedPath<SomeBase, SomeType> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");
}
