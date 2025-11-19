use phantom_coerce::Coerce;
use std::marker::PhantomData;

#[derive(Coerce)]
#[coerce(owned_from = "MultiOwned<Z, Z>", owned_to = "MultiOwned<X, Y>")]
#[coerce(owned_from = "MultiOwned<X, Y>", owned_to = "MultiOwned<Y, X>")]
struct MultiOwned<P1, P2> {
    p1: PhantomData<P1>,
    p2: PhantomData<P2>,
    data: Vec<i32>,
}

#[derive(Clone)]
struct X;
#[derive(Clone)]
struct Y;
#[derive(Clone)]
struct Z;

impl<P1, P2> MultiOwned<P1, P2> {
    fn new(data: Vec<i32>) -> Self {
        Self {
            p1: PhantomData,
            p2: PhantomData,
            data,
        }
    }

    fn sum(&self) -> i32 {
        self.data.iter().sum()
    }
}

#[test]
fn multi_owned_coercion() {
    let multi = MultiOwned::<Z, Z>::new(vec![1, 2, 3, 4, 5]);

    // Coerce to X, Y
    let coerced: MultiOwned<X, Y> = multi.into_coerced();
    assert_eq!(coerced.sum(), 15);

    // Can chain owned coercions
    let coerced2: MultiOwned<Y, X> = coerced.into_coerced();
    assert_eq!(coerced2.sum(), 15);
}
