// This should fail because Coerce only works on structs, not enums

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Marker1;
struct Marker2;

#[derive(Coerce)]
#[coerce(borrowed = "BadEnum<Marker2>")]
enum BadEnum<M> {
    Variant {
        phantom: PhantomData<M>,
        value: i32,
    }
}

fn main() {}
