// ## 5. Structs & Lifetimes
//
// Structs make lifetimes more interesting because a reference can be
// stored rather than used immediately.
//
// Topics:
//
// - References inside structs
// - Struct lifetime parameters
// - `struct Foo<'a>`
// - Lifetime constraints between fields
// - Struct construction
// - Different lifetimes in one struct
// - Enums containing references
// - Self-referential structs
// - Why self-references are difficult

use crate::*;

pub fn structs_with_lifetimes() {
	section!("5. Structs & Lifetimes");
	struct_with_borrow();
	tuple_struct();
	construct_borrowed();
	one_struct_lifetime();
	multiple_struct_lifetimes();
	lifetime_outlives();
	lifetime_constraint_direction();
	constrained_struct();
	constraint_syntax();
}

// -----------------------------------------------------------------------------
// 5.1 REFERENCES INSIDE STRUCTS (Named field)
// -----------------------------------------------------------------------------

fn struct_with_borrow() {
	// Structs hold data in (2) ways.
	// 1) Own it
	// 2) borrow it.
	//
	// A struct can contain a reference:
	//
	// `'a` is the lifetime parameter of the struct.
	//
	// It describes how long the reference stored inside the struct
	// is valid.
	//
	#[derive(Debug)]
	struct Borrowed<'a> {
		owned: String,
		borrowed: &'a i32,
	}
	// The important constraint is:
	//
	//     referenced data must outlive the borrow stored in the struct.
	//
	// In other words:
	//
	//     &'a i32
	//      │
	//      └── the i32 must remain valid for `'a`
	let x = 1;

	let r#struct = Borrowed {
		owned: String::from("owned"),
		borrowed: &x,
	};
	println!("struct_with_borrow {:?}", r#struct);

	// fn failure() {
	// 	let borrowed;
	// 	{
	// 		let x = 42;

	// 		borrowed = Borrowed {
	// 			owned: String::from("owned"),
	// 			borrowed: &x,
	// 		};
	// 	}
	// 	// error[E0597]: `x` does not live long enough
	// 	println!("{}", borrowed.borrowed); // ❌
	// }
	// // failure()
}

// -----------------------------------------------------------------------------
// 5.2 A STRUCT LIFETIME PARAMETER (Tuple)
//
// So the struct creates a distinct type around the underlying value.
// -----------------------------------------------------------------------------
//
// This:
//
//     struct Borrowed<'a>(&'a i32);
//
// can be read as:
//
//     "Borrowed contains a reference to an i32,
//      and that reference is valid for lifetime `'a`."
//
// `'a` is NOT a field.
//
// It is a parameter used to describe the lifetime of the
// reference stored by the struct.
#[derive(Debug)]
struct Borrowing<'a>(&'a i32);

fn tuple_struct() {
	let x = 42;

	let borrowed = Borrowing(&x);

	println!("tuple_struct {}", borrowed.0);
}

// -----------------------------------------------------------------------------
// 5.3 CONSTRUCTING A BORROWED STRUCT
// -----------------------------------------------------------------------------

fn construct_borrowed() {
	let x = 10;

	// `x` lives in this scope.
	//
	// `borrowed` contains a reference to `x`.
	let borrowed = Borrowing(&x);

	println!("construct_borrowed {:?}", borrowed);

	// This is valid because `x` is still alive.
}

// When this function ends:
//
//     borrowed
//          ↓
//        &x
//          ↓
//          x
//
// `borrowed` is dropped before `x`.
//
// Therefore the reference never outlives the data it references.

// -----------------------------------------------------------------------------
// 5.4 THE CORE CONSTRAINT
// -----------------------------------------------------------------------------
//
// Consider:
//
//     let x = 10;
//     let borrowed = Borrowing(&x);
//
// There are two lifetimes involved:
//
//     x
//     ├──────────────────────────────┤
//     │                              │
//     │        borrowed              │
//     │        ├───────────────┤     │
//     │        │               │     │
//     └────────┴───────────────┴─────┘
//
// The lifetime of the reference stored in `borrowed`
// must fit within the lifetime of `x`.
//
// Conceptually:
//
//     lifetime of x
//            >=
//     lifetime required by &x
//
// This is the fundamental constraint behind references
// stored in structs.

// -----------------------------------------------------------------------------
// 5.5 MULTIPLE REFERENCES
// -----------------------------------------------------------------------------
//
// A struct can contain multiple references:
//
//     struct NamedBorrowed<'a> {
//         x: &'a i32,
//         y: &'a i32,
//     }
//
// Here both references use the same lifetime parameter `'a`.

#[derive(Debug)]
struct NamedBorrowed<'a> {
	x: &'a i32,
	y: &'a i32,
}

// This means:
//
//     x: &'a i32
//     y: &'a i32
//
// Both references are described as having lifetime `'a`.
//
// Therefore the struct requires both referenced values to remain
// valid for the lifetime represented by `'a`.
//
//
// IMPORTANT:
//
// Using the same lifetime parameter does NOT mean the two values
// were created at exactly the same time.
//
// It means the references are both valid for the lifetime `'a`
// required by this particular instance of the struct.

// -----------------------------------------------------------------------------
// 5.6 DIFFERENT LIFETIMES
// -----------------------------------------------------------------------------
//
// Sometimes the fields need independent lifetimes.
//
// We can express that with multiple lifetime parameters:

#[derive(Debug)]
struct IndependentlyBorrowed<'a, 'b> {
	x: &'a i32,
	y: &'b i32,
}

// Now:
//
//     x: &'a i32
//     y: &'b i32
//
// `'a` and `'b` are independent.
//
// There is no requirement that:
//
//     'a == 'b
//
// or:
//
//     'a: 'b
//
// or:
//
//     'b: 'a
//
// unless we explicitly introduce such a constraint.
//
//
// This is useful when the two references may have different
// valid durations.

// -----------------------------------------------------------------------------
// 5.7 STRUCT LIFETIME CONSTRAINTS: ONE LIFETIME
// -----------------------------------------------------------------------------
//
// Start with the simplest case:
//
//     struct Borrowed<'a> {
//         value: &'a i32,
//     }
//
// There is one lifetime:
//
//     'a
//
// The struct contains a reference that is valid for `'a`.
//
//
//     Borrowed<'a>
//          │
//          └── value: &'a i32
//
// The important relationship is:
//
//     reference lifetime
//            │
//            ▼
//           'a
//
// The referenced value must remain valid for as long as that
// reference is required to be valid.

fn one_struct_lifetime() {
	#[derive(Debug)]
	struct Borrowed<'a> {
		value: &'a i32,
	}

	let x = 42;

	let borrowed = Borrowed { value: &x };

	println!("borrowed: {:?}", borrowed);

	// `borrowed.value` borrows `x`.
	//
	// Therefore `x` must remain valid while this borrow is used.
	println!("value: {}", borrowed.value);
}

// -----------------------------------------------------------------------------
// 5.8 STRUCTS WITH MULTIPLE LIFETIMES
// -----------------------------------------------------------------------------
//
// A struct can contain references with DIFFERENT lifetimes.
//
//     struct Pair<'a, 'b> {
//         first: &'a i32,
//         second: &'b i32,
//     }
//
// This means:
//
//     first  ──> &'a i32
//     second ──> &'b i32
//
// `'a` and `'b` are independent.
//
// There is initially NO relationship saying:
//
//     'a == 'b'
//
// or:
//
//     'a: 'b
//
// or:
//
//     'b: 'a
//
// They are simply two separate lifetimes.

fn multiple_struct_lifetimes() {
	#[derive(Debug)]
	struct Pair<'a, 'b> {
		first: &'a i32,
		second: &'b i32,
	}

	let first = 10;
	let second = 20;

	let pair = Pair {
		first: &first,
		second: &second,
	};

	println!("pair: {:?}", pair);

	// `first` and `second` can have different lifetimes.
	//
	// The struct does not require one to outlive the other.
	println!("first: {}", pair.first);
	println!("second: {}", pair.second);
}

// -----------------------------------------------------------------------------
// 5.9 WHY WOULD WE NEED A LIFETIME CONSTRAINT?
// -----------------------------------------------------------------------------
//
// Now we have:
//
//     'a
//     'b
//
// But sometimes we need to tell Rust that one lifetime lasts
// at least as long as another.
//
// The syntax is:
//
//     'a: 'b
//
// Read it as:
//
//     "'a outlives 'b"
//
// Or:
//
//     "'a lasts at least as long as 'b."
//
//
//
// This gives us an ordering:
//
//     'a
//     ├─────────────────────────────┐
//     │                             │
//     │     'b                      │
//     │     ├──────────────┤        │
//     │     │              │        │
//     └─────┴──────────────┴────────┘
//
// `'a` is the longer lifetime.
// `'b` is the shorter lifetime.
//
// IMPORTANT:
//
//     'a: 'b
//
// does NOT:
//
//     - extend `'b`
//     - extend `'a`
//     - keep a value alive
//     - prevent a value from being dropped
//
// It only establishes a relationship between lifetimes that
// already exist.

fn lifetime_outlives() {
	// This function demonstrates the useful consequence of
	// an outlives relationship.
	//
	// If `'a` outlives `'b`, then a reference valid for `'a`
	// can also be used where a reference valid for `'b` is required.

	fn shorten<'a, 'b>(value: &'a i32) -> &'b i32
	where
		'a: 'b,
	{
		// `value` is valid for `'a`.
		//
		// `'a` is guaranteed to last at least as long as `'b`.
		//
		// Therefore it is safe to use this reference for `'b`.
		value
	}

	let long_lived = 42;

	{
		let result = shorten(&long_lived);

		println!("result: {result}");

		// Here Rust can choose a shorter `'b` for `result`.
		//
		// The important relationship is:
		//
		//     lifetime(long_lived)
		//              │
		//              │  'a
		//              │
		//              ├───────────────────────┐
		//              │                       │
		//              │       'b              │
		//              │       ├────────┤      │
		//              └───────┴────────┴──────┘
		//
		// Since `'a: 'b`, the reference valid for `'a` is also
		// valid for the shorter `'b`.
	}

	// `long_lived` is still alive here.
	println!("long_lived: {long_lived}");
}

// -----------------------------------------------------------------------------
// 5.10 THE DIRECTION OF THE CONSTRAINT
// -----------------------------------------------------------------------------
//
// The direction is important.
//
//     'a: 'b
//
// means:
//
//     'a OUTLIVES 'b
//
// Think:
//
//     LONGER
//       │
//       ▼
//      'a
//       │
//       │ outlives
//       ▼
//      'b
//     SHORTER
//
// Therefore:
//
//     &'a T
//
// can safely be used where:
//
//     &'b T
//
// is required.
//
// But the reverse is NOT guaranteed.
//
// A reference that is only valid for `'b` cannot automatically
// be treated as valid for the longer `'a`.
//
//
//
// This is the core reason lifetime constraints exist:
//
//     LONGER lifetime
//            │
//            │ can satisfy
//            ▼
//     SHORTER lifetime
//
// but:
//
//     SHORTER lifetime
//            │
//            X cannot satisfy
//            ▼
//     LONGER lifetime

fn lifetime_constraint_direction() {
	fn use_shorter<'a, 'b>(value: &'a i32)
	where
		'a: 'b,
	{
		// `'a` is guaranteed to outlive `'b`.
		//
		// Therefore the `'a` reference can be used for `'b`.
		let shorter: &'b i32 = value;

		println!("shorter: {shorter}");
	}

	let value = 100;

	use_shorter(&value);

	println!("value: {value}");
}

// -----------------------------------------------------------------------------
// 5.11 APPLYING THE CONSTRAINT TO A STRUCT
// -----------------------------------------------------------------------------
//
// Now bring the same idea back to structs.
//
// We can write:
//
//     struct Borrowed<'a, 'b>
//     where
//         'a: 'b,
//     {
//         long: &'a i32,
//         short: &'b i32,
//     }
//
// There are now TWO independent references:
//
//     long  ──> &'a i32
//     short ──> &'b i32
//
// PLUS one relationship:
//
//     'a: 'b
//
// So the complete meaning is:
//
//     "Borrowed contains one reference valid for `'a`,
//      another reference valid for `'b`,
//      and `'a` is guaranteed to outlive `'b`."
//
//
//
// This:
//
//     'a: 'b
//
// does NOT mean:
//
//     long == short
//
// and it does NOT mean:
//
//     both references have the same lifetime.
//
// It only means:
//
//     'a is at least as long as 'b.
//
//
//
// The progression is:
//
//     1. &'a T
//        "this reference has lifetime `'a`"
//
//     2. &'a T + &'b T
//        "these references have potentially different lifetimes"
//
//     3. 'a: 'b
//        "we additionally know that `'a` lasts at least as long
//         as `'b`."

fn constrained_struct() {
	#[derive(Debug)]
	struct Borrowed<'a, 'b>
	where
		'a: 'b,
	{
		long: &'a i32,
		short: &'b i32,
	}

	let long_lived = 10;

	{
		let short_lived = 20;

		let borrowed = Borrowed {
			long: &long_lived,
			short: &short_lived,
		};

		println!("borrowed: {:?}", borrowed);

		println!("long: {}", borrowed.long);
		println!("short: {}", borrowed.short);

		// At this point:
		//
		//     long_lived
		//         │
		//         │
		//         ▼
		//     borrowed.long
		//
		//     short_lived
		//         │
		//         │
		//         ▼
		//     borrowed.short
		//
		// `'a` is constrained to outlive `'b`.
		//
		// The important thing is that the compiler now has an
		// ordering relationship it can use when reasoning about
		// these references.
	}

	// `short_lived` is gone here.
	//
	// But `long_lived` is still alive.
	println!("long_lived: {long_lived}");
}

// -----------------------------------------------------------------------------
// 5.12 CONSTRAINT SYNTAX
// -----------------------------------------------------------------------------
//
// The same lifetime relationship can be written in two places.
//
// `where` clause:
//
//     struct Foo<'a, 'b>
//     where
//         'a: 'b,
//     {
//         x: &'a i32,
//         y: &'b i32,
//     }
//
// Or inline:
//
//     struct Foo<'a: 'b, 'b> {
//         x: &'a i32,
//         y: &'b i32,
//     }
//
// Both mean:
//
//     'a: 'b
//
// The `where` form is often easier to read when there are
// multiple or complicated constraints.

fn constraint_syntax() {
	#[derive(Debug)]
	struct WhereForm<'a, 'b>
	where
		'a: 'b,
	{
		long: &'a i32,
		short: &'b i32,
	}

	#[derive(Debug)]
	struct InlineForm<'a: 'b, 'b> {
		long: &'a i32,
		short: &'b i32,
	}

	let x = 1;
	let y = 2;

	let where_form = WhereForm {
		long: &x,
		short: &y,
	};

	let inline_form = InlineForm {
		long: &x,
		short: &y,
	};

	println!("where form: {:?}", where_form);
	println!("inline form: {:?}", inline_form);
}

// -----------------------------------------------------------------------------
// 5.13 ENUMS CAN ALSO CONTAIN REFERENCES
// -----------------------------------------------------------------------------
//
// The same lifetime rules apply to enums.
//
// This enum can contain either:
//
//     - an owned i32
//     - a reference to an i32
//
// Because one variant contains a reference, the enum needs a
// lifetime parameter describing that reference.

#[derive(Debug)]
enum Either<'a> {
	Num(i32),
	Ref(&'a i32),
}

// `Num` does not use `'a`.
//
// `Ref` does:
//
//     Ref(&'a i32)
//
// The enum therefore carries `'a` so that Rust can track the
// validity of the reference when the `Ref` variant is used.

// -----------------------------------------------------------------------------
// 5.14 STRUCTS AND ENUMS IN PRACTICE
// -----------------------------------------------------------------------------

fn struct_and_enum_example() {
	let x = 10;
	let y = 20;

	let single = Borrowing(&x);

	let double = NamedBorrowed { x: &x, y: &y };

	let independent = IndependentlyBorrowed { x: &x, y: &y };

	let reference = Either::Ref(&x);
	let number = Either::Num(y);

	println!("single: {:?}", single);
	println!("double: {:?}", double);
	println!("independent: {:?}", independent);
	println!("reference: {:?}", reference);
	println!("number: {:?}", number);
}

// -----------------------------------------------------------------------------
// 5.15 STRUCT LIFETIME VS VALUE LIFETIME
// -----------------------------------------------------------------------------
//
// It is useful to distinguish:
//
//     x
//     │
//     └── actual value
//
// from:
//
//     borrowed
//     │
//     └── struct containing a reference to x
//
// The struct does NOT own `x`.
//
// It only contains a reference to `x`.
//
// Therefore:
//
//     dropping the struct
//
// does NOT:
//
//     drop x
//
// And:
//
//     dropping x
//
// cannot happen while the struct still requires the reference.
//
// The borrow checker enforces that relationship.

// -----------------------------------------------------------------------------
// 5.16 MUTATING DATA WHILE A STRUCT BORROWS IT
// -----------------------------------------------------------------------------
//
// A struct containing a reference keeps that borrow alive for
// as long as the reference is considered active.
//
// Therefore this can prevent mutation:
//
//     let mut y = 20;
//
//     let borrowed = NamedBorrowed {
//         x: &x,
//         y: &y,
//     };
//
//     y += 10;
//
// Depending on whether `borrowed` is still used, Rust may reject
// the mutation because `borrowed.y` is an immutable reference to `y`.
//
// If the struct is no longer needed, its borrow can end.

fn mutation_after_borrow() {
	let mut y = 20;

	let borrowed = Borrowing(&y);

	println!("{:?}", borrowed);

	// `borrowed` is no longer used after this point.
	//
	// Its borrow can end here.

	y += 10;

	println!("y = {}", y);
}

// -----------------------------------------------------------------------------
// 5.17 EXPLICITLY DROPPING A BORROWED STRUCT
// -----------------------------------------------------------------------------
//
// `drop` can be used to consume the struct and end its ownership.
//
// This can make the contained borrow unavailable:
//
//     let borrowed = Borrowed(&y);
//
//     drop(borrowed);
//
//     y += 10;
//
// However, modern Rust's non-lexical lifetime analysis often
// recognizes automatically when a borrow is no longer used,
// so explicit `drop` is usually unnecessary.

// -----------------------------------------------------------------------------
// 5.18 SELF-REFERENTIAL STRUCTS
// -----------------------------------------------------------------------------
//
// A much harder case is a struct that tries to contain:
//
//     1. owned data
//     2. a reference into that same owned data
//
// Conceptually:
//
//     struct SelfRef {
//         data: String,
//         reference: &str,   // points inside `data`
//     }
//
// The problem is that the struct owns `data`, while `reference`
// points into `data`.
//
// If the struct moves in memory, the location of `data` can change.
//
// The reference would then potentially point to the old location.
//
// Rust therefore makes ordinary self-referential structs
// intentionally difficult to construct safely.

// -----------------------------------------------------------------------------
// 5.19 WHY SELF-REFERENTIAL STRUCTS ARE DIFFICULT
// -----------------------------------------------------------------------------
//
// Imagine:
//
//     struct SelfRef {
//         data: String,
//         reference: &str,
//     }
//
// We want:
//
//     SelfRef
//     ┌──────────────────────────┐
//     │ data: "Hello"            │
//     │       ↑                  │
//     │       │                  │
//     │ reference ───────────────┘
//     └──────────────────────────┘
//
// But the struct itself may move.
//
// Before:
//
//     address A
//     ┌──────────────┐
//     │ data         │
//     │ reference ───┼──→ data
//     └──────────────┘
//
// After moving the struct:
//
//     address B
//     ┌──────────────┐
//     │ data         │
//     │ reference ───┼──→ address A ❌
//     └──────────────┘
//
// The reference could now point to the old location.
//
// This is one reason Rust distinguishes ordinary borrowing from
// more advanced concepts such as pinning.

// -----------------------------------------------------------------------------
// 5.20 THE BIG STRUCT LIFETIME RULE
// -----------------------------------------------------------------------------
//
// When a struct stores a reference:
//
//     struct Foo<'a> {
//         value: &'a T,
//     }
//
// think:
//
//     Foo<'a>
//         │
//         └── borrows T for `'a`
//
// The data being borrowed must remain valid for every point
// at which the struct may use that reference.
//
//
//
// With multiple references:
//
//     struct Foo<'a, 'b> {
//         x: &'a T,
//         y: &'b T,
//     }
//
// the references can have independent lifetimes.
//
//
//
// With:
//
//     'a: 'b
//
// we establish:
//
//     'a outlives 'b.
//
//
//
// The lifetime parameter does not extend either value's lifetime.
//
// It only describes a constraint that must already be true
// for the program to be valid.

// // 1. Unit struct
// struct Marker;

// // 2. Tuple struct
// struct Point(i32, i32);

// // 3. Named-field struct
// struct User {
//     name: String,
//     age: u32,
// }
