// This should fail because we try to coerce from Container<C, _> when only Container<A, _> is allowed

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct A;
struct B;
struct C; // Not listed in borrowed_from
struct X;
struct Y;

#[derive(Coerce)]
#[coerce(borrowed_from = "Container<A, _>", borrowed_to = "Container<B, _>")]
struct Container<T1, T2> {
    phantom1: PhantomData<T1>,
    phantom2: PhantomData<T2>,
    value: String,
}

fn main() {
    let container = Container::<C, X> {
        phantom1: PhantomData,
        phantom2: PhantomData,
        value: "test".to_string(),
    };

    // This should fail: Container<C, _> was not listed in borrowed_from (only Container<A, _> is allowed)
    let _coerced: &Container<B, X> = container.coerce();
}
