use phantom_coerce::Coerce;
use std::marker::PhantomData;

#[derive(Coerce)]
#[coerce(owned_from = "Owned<OriginalOwned>", owned_to = "Owned<OtherOwned>")]
#[coerce(
    borrowed_from = "Owned<OriginalOwned>",
    borrowed_to = "Owned<OtherOwned>"
)]
struct Owned<Marker> {
    phantom: PhantomData<Marker>,
    value: String,
}

struct OriginalOwned;
struct OtherOwned;

impl<M> Owned<M> {
    fn new(value: &str) -> Self {
        Self {
            phantom: PhantomData,
            value: value.to_string(),
        }
    }

    fn get_value(&self) -> &str {
        &self.value
    }
}

#[test]
fn owned_coercion() {
    let owned = Owned::<OriginalOwned>::new("hello");

    // Test owned coercion (consumes the original)
    let coerced: Owned<OtherOwned> = owned.into_coerced();
    assert_eq!(coerced.get_value(), "hello");
}

#[test]
fn borrowed_vs_owned() {
    let owned1 = Owned::<OriginalOwned>::new("borrowed");

    // Can use borrowed coercion multiple times
    let borrowed1: &Owned<OtherOwned> = owned1.coerce();
    assert_eq!(borrowed1.get_value(), "borrowed");
    let borrowed2: &Owned<OtherOwned> = owned1.coerce();
    assert_eq!(borrowed2.get_value(), "borrowed");

    // Now consume it with owned coercion
    let owned_coerced: Owned<OtherOwned> = owned1.into_coerced();
    assert_eq!(owned_coerced.get_value(), "borrowed");
}
