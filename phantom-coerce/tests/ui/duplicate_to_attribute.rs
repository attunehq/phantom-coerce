// This should fail because borrowed_to is specified twice in the same attribute

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct TypeB;
struct Generic;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA>", borrowed_to = "Container<Generic>", borrowed_to = "Container<TypeB>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
