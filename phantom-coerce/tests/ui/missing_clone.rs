// This should fail because the type doesn't implement Clone but uses cloned coercion

use std::marker::PhantomData;
use phantom_coerce::Coerce;

struct Marker1;
struct Marker2;

// Missing #[derive(Clone)]
#[derive(Coerce)]
#[coerce(cloned_from = "NoClone<Marker1>", cloned_to = "NoClone<Marker2>")]
struct NoClone<M> {
    phantom: PhantomData<M>,
    value: String,
}

fn main() {
    let no_clone = NoClone::<Marker1> {
        phantom: PhantomData,
        value: "test".to_string(),
    };

    let _: NoClone<Marker2> = no_clone.to_coerced();
}
