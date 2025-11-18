// This should fail because no #[coerce(...)] attributes are provided

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Marker;

#[derive(Coerce)]
struct NoAttrs<M> {
    phantom: PhantomData<M>,
    value: i32,
}

fn main() {}
