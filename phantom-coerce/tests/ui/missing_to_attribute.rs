// This should fail because borrowed_from is specified but borrowed_to is missing

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct TypeB;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<TypeA>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
