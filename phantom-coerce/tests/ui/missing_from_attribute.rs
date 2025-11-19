// This should fail because borrowed_to is specified but borrowed_from is missing

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct TypeA;
struct TypeB;

#[derive(Coerce)]
#[coerce(borrowed_to = "Container<TypeB>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
