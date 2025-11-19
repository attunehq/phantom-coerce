/// Test that type hole syntax prevents cross-parameter coercion
use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct Absolute;
struct SomeBase;

struct File;
struct Directory;

#[derive(Coerce)]
#[coerce(borrowed_from = "TypedPath<Absolute, _>", borrowed_to = "TypedPath<SomeBase, _>")]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

fn main() {
    let path = TypedPath::<Absolute, File> {
        base: PhantomData,
        ty: PhantomData,
        path: "/file.txt".to_string(),
    };

    // This should fail: type hole preserves Type parameter, so File cannot become Directory
    let _coerced: &TypedPath<SomeBase, Directory> = path.coerce();
}
