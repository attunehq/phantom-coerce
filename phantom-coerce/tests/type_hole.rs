use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct ParamA;
struct ParamB;
struct ParamX;
struct ParamY;
struct GenericParam;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "TypeHoleSecond<ParamA | ParamB, _>",
    borrowed_to = "TypeHoleSecond<GenericParam, _>"
)]
struct TypeHoleSecond<First, Second> {
    phantom_first: PhantomData<First>,
    phantom_second: PhantomData<Second>,
    value: String,
}

impl<First, Second> TypeHoleSecond<First, Second> {
    fn new(value: &str) -> Self {
        Self {
            phantom_first: PhantomData,
            phantom_second: PhantomData,
            value: value.to_string(),
        }
    }

    fn get_value(&self) -> &str {
        &self.value
    }
}

#[test]
fn type_hole_in_second_position() {
    // Test with ParamA in first position, ParamX in second position
    let test_a = TypeHoleSecond::<ParamA, ParamX>::new("type hole second A");

    // ParamA -> GenericParam, ParamX preserved by type hole
    let coerced_a: &TypeHoleSecond<GenericParam, ParamX> = test_a.coerce();
    assert_eq!(coerced_a.get_value(), "type hole second A");

    // Test with ParamB in first position, ParamY in second position
    let test_b = TypeHoleSecond::<ParamB, ParamY>::new("type hole second B");

    // ParamB -> GenericParam, ParamY preserved by type hole
    let coerced_b: &TypeHoleSecond<GenericParam, ParamY> = test_b.coerce();
    assert_eq!(coerced_b.get_value(), "type hole second B");
}
