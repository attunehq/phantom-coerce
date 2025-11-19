// This should fail because asref is only valid for borrowed coercions

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Marker1;
struct Marker2;

#[derive(Coerce, Clone)]
#[coerce(cloned_from = "BadAsRef<Marker1>", cloned_to = "BadAsRef<Marker2>", asref)]
struct BadAsRef<M> {
    phantom: PhantomData<M>,
    value: String,
}

fn main() {}
