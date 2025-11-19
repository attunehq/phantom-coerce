// This should fail because owned_from is used with borrowed_to (mismatched modes)

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct Generic;

#[derive(Coerce)]
#[coerce(owned_from = "Container<TypeA>", borrowed_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
