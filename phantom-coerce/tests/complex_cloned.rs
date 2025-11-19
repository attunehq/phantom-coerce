use phantom_coerce::Coerce;
use std::marker::PhantomData;

#[derive(Coerce, Clone)]
#[coerce(
    cloned_from = "ComplexCloned<A, X>",
    cloned_to = "ComplexCloned<B, Y> | ComplexCloned<A, Y>"
)]
struct ComplexCloned<P1, P2> {
    p1: PhantomData<P1>,
    p2: PhantomData<P2>,
    data: Vec<String>,
    count: i32,
}

#[derive(Clone)]
struct A;
#[derive(Clone)]
struct B;
#[derive(Clone)]
struct X;
#[derive(Clone)]
struct Y;

impl<P1, P2> ComplexCloned<P1, P2> {
    fn new(data: &[&str], count: i32) -> Self {
        Self {
            p1: PhantomData,
            p2: PhantomData,
            data: data.iter().map(|s| s.to_string()).collect(),
            count,
        }
    }

    fn get_count(&self) -> i32 {
        self.count
    }

    fn get_data_len(&self) -> usize {
        self.data.len()
    }
}

#[test]
fn complex_cloned_coercion() {
    let complex = ComplexCloned::<A, X>::new(&["one", "two", "three"], 42);

    // Clone and coerce to different markers
    let coerced1: ComplexCloned<B, Y> = complex.to_coerced();
    assert_eq!(coerced1.get_count(), 42);
    assert_eq!(coerced1.get_data_len(), 3);

    // Original still works
    assert_eq!(complex.get_count(), 42);

    // Can coerce to different target
    let coerced2: ComplexCloned<A, Y> = complex.to_coerced();
    assert_eq!(coerced2.get_count(), 42);
    assert_eq!(coerced2.get_data_len(), 3);
}
