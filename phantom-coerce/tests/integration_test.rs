use phantom_coerce::Coerce;
use std::marker::PhantomData;

struct Relative;
struct Absolute;
struct SomeBase;

struct Directory;
struct File;
struct SomeType;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "TypedPathRestricted<Absolute | Relative, _>",
    borrowed_to = "TypedPathRestricted<SomeBase, _>"
)]
struct TypedPathRestricted<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

impl<Base, Type> TypedPathRestricted<Base, Type> {
    fn new(path: &str) -> Self {
        Self {
            base: PhantomData,
            ty: PhantomData,
            path: path.to_string(),
        }
    }

    fn as_str(&self) -> &str {
        &self.path
    }
}

#[test]
fn test_single_param_coercion_restricted() {
    let path = TypedPathRestricted::<Absolute, File>::new("/home/user/file.txt");

    // Coerce Base parameter only (Type stays as File)
    let coerced: &TypedPathRestricted<SomeBase, File> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");

    // TypeHole ensures Type parameter is preserved
    // This CANNOT compile (File doesn't change to Directory):
    // let coerced: &TypedPathRestricted<SomeBase, Directory> = path.coerce();

    let path = TypedPathRestricted::<Relative, Directory>::new("user/dir");

    // Coerce Base parameter only (Type stays as Directory)
    let coerced: &TypedPathRestricted<SomeBase, Directory> = path.coerce();
    assert_eq!(coerced.as_str(), "user/dir");

    // TypeHole ensures Type parameter is preserved
    // This CANNOT compile (Directory doesn't change to File):
    // let coerced: &TypedPathRestricted<SomeBase, File> = path.coerce();
}

#[derive(Coerce)]
#[coerce(
    borrowed_from = "TypedPath<Absolute, File>",
    borrowed_to = "TypedPath<SomeBase | Absolute, File | SomeType>"
)]
struct TypedPath<Base, Type> {
    base: PhantomData<Base>,
    ty: PhantomData<Type>,
    path: String,
}

impl<Base, Type> TypedPath<Base, Type> {
    fn new(path: &str) -> Self {
        Self {
            base: PhantomData,
            ty: PhantomData,
            path: path.to_string(),
        }
    }

    fn as_str(&self) -> &str {
        &self.path
    }
}

#[test]
fn test_single_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt");

    // Coerce Base parameter
    let coerced: &TypedPath<SomeBase, File> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");

    // Coerce Type parameter
    let coerced2: &TypedPath<Absolute, SomeType> = path.coerce();
    assert_eq!(coerced2.as_str(), "/home/user/file.txt");
}

#[test]
fn test_multi_param_coercion() {
    let path = TypedPath::<Absolute, File>::new("/home/user/file.txt");

    // Coerce both parameters at once
    let coerced: &TypedPath<SomeBase, SomeType> = path.coerce();
    assert_eq!(coerced.as_str(), "/home/user/file.txt");
}

// Test with a simple single-parameter type
#[derive(Coerce)]
#[coerce(
    borrowed_from = "Simple<OriginalMarker>",
    borrowed_to = "Simple<OtherMarker>"
)]
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
fn test_complex_coercion() {
    // Test coercing to Complex<B, OtherState> (both params changed)
    let complex1 = Complex::<A, State>::new("test1", 123);
    let coerced1: &Complex<B, OtherState> = complex1.coerce();
    assert_eq!(coerced1.get_data(), ("test1", 123));

    // Test coercing to Complex<OtherData, AnotherState> (both params changed differently)
    let complex2 = Complex::<A, State>::new("test2", 456);
    let coerced2: &Complex<OtherData, AnotherState> = complex2.coerce();
    assert_eq!(coerced2.get_data(), ("test2", 456));
}

// Test owned coercions
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
fn test_owned_coercion() {
    let owned = Owned::<OriginalOwned>::new("hello");

    // Test owned coercion (consumes the original)
    let coerced: Owned<OtherOwned> = owned.into_coerced();
    assert_eq!(coerced.get_value(), "hello");
}

#[test]
fn test_borrowed_vs_owned() {
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

// Test owned coercion with multiple parameters
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
fn test_multi_owned_coercion() {
    let multi = MultiOwned::<Z, Z>::new(vec![1, 2, 3, 4, 5]);

    // Coerce to X, Y
    let coerced: MultiOwned<X, Y> = multi.into_coerced();
    assert_eq!(coerced.sum(), 15);

    // Can chain owned coercions
    let coerced2: MultiOwned<Y, X> = coerced.into_coerced();
    assert_eq!(coerced2.sum(), 15);
}

// Test cloned coercions (requires Clone)
#[derive(Coerce, Clone)]
#[coerce(
    cloned_from = "Cloned<ClonedMarker1>",
    cloned_to = "Cloned<OtherMarker>"
)]
struct Cloned<Marker> {
    phantom: PhantomData<Marker>,
    value: String,
}

#[derive(Clone)]
struct ClonedMarker1;

impl<M> Cloned<M> {
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
fn test_cloned_coercion() {
    let cloned = Cloned::<ClonedMarker1>::new("hello cloned");

    // Clone and coerce (source remains usable)
    let coerced: Cloned<OtherMarker> = cloned.to_coerced();
    assert_eq!(coerced.get_value(), "hello cloned");

    // Original is still available
    assert_eq!(cloned.get_value(), "hello cloned");

    // Can call to_coerced multiple times
    let coerced2: Cloned<OtherMarker> = cloned.to_coerced();
    assert_eq!(coerced2.get_value(), "hello cloned");
}

// Test cloned with non-Clone fields should not compile (compile-time check)
// This is commented out since it should fail to compile
// #[derive(Coerce)]
// #[coerce(cloned = "NonCloneable<OtherMarker>")]
// struct NonCloneable<Marker> {
//     phantom: PhantomData<Marker>,
//     value: std::rc::Rc<String>, // Rc is Clone
//     non_clone: std::sync::MutexGuard<'static, i32>, // MutexGuard is NOT Clone
// }

// Test cloned with complex types
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
fn test_complex_cloned_coercion() {
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

// Test AsRef integration with borrowed coercion
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
fn test_asref_integration() {
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

#[test]
fn test_turbofish_syntax() {
    // Test borrowed with turbofish
    let path = TypedPath::<Absolute, File>::new("/test");
    let coerced = path.coerce::<TypedPath<SomeBase, File>>();
    assert_eq!(coerced.as_str(), "/test");

    // Test owned with turbofish
    let owned = Owned::<OriginalOwned>::new("owned turbofish");
    let coerced_owned = owned.into_coerced::<Owned<OtherOwned>>();
    assert_eq!(coerced_owned.get_value(), "owned turbofish");

    // Test cloned with turbofish
    let cloned = Cloned::<ClonedMarker1>::new("turbofish");
    let coerced_cloned = cloned.to_coerced::<Cloned<OtherMarker>>();
    assert_eq!(coerced_cloned.get_value(), "turbofish");
}

// Test type hole in different positions with explicit source types
struct ParamA;
struct ParamB;
struct ParamX;
struct ParamY;
struct GenericParam;

// TypeHole in second position: coerce first param, preserve second param
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
fn test_type_hole_in_second_position() {
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

// Test nested alternatives: both parameter-level and top-level on both from/to sides
// This creates a massive Cartesian product to stress-test the parser
struct SourceA;
struct SourceB;
struct SourceC;

struct SourceX;
struct SourceY;
struct SourceZ;

struct TargetM;
struct TargetN;
struct TargetO;

struct TargetP;
struct TargetQ;
struct TargetR;

struct TargetS;
struct TargetT;

struct TargetU;
struct TargetV;

#[derive(Coerce)]
#[coerce(
    borrowed_from = "NestedAlternatives<SourceA | SourceB, SourceX | SourceY> | NestedAlternatives<SourceC, SourceZ>",
    borrowed_to = "NestedAlternatives<TargetM | TargetN, TargetP | TargetQ> | NestedAlternatives<TargetO, TargetR> | NestedAlternatives<TargetS | TargetT, TargetU | TargetV>"
)]
struct NestedAlternatives<First, Second> {
    phantom_first: PhantomData<First>,
    phantom_second: PhantomData<Second>,
    data: String,
}

impl<First, Second> NestedAlternatives<First, Second> {
    fn new(data: &str) -> Self {
        Self {
            phantom_first: PhantomData,
            phantom_second: PhantomData,
            data: data.to_string(),
        }
    }

    fn get_data(&self) -> &str {
        &self.data
    }
}

#[test]
fn test_nested_parameter_and_top_level_alternatives() {
    // FROM side generates: 2x2 + 1 = 5 source types
    // TO side generates: 2x2 + 1 + 2x2 = 9 target types
    // Total coercion impls: 5 * 9 = 45 implementations

    // Test from SourceA, SourceX (first from alternative, first param combo)
    let ax = NestedAlternatives::<SourceA, SourceX>::new("ax");

    // To first top-level target alternative (2x2=4 combinations)
    let _: &NestedAlternatives<TargetM, TargetP> = ax.coerce();
    let _: &NestedAlternatives<TargetM, TargetQ> = ax.coerce();
    let _: &NestedAlternatives<TargetN, TargetP> = ax.coerce();
    let _: &NestedAlternatives<TargetN, TargetQ> = ax.coerce();

    // To second top-level target alternative (1 combination)
    let _: &NestedAlternatives<TargetO, TargetR> = ax.coerce();

    // To third top-level target alternative (2x2=4 combinations)
    let _: &NestedAlternatives<TargetS, TargetU> = ax.coerce();
    let _: &NestedAlternatives<TargetS, TargetV> = ax.coerce();
    let _: &NestedAlternatives<TargetT, TargetU> = ax.coerce();
    let _: &NestedAlternatives<TargetT, TargetV> = ax.coerce();

    // Test from SourceA, SourceY (first from alternative, second param combo)
    let ay = NestedAlternatives::<SourceA, SourceY>::new("ay");
    let _: &NestedAlternatives<TargetM, TargetP> = ay.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = ay.coerce();
    let _: &NestedAlternatives<TargetT, TargetV> = ay.coerce();

    // Test from SourceB, SourceX (first from alternative, third param combo)
    let bx = NestedAlternatives::<SourceB, SourceX>::new("bx");
    let _: &NestedAlternatives<TargetN, TargetQ> = bx.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = bx.coerce();
    let _: &NestedAlternatives<TargetS, TargetU> = bx.coerce();

    // Test from SourceB, SourceY (first from alternative, fourth param combo)
    let by = NestedAlternatives::<SourceB, SourceY>::new("by");
    let _: &NestedAlternatives<TargetM, TargetP> = by.coerce();
    let _: &NestedAlternatives<TargetM, TargetQ> = by.coerce();
    let _: &NestedAlternatives<TargetS, TargetV> = by.coerce();

    // Test from SourceC, SourceZ (second from alternative)
    let cz = NestedAlternatives::<SourceC, SourceZ>::new("cz");
    let _: &NestedAlternatives<TargetM, TargetP> = cz.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = cz.coerce();
    let _: &NestedAlternatives<TargetT, TargetU> = cz.coerce();
    let _: &NestedAlternatives<TargetT, TargetV> = cz.coerce();
}
