// This should fail because borrowed_to has an empty string

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA>", borrowed_to = "")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
