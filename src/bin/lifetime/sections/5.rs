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
	// construct_borrowed();
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
// 5.7 STRUCT LIFETIME CONSTRAINTS
// -----------------------------------------------------------------------------
//
// This is where the syntax becomes important.
//
// Consider:
//
//     struct Foo<'a> {
//         value: &'a i32,
//     }
//
// The relationship is:
//
//     Foo<'a>
//        │
//        └── contains &'a i32
//
// Therefore:
//
//     the referenced i32 must remain valid for `'a`.
//
//
//
// A lifetime parameter can also be constrained:
//
//     'a: 'b
//
// Read this as:
//
//     "'a outlives 'b"
//
// Meaning:
//
//     'a lasts at least as long as 'b.
//
//
//
// For example:
//
//     struct Foo<'a, 'b>
//     where
//         'a: 'b,
//     {
//         x: &'a i32,
//         y: &'b i32,
//     }
//
// This says:
//
//     'a
//     ├───────────────────────────────┐
//     │                               │
//     │      'b                       │
//     │      ├───────────────┤        │
//     │      │               │        │
//     └──────┴───────────────┴────────┘
//
// `'a` must outlive `'b`.
//
//
// The constraint does NOT mean that the references are the same.
// It only establishes an ordering between their lifetimes.

// -----------------------------------------------------------------------------
// 5.8 CONSTRAINTS IN A STRUCT
// -----------------------------------------------------------------------------
//
// We can therefore think of this:
//
//     struct Foo<'a, 'b>
//     where
//         'a: 'b,
//     {
//         x: &'a i32,
//         y: &'b i32,
//     }
//
// as saying:
//
//     x ──borrowed for──> 'a
//     y ──borrowed for──> 'b
//
//     'a ────────────────> 'b
//          outlives
//
// This gives the compiler enough information to reason about
// which reference remains valid for longer.

// -----------------------------------------------------------------------------
// 5.9 CONSTRAINTS CAN ALSO APPEAR INLINE
// -----------------------------------------------------------------------------
//
// Lifetime bounds can be written in a `where` clause or directly
// on the parameter list.
//
// These describe the same relationship:
//
//     fn example<'a, 'b>(x: &'a i32, y: &'b i32)
//     where
//         'a: 'b
//     {
//     }
//
//
//
// and:
//
//     fn example<'a: 'b, 'b>(x: &'a i32, y: &'b i32)
//     {
//     }
//
// The `where` form is often easier to read once constraints become
// more complicated.

// -----------------------------------------------------------------------------
// 5.10 WHAT THE CONSTRAINT ACTUALLY PROTECTS
// -----------------------------------------------------------------------------
//
// The purpose of these constraints is always the same:
//
//     prevent a stored reference from becoming dangling.
//
// For example, this cannot be allowed:
//
//     let borrowed;
//
//     {
//         let x = 10;
//         borrowed = Borrowed(&x);
//     }
//
// `x` would be dropped here.
//
// If `borrowed` remained usable afterward:
//
//     borrowed
//         │
//         ▼
//        &x
//         │
//         ▼
//      dropped x
//
// we would have a dangling reference.
//
// Rust rejects this situation.

// -----------------------------------------------------------------------------
// 5.11 ENUMS CAN ALSO CONTAIN REFERENCES
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
// 5.12 STRUCTS AND ENUMS IN PRACTICE
// -----------------------------------------------------------------------------

fn struct_and_enum_example() {
	let x = 10;
	let y = 20;

	let single = Borrowing(&x);

	let double = NamedBorrowed {
		x: &x,
		y: &y,
	};

	let independent = IndependentlyBorrowed {
		x: &x,
		y: &y,
	};

	let reference = Either::Ref(&x);
	let number = Either::Num(y);

	println!("single: {:?}", single);
	println!("double: {:?}", double);
	println!("independent: {:?}", independent);
	println!("reference: {:?}", reference);
	println!("number: {:?}", number);
}

// -----------------------------------------------------------------------------
// 5.13 STRUCT LIFETIME VS VALUE LIFETIME
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
// 5.14 MUTATING DATA WHILE A STRUCT BORROWS IT
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
// 5.15 EXPLICITLY DROPPING A BORROWED STRUCT
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
// 5.16 SELF-REFERENTIAL STRUCTS
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
// 5.17 WHY SELF-REFERENTIAL STRUCTS ARE DIFFICULT
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
// 5.18 THE BIG STRUCT LIFETIME RULE
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
