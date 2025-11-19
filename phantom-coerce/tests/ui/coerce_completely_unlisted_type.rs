// This should fail because we try to coerce from a completely different type structure

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct TypeB;
struct Generic;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA | TypeB>", borrowed_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

// A completely different type
struct DifferentContainer<T> {
    phantom: PhantomData<T>,
    data: i32,
}

fn main() {
    let different = DifferentContainer::<TypeA> {
        phantom: PhantomData,
        data: 42,
    };

    // This should fail: DifferentContainer is not Container, even though TypeA is listed
    let _coerced: &Container<Generic> = different.coerce();
}
