// This should fail because borrowed_from has an empty string

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Generic;

#[derive(Coerce)]
#[coerce(borrowed_from = "", borrowed_to = "Container<Generic>")]
struct Container<T> {
    phantom: PhantomData<T>,
    value: String,
}

fn main() {}
