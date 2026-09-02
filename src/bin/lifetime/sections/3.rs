// ## 3. Lifetimes
//
// A lifetime describes how long a reference is valid.
//
// Ownership answers:
//   "Who is responsible for this value?"
//
// Borrowing answers:
//   "Can I access this value without owning it?"
//
// Lifetimes answer:
//   "For how long is this reference guaranteed to remain valid?"
//

use crate::section;

pub fn lifetimes() {
	section!("3. Lifetimes");

	value_lifetime();
	reference_lifetime();
	lifetime_annotations();
	multiple_lifetimes();
	lifetime_elision();
	borrow_lifetimes();
	borrowing_rules();

	let (a, b) = (10, 100);
	outlives(&a, &b);

	static_lifetime();
}

// -----------------------------------------------------------------------------
// 3.1 SCOPE → VALUE LIFETIME
// -----------------------------------------------------------------------------
//
// For ordinary local values, scope gives us an easy way to visualize
// the lifetime of the value.
//
//     {
//         let x = 7;
//
//         // x is in scope
//         // x's value is alive
//
//     } // x goes out of scope and is dropped
//
// Therefore:
//
//     scope ends
//          ↓
//     owner goes away
//          ↓
//     value is dropped
//          ↓
//     value's lifetime ends
//
// Scope is not itself a lifetime, but it is one of the easiest ways
// to observe when a local value's lifetime ends.

fn value_lifetime() {
	let x = 7;

	{
		let y = 9;

		println!("x: {x}");
		println!("y: {y}");

		// Both values are alive here.
	}

	// `y` is no longer available.
	// println!("{y}"); // ❌

	// `x` is still alive because its scope has not ended.
	println!("x: {x}");

	// `x` is eventually dropped when this function's scope ends.
}

// -----------------------------------------------------------------------------
// 3.2 REFERENCE LIFETIME
// -----------------------------------------------------------------------------
//
// A reference does not own the value it points to.
//
//     let x = 7;
//     let r = &x;
//
//     x
//     │
//     │ owns
//     ▼
//     7
//     ▲
//     │
//     │ borrowed through
//     │
//     r
//
// The reference is only valid while the referenced value remains
// valid and the borrow is permitted.
//
// The fundamental lifetime rule:
//
//     A reference must never outlive the data it refers to.

fn reference_lifetime() {
	let x = 7;

	let reference = &x;

	println!("x: {x}");
	println!("reference: {reference}");

	// `reference` borrows `x`.
	//
	// `x` must remain valid for as long as `reference` is used.
}

// -----------------------------------------------------------------------------
// 3.3 LIFETIME ANNOTATIONS
// -----------------------------------------------------------------------------
//
// A lifetime annotation such as `'a`:
//
//     - does NOT create a lifetime
//     - does NOT extend a lifetime
//     - does NOT control when a value is dropped
//
// It gives defines something which already exists so that Rust can describe a
// relationship involving references.
//
// For example:
//
//     &'a i32
//
// can be read as:
//
//     "a reference to an i32 that is valid for lifetime `'a`."
//
// Think of `'a` as a LABEL for a lifetime, not a duration that
// we manually choose.
//
// The compiler determines the actual lifetime.
//
// The annotation allows us to talk about that lifetime.

fn lifetime_annotations() {
	fn inspect<'a>(value: &'a i32) {
		println!("value: {value}");
	}

	let x = 42;

	inspect(&x);

	// `'a` did not make `x` live longer.
	//
	// It describes something which is elided in other languages
}

// -----------------------------------------------------------------------------
// 3.4 MULTIPLE LIFETIMES
// -----------------------------------------------------------------------------
//
// Different references can have different lifetimes.
//
//     'a
//     'b
//
// are simply different lifetime parameters.
//
// There is no requirement that:
//
//     'a == 'b
//
// unless a relationship explicitly requires it.
//
// This function accepts two references with independent lifetimes.
//
//     &'a i32
//     &'b i32
//
// means:
//
//     "x has lifetime `'a`."
//     "y has lifetime `'b`."

fn multiple_lifetimes() {
	fn inspect<'a, 'b>(x: &'a i32, y: &'b i32) {
		println!("x: {x}");
		println!("y: {y}");
	}

	let x = 1;
	let y = 2;

	inspect(&x, &y);

	// There is no requirement that `x` and `y` have identical
	// lifetimes.
	//
	// Each reference has its own lifetime.
}

// -----------------------------------------------------------------------------
// 3.5 LIFETIME ELISION
// -----------------------------------------------------------------------------
//
// Rust can infer many lifetime annotations automatically.
//
// This:
//
//     fn inspect<'a>(value: &'a i32) {
//         println!("{value}");
//     }
//
// can be written:
//
//     fn inspect(value: &i32) {
//         println!("{value}");
//     }
//
// The lifetime still exists.
//
// We simply don't have to write the annotation because Rust can
// determine the relationship from the function signature.
//
// Lifetime elision is therefore:
//
//     "omit lifetime syntax when the compiler can infer it."
//
// It does NOT mean the reference has no lifetime.

fn lifetime_elision() {
	fn inspect(value: &i32) {
		println!("value: {value}");
	}

	let x = 42;

	inspect(&x);

	// `&i32` still has a lifetime.
	//
	// Rust simply inferred it for us.
}

// -----------------------------------------------------------------------------
// 3.6 BORROW LIFETIMES
// -----------------------------------------------------------------------------
//
// A borrow has a lifetime describing the period during which
// that reference is considered active.
//
// Modern Rust uses Non-Lexical Lifetimes (NLL).
//
// This means a borrow can end when the reference is no longer
// used, rather than necessarily lasting until the end of the
// surrounding scope.
//
// For example:
//
//     let immutable = &x;
//     println!("{immutable}");
//
//     // immutable borrow ends here because it is no longer used.
//
//     let mutable = &mut x;
//
// The immutable and mutable borrows do not overlap.

fn borrow_lifetimes() {
	let mut x = 0;

	let immutable = &x;

	println!("immutable: {immutable}");

	// `immutable` is never used again.
	//
	// Its borrow can end here.

	let mutable = &mut x;

	*mutable += 1;

	println!("mutable: {mutable}");
}

// -----------------------------------------------------------------------------
// 3.7 BORROWING RULES
// -----------------------------------------------------------------------------
//
// Lifetime rules interact with Rust's borrowing rules.
//
// For a particular value, Rust allows:
//
//     MANY immutable references
//
// OR:
//
//     ONE mutable reference
//
// but not overlapping access of both kinds.
//
// In other words:
//
//     &T
//     &T
//     &T
//
// is allowed.
//
// But:
//
//     &mut T
//
// requires exclusive access for the duration of that mutable borrow.
//
// The lifetime of the borrow determines how long that restriction
// applies.

fn borrowing_rules() {
	let mut x = 7;

	let a = &x;
	let b = &x;

	println!("a: {a}");
	println!("b: {b}");

	// Both immutable borrows can coexist.
	//
	// They only read the value.

	let c = &mut x;

	*c += 1;

	println!("mutable: {c}");

	// `a` and `b` cannot be used while `c` is the active mutable
	// borrow.
	//
	// Uncommenting this would violate the borrowing rules:
	//
	// println!("{a}"); // ❌
}

// -----------------------------------------------------------------------------
// 3.8 OUTLIVES
// -----------------------------------------------------------------------------
//
// Lifetime relationships can be expressed using:
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
//     'a
//     ├──────────────────────┐
//     │                      │
//     │                      │
//     └──────────────┐       │
//                    │       │
//                    ▼       ▼
//                   'b      end
//
// `'a: 'b` therefore guarantees that anything requiring `'b`
// can safely use something valid for `'a`.
//
// This is a relationship between lifetimes.
// It does not create either lifetime.

fn outlives<'a, 'b>(x: &'a i32, y: &'b i32)
where
	'a: 'b,
{
	println!("x: {x}");
	println!("y: {y}");

	// `'a: 'b` means:
	//
	//     'a lasts at least as long as 'b.
}

// -----------------------------------------------------------------------------
// 3.9 `'STATIC`
// -----------------------------------------------------------------------------
//
// `'static` is a special lifetime.
//
// It means:
//
//     "this reference is valid for the entire duration of the program."
//
// String literals are a common example:
//
//     "hello"
//
// The string data is stored in the compiled program and therefore
// remains available for the entire program.
//
//
//     &'static str
//
// means:
//
//     "a reference to a str that is valid for the entire program."
//
// `'static` does NOT mean:
//
//     "the value can never be dropped."
//
// It describes the validity of a reference.

fn static_lifetime() {
	let city: &'static str = "London";

	println!("city: {city}");

	// The string literal remains valid for the entire program.
}

// -----------------------------------------------------------------------------
// 3.10 THE LIFETIME MODEL
// -----------------------------------------------------------------------------
//
// Ownership:
//
//     Who owns the value?
//
// Borrowing:
//
//     Who is temporarily allowed to access the value?
//
// Lifetime:
//
//     For how long is that reference guaranteed to remain valid?
//
//
//
// Ownership determines:
//
//     when data is destroyed.
//
// Borrowing determines:
//
//     who may access data without owning it.
//
// Lifetimes determine:
//
//     how long those references remain valid.
//
//
//
// The fundamental invariant is:
//
//     reference lifetime
//             <=
//     referenced data lifetime
//
//
// Lifetime annotations (`'a`) allow us to describe relationships
// between those reference lifetimes.
//
// They do not:
//
//     - create lifetimes
//     - extend lifetimes
//     - prevent values from being dropped
//     - make data live forever
//
// Rust normally infers lifetimes.
//
// Explicit annotations become necessary when Rust needs help
// understanding a relationship between references.

// SCOPE
//     Where a binding is accessible.

// LIFETIME
//     How long a value/reference is valid.

// DROP
//     The destruction event that can end a value's lifetime.
