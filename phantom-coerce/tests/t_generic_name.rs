// Test that using T as a generic name no longer conflicts with the method's generic

use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct TypeA;
struct TypeB;

#[derive(Coerce)]
#[coerce(borrowed_from = "UsesT<TypeA>", borrowed_to = "UsesT<TypeB>")]
struct UsesT<T> {
    phantom: PhantomData<T>,
}

#[test]
fn t_generic_name_no_longer_conflicts() {
    let test = UsesT::<TypeA> {
        phantom: PhantomData,
    };

    // This used to fail with "the name `T` is already used" but now works!
    let _coerced: &UsesT<TypeB> = test.coerce();
}
