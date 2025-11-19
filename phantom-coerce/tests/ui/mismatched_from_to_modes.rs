// This should fail because borrowed_from is used with cloned_to (mismatched modes)

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct Generic;

#[derive(Coerce, Clone)]
#[coerce(borrowed_from = "Container<TypeA>", cloned_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
