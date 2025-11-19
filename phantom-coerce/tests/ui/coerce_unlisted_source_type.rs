// This should fail because we try to coerce from a type not listed in borrowed_from

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct TypeB;
struct TypeC; // Not listed in borrowed_from
struct Generic;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA | TypeB>", borrowed_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {
    let container_c = Container::<TypeC> {
        phantom: PhantomData,
        value: "test".to_string(),
    };

    // This should fail: TypeC was not listed in borrowed_from
    let _coerced: &Container<Generic> = container_c.coerce();
}
