use phantom_coerce::Coerce;
use std::marker::PhantomData;

#[derive(Clone)]
struct WithAsRef;
#[derive(Clone)]
struct ToAsRef;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "AsRefTest<WithAsRef>",
    borrowed_to = "AsRefTest<ToAsRef>",
    asref
)]
struct AsRefTest<M> {
    marker: PhantomData<M>,
    value: i32,
}

impl<M> AsRefTest<M> {
    fn new(value: i32) -> Self {
        Self {
            marker: PhantomData,
            value,
        }
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}

#[test]
fn asref_integration() {
    let test = AsRefTest::<WithAsRef>::new(123);

    // Can use AsRef
    let as_ref: &AsRefTest<ToAsRef> = test.as_ref();
    assert_eq!(as_ref.get_value(), 123);

    // AsRef uses coerce internally
    let coerced: &AsRefTest<ToAsRef> = test.coerce();
    assert_eq!(coerced.get_value(), 123);

    // Can also use turbofish syntax
    let turbofish = test.coerce::<AsRefTest<ToAsRef>>();
    assert_eq!(turbofish.get_value(), 123);
}
