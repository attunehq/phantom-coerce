use phantom_coerce::Coerce;
use std::marker::PhantomData;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "Complex<A, State>",
    borrowed_to = "Complex<B, OtherState> | Complex<OtherData, AnotherState>"
)]
struct Complex<D, S> {
    phantom_data: PhantomData<D>,
    phantom_state: PhantomData<S>,
    real_field1: String,
    real_field2: i32,
}

#[derive(Clone)]
struct A;
#[derive(Clone)]
struct B;
struct State;
struct OtherState;
struct OtherData;
struct AnotherState;

impl<D, S> Complex<D, S> {
    fn new(s: &str, i: i32) -> Self {
        Self {
            phantom_data: PhantomData,
            phantom_state: PhantomData,
            real_field1: s.to_string(),
            real_field2: i,
        }
    }

    fn get_data(&self) -> (&str, i32) {
        (&self.real_field1, self.real_field2)
    }
}

#[test]
fn complex_coercion() {
    // Test coercing to Complex<B, OtherState> (both params changed)
    let complex1 = Complex::<A, State>::new("test1", 123);
    let coerced1: &Complex<B, OtherState> = complex1.coerce();
    assert_eq!(coerced1.get_data(), ("test1", 123));

    // Test coercing to Complex<OtherData, AnotherState> (both params changed differently)
    let complex2 = Complex::<A, State>::new("test2", 456);
    let coerced2: &Complex<OtherData, AnotherState> = complex2.coerce();
    assert_eq!(coerced2.get_data(), ("test2", 456));
}
