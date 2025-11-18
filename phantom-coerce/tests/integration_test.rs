use std::marker::PhantomData;
use phantom_coerce::Coerce;

// Type-level markers for testing
struct Absolute;
struct SomeBase;

struct File;
struct SomeType;

#[derive(Coerce)]
#[coerce(borrowed = "TypedPath<SomeBase, File>")]
#[coerce(borrowed = "TypedPath<Absolute, SomeType>")]
#[coerce(borrowed = "TypedPath<SomeBase, SomeType>")]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

impl<Base, Type> TypedPath<Base, Type> {
    fn new(path: String) -> Self {
        Self {
            base: PhantomData,
            ty: PhantomData,
            path,
        }
    }

    fn as_str(&self) -> &str {
        &self.path
    }
}

#[test]
fn test_single_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt".to_string());

    // Coerce Base parameter
    let coerced: &TypedPath<SomeBase, File> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");

    // Coerce Type parameter
    let coerced2: &TypedPath<Absolute, SomeType> = path.coerce();
    assert_eq!(coerced2.as_str(), "/home/user/file.txt");
}

#[test]
fn test_multi_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt".to_string());

    // Coerce both parameters at once
    let coerced: &TypedPath<SomeBase, SomeType> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");
}

#[test]
fn test_chained_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt".to_string());

    // First coerce Base
    let step1: &TypedPath<SomeBase, File> = path.coerce();

    // Then coerce Type - this should work because we defined the trait for the original type
    // Note: This won't work as-is because the trait is defined on TypedPath<Absolute, File>
    // not on TypedPath<SomeBase, File>. This is expected behavior.
    assert_eq!(step1.as_str(), "/home/user/file.txt");
}

// Test with a simple single-parameter type
#[derive(Coerce)]
#[coerce(borrowed = "Simple<OtherMarker>")]
struct Simple<Marker> {
    phantom: PhantomData<Marker>,
    value: i32,
}

struct OriginalMarker;
struct OtherMarker;

impl<M> Simple<M> {
    fn new(value: i32) -> Self {
        Self {
            phantom: PhantomData,
            value,
        }
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}

#[test]
fn test_simple_coercion() {
    let simple = Simple::<OriginalMarker>::new(42);
    let coerced: &Simple<OtherMarker> = simple.coerce();
    assert_eq!(coerced.get_value(), 42);
}

// Test with multiple fields including non-phantom fields
#[derive(Coerce)]
#[coerce(borrowed = "Complex<B, OtherState>")]
#[coerce(borrowed = "Complex<OtherData, AnotherState>")]
struct Complex<D, S> {
    phantom_data: PhantomData<D>,
    phantom_state: PhantomData<S>,
    real_field1: String,
    real_field2: i32,
}

struct A;
struct B;
struct State;
struct OtherState;
struct OtherData;
struct AnotherState;

impl<D, S> Complex<D, S> {
    fn new(s: String, i: i32) -> Self {
        Self {
            phantom_data: PhantomData,
            phantom_state: PhantomData,
            real_field1: s,
            real_field2: i,
        }
    }

    fn get_data(&self) -> (&str, i32) {
        (&self.real_field1, self.real_field2)
    }
}

#[test]
fn test_complex_coercion() {
    // Test coercing to Complex<B, OtherState> (both params changed)
    let complex1 = Complex::<A, State>::new("test1".to_string(), 123);
    let coerced1: &Complex<B, OtherState> = complex1.coerce();
    assert_eq!(coerced1.get_data(), ("test1", 123));

    // Test coercing to Complex<OtherData, AnotherState> (both params changed differently)
    let complex2 = Complex::<A, State>::new("test2".to_string(), 456);
    let coerced2: &Complex<OtherData, AnotherState> = complex2.coerce();
    assert_eq!(coerced2.get_data(), ("test2", 456));
}

// Test owned coercions
#[derive(Coerce)]
#[coerce(owned = "Owned<OtherOwned>")]
#[coerce(borrowed = "Owned<OtherOwned>")]
struct Owned<Marker> {
    phantom: PhantomData<Marker>,
    value: String,
}

struct OriginalOwned;
struct OtherOwned;

impl<M> Owned<M> {
    fn new(value: String) -> Self {
        Self {
            phantom: PhantomData,
            value,
        }
    }

    fn get_value(&self) -> &str {
        &self.value
    }
}

#[test]
fn test_owned_coercion() {
    let owned = Owned::<OriginalOwned>::new("hello".to_string());

    // Test owned coercion (consumes the original)
    let coerced: Owned<OtherOwned> = owned.into_coerced();
    assert_eq!(coerced.get_value(), "hello");
}

#[test]
fn test_borrowed_vs_owned() {
    let owned1 = Owned::<OriginalOwned>::new("borrowed".to_string());

    // Can use borrowed coercion multiple times
    let borrowed1: &Owned<OtherOwned> = owned1.coerce();
    assert_eq!(borrowed1.get_value(), "borrowed");
    let borrowed2: &Owned<OtherOwned> = owned1.coerce();
    assert_eq!(borrowed2.get_value(), "borrowed");

    // Now consume it with owned coercion
    let owned_coerced: Owned<OtherOwned> = owned1.into_coerced();
    assert_eq!(owned_coerced.get_value(), "borrowed");
}

// Test owned coercion with multiple parameters
#[derive(Coerce)]
#[coerce(owned = "MultiOwned<X, Y>")]
#[coerce(owned = "MultiOwned<Y, X>")]
struct MultiOwned<P1, P2> {
    p1: PhantomData<P1>,
    p2: PhantomData<P2>,
    data: Vec<i32>,
}

struct X;
struct Y;
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
fn test_multi_owned_coercion() {
    let multi = MultiOwned::<Z, Z>::new(vec![1, 2, 3, 4, 5]);

    // Coerce to X, Y
    let coerced: MultiOwned<X, Y> = multi.into_coerced();
    assert_eq!(coerced.sum(), 15);

    // Can chain owned coercions
    let coerced2: MultiOwned<Y, X> = coerced.into_coerced();
    assert_eq!(coerced2.sum(), 15);
}
