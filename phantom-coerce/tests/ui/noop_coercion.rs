// This should fail because we're coercing from the same type to itself (no-op)

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA>", borrowed_to = "Container<TypeA>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
