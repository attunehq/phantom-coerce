use phantom_coerce::Coerce;
use std::marker::PhantomData;

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
}

impl<First, Second> NestedAlternatives<First, Second> {
    fn new() -> Self {
        Self {
            phantom_first: PhantomData,
            phantom_second: PhantomData,
        }
    }
}

#[test]
fn nested_parameter_and_top_level_alternatives() {
    // FROM side generates: 2x2 + 1 = 5 source types
    // TO side generates: 2x2 + 1 + 2x2 = 9 target types
    // Total coercion impls: 5 * 9 = 45 implementations

    // Test from SourceA, SourceX (first from alternative, first param combo)
    let ax = NestedAlternatives::<SourceA, SourceX>::new();

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
    let ay = NestedAlternatives::<SourceA, SourceY>::new();
    let _: &NestedAlternatives<TargetM, TargetP> = ay.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = ay.coerce();
    let _: &NestedAlternatives<TargetT, TargetV> = ay.coerce();

    // Test from SourceB, SourceX (first from alternative, third param combo)
    let bx = NestedAlternatives::<SourceB, SourceX>::new();
    let _: &NestedAlternatives<TargetN, TargetQ> = bx.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = bx.coerce();
    let _: &NestedAlternatives<TargetS, TargetU> = bx.coerce();

    // Test from SourceB, SourceY (first from alternative, fourth param combo)
    let by = NestedAlternatives::<SourceB, SourceY>::new();
    let _: &NestedAlternatives<TargetM, TargetP> = by.coerce();
    let _: &NestedAlternatives<TargetM, TargetQ> = by.coerce();
    let _: &NestedAlternatives<TargetS, TargetV> = by.coerce();

    // Test from SourceC, SourceZ (second from alternative)
    let cz = NestedAlternatives::<SourceC, SourceZ>::new();
    let _: &NestedAlternatives<TargetM, TargetP> = cz.coerce();
    let _: &NestedAlternatives<TargetO, TargetR> = cz.coerce();
    let _: &NestedAlternatives<TargetT, TargetU> = cz.coerce();
    let _: &NestedAlternatives<TargetT, TargetV> = cz.coerce();
}
