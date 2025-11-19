// This should fail because cloned_from is used with owned_to (mismatched modes)

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct Generic;

#[derive(Coerce, Clone)]
#[coerce(cloned_from = "Container<TypeA>", owned_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
