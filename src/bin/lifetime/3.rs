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
// Values exist for some duration.
// References have lifetimes describing how long they are valid
// to use while referring to those values.
//
// Lifetime annotations (`'a`) are used to describe the lifetime
// of a reference and, more importantly, relationships between
// references and the data they refer to.
//
// The compiler uses lifetimes to ensure that a reference never
// outlives the data it refers to.
//
// Most lifetimes are inferred automatically. Explicit lifetime
// annotations are primarily needed when we need to describe a
// relationship between multiple references.
//
// Topics:
// - What a lifetime represents
// - Value lifetime vs reference lifetime
// - Reference validity
// - Lifetime annotations (`'a`)
// - Lifetime parameters
// - Lifetime elision
// - Lifetime relationships
// - Returning references
// - Mutable references
// - Outliving (`'a: 'b`)
// - `'static`

pub fn lifetimes() {
	value_lifetime();
	lifetime_params();
	multi_lifetime_params();
	returning_refs();
	invalid_refs();
	return_of_owned_value();
}

// -----------------------------------------------------------------------------
// 3.1 VALUES AND REFERENCES HAVE DIFFERENT LIFETIME CONCEPTS
// -----------------------------------------------------------------------------
//
// A VALUE exists for some period of time.
//
// A REFERENCE is valid for some period of time while pointing at
// that value.
//
// For example:
//
//     let x = 7;
//         │
//         └── value exists until `x` is dropped
//
//     let r = &x;
//         │
//         └── reference is valid only while `x` remains valid
//             and the borrow is permitted
//
// The compiler's lifetime system is primarily concerned with
// proving that references never outlive the data they reference.
//
// Lifetime annotations (`'a`) describe reference lifetimes and
// relationships between them. They do not control when a value
// is dropped or extend the lifetime of a value.

fn value_lifetime() {
	let x = 7;

	{
		let y = 9;

		println!("{x}");
		println!("{y}");

		// `y` is still alive here.
	}

	// `y` no longer exists.
	// println!("{y}"); // ❌ `y` was dropped

	// `x` is still alive because its scope has not ended.
	println!("{x}");
}

// -----------------------------------------------------------------------------
// 3.2 A REFERENCE HAS A LIFETIME
// -----------------------------------------------------------------------------
//
// A reference can only be used while the value it references is valid.
//
// The reference below borrows `x`.
//
// The compiler ensures that the borrow does not outlive `x`.

fn reference_lifetime() {
	let x = 7;

	let reference = &x;

	println!("{reference}");

	// `reference` cannot remain valid after `x` is dropped.
}

// -----------------------------------------------------------------------------
// 3.3 LIFETIME ANNOTATIONS
// -----------------------------------------------------------------------------
//
// `'a` is a lifetime parameter.
//
// It does NOT create a lifetime.
// It names a lifetime that already exists.
//
// This says:
//
//   "The reference `x` is valid for some lifetime `'a`."
//
fn lifetime_params() {
	fn print_one<'a>(x: &'a i32) {
		println!("`print_one`: x is {}", x);
	}

	// In this simple case, the explicit lifetime is unnecessary.
	//
	// Rust's lifetime-elision rules allow us to write:

	fn print_one_elided(x: &i32) {
		println!("`print_one_elided`: x is {}", x);
	}

	// x is a value which has a lifetime, the scope of this function
	//
	let x = 1;

	// These are equivalent from the caller's perspective:
	print_one(&x);
	print_one_elided(&x);
	//
	// The explicit `'a` becomes important when we need to describe
	// relationships between multiple references.
}

// -----------------------------------------------------------------------------
// 3.4 MULTIPLE REFERENCES
// -----------------------------------------------------------------------------
//
// Different references can have different lifetimes.
//
// `'a` and `'b` are independent lifetime parameters.

fn multi_lifetime_params() {
	fn print_multi<'a, 'b>(x: &'a i32, y: &'b i32) {
		// There is no requirement here that `'a` and `'b` be the same.
		//
		// Each reference only needs to remain valid for the duration
		// in which the function uses it.
		println!("`print_multi`: x is {}, y is {}", x, y);
	}

	let x = 1;
	let y = 10;

	print_multi(&x, &y);
}

// -----------------------------------------------------------------------------
// 3.5 RETURNING A BORROWED REFERENCE
// -----------------------------------------------------------------------------
//
// When a function returns a reference, Rust needs to know which
// input reference the returned reference is associated with.
//
// Here, the returned reference comes from `x`.

fn returning_refs() {
	fn pass_x<'a, 'b>(x: &'a i32, y: &'b f64) -> &'a i32 {
		x
	}

	let x = 1;
	let y = 3.12;
	let z = pass_x(&x, &y);

	println!("Notice what the type of z is {}", z)
	// The important relationship is:
	//
	//     x: &'a i32
	//          │
	//          └──────────────┐
	//                         ↓
	//                  return: &'a i32
	//
	// The returned reference has the same lifetime relationship as `x`.
	//
	// Therefore the caller cannot use the returned reference longer
	// than `x` remains valid.
}

// -----------------------------------------------------------------------------
// 3.6 LIFETIME ELISION
// -----------------------------------------------------------------------------
//
// Rust can infer certain lifetime relationships automatically.
//

fn lifetime_elision() {
	// This:

	// fn first_elided(x: &i32, _: &i32) -> &i32 {
	// 	x
	// }

	// cannot be written this way:
	//
	// fn first(x: &i32, y: &i32) -> &i32 {
	// 	x
	// }
	//
	//
	// We therefore need to make the relationship explicit:
	fn first<'a, 'b>(x: &'a i32, _: &'b i32) -> &'a i32 {
		x
	}

	let x = 1;
	let y = 10;
	first(&x, &y);

	// This doesn't work either
	//
	// Produces
	//  "missing lifetime specifier"
	//
	// because there are multiple input references and Rust cannot
	// determine which input the returned reference belongs to.
	//
	// this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
	fn second(x: &i32, y: &f64) -> &i32 {
		x
	}

	let x = 1;
	let y = 3.12;
	second(&x, &y);
}

// -----------------------------------------------------------------------------
// 3.7 THE BORROW CANNOT OUTLIVE THE VALUE
// -----------------------------------------------------------------------------
//
// This is the fundamental lifetime rule:
//
//     reference lifetime <= referenced value lifetime
//
// The following would be invalid:

fn invalid_refs() {
	// `x` is dropped when this fn returns.
	//
	// Returning `&x` would therefore return a reference to data
	// that no longer exists.
	//
	// Rust rejects this rather than allowing a dangling reference.
	// fn invalid_reference() -> &i32 {
	// 	// consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`: `'static `rustcE0106
	// 	let x = 7;
	// 	&x
	// }
	// invalid_reference();
	// fn invalid_reference2() -> &String {
	// 	// consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`: `'static `rustcE0106
	// 	let x = String::from("Hello string");
	// 	&x
	// }
	// invalid_reference2();

	// `static` declares a value that exists for the entire lifetime
	// of the program.
	static NUMBER: i32 = 7;

	fn return_valid_ref() -> &'static i32 {
		// `fn_value` is a reference to the `static` value `NUMBER`.
		//
		// The reference itself is local to this function.
		let fn_value = &NUMBER;

		// `fn_value` goes out of scope here, so the local reference
		// is no longer available.
		//
		// The `NUMBER` it points to remains alive for the entire
		// lifetime of the program and by extension this function.
		// So it fixes the previous errors.
		&fn_value
	}

	// The returned reference points directly to `NUMBER`, which
	// still exists after `get_number()` returns.
	let static_typed_value = return_valid_ref();

	println!("static_typed_value {}", static_typed_value);
}

// -----------------------------------------------------------------------------
// 3.8 OWNERSHIP SOLVES THE PROBLEM
// -----------------------------------------------------------------------------
//

fn return_of_owned_value() {
	// If the caller needs the data after the function returns,
	// return ownership instead of a reference.
	//
	// The returned String belongs to the caller.
	fn valid_output() -> String {
		String::from("foo")
	}

	let new_value = valid_output();
	// No lifetime annotation is necessary.
	//
	// The String is moved to the caller, so its lifetime is now
	// controlled by whoever owns it.
	//
	// This is fundamentally different from:
	//
	// fn invalid_output<'a>() -> &'a String
	//
	// which would attempt to return a reference to locally-owned data.
}

// -----------------------------------------------------------------------------
// 3.9 MUTABLE REFERENCES HAVE LIFETIMES TOO
// -----------------------------------------------------------------------------
//
// Mutable references follow the same lifetime rules.
//
// While `x` is mutably borrowed, the mutable reference has
// exclusive access to the value.

fn add_one<'a>(x: &'a mut i32) {
	*x += 1;
}

// Again, the explicit lifetime is unnecessary here:
//
// fn add_one(x: &mut i32) {
//     *x += 1;
// }
//
// The important concept is not the annotation itself.
//
// The important concept is:
//
//     &mut T
//       │
//       └── exclusive access for the duration of the borrow

// -----------------------------------------------------------------------------
// 3.10 BORROWING ENDS WHEN THE REFERENCE IS NO LONGER USED
// -----------------------------------------------------------------------------
//
// Modern Rust uses non-lexical lifetimes.
//
// A borrow does not necessarily last until the end of the
// surrounding scope. It can end when the reference is no longer used.

fn borrow_ends_when_done() {
	let mut x = 7;

	let reference = &x;

	println!("{reference}");

	// The immutable borrow is no longer needed here.

	let mutable_reference = &mut x;
	*mutable_reference += 1;

	println!("{x}");
}

// -----------------------------------------------------------------------------
// 3.11 MUTABLE + IMMUTABLE BORROWS
// -----------------------------------------------------------------------------
//
// You cannot have an active mutable borrow while another
// reference is still being used.

fn borrowing_rules() {
	let mut x = 7;

	let a = &x;
	println!("{a}");

	// `a` is no longer used after this point.

	let b = &mut x;
	*b += 1;

	println!("{x}");
}

// The key rule:
//
//     Many immutable references
//             OR
//     One mutable reference
//
// But not simultaneously when their lifetimes overlap.

// -----------------------------------------------------------------------------
// 3.12 OUTLIVING
// -----------------------------------------------------------------------------
//
// `'a: 'b` means:
//
//     lifetime `'a` outlives lifetime `'b`.
//
// In other words, anything valid for `'b` is also valid for `'a`.
//
// Read:
//
//     'a: 'b
//
// as:
//
//     "'a` lives at least as long as `'b`."

fn outlives<'a, 'b>(x: &'a i32, _: &'b i32) where 'a: 'b {
	println!("{x}");
}

// -----------------------------------------------------------------------------
// 3.13 `'static`
// -----------------------------------------------------------------------------
//
// `'static` means a reference can remain valid for the entire
// duration of the program.
//
// String literals are a common example:

fn static_reference() {
	let city: &'static str = "London";

	println!("{city}");
}

// The string literal is embedded in the program's binary,
// so it is available for the entire program lifetime.
//
// `'static` does NOT mean:
//
//     "this value must be global"
//
// It means:
//
//     "this reference is valid for the entire program."

// -----------------------------------------------------------------------------
// 3.14 STATIC BOUND VS STATIC REFERENCE
// -----------------------------------------------------------------------------
//
// These are related but different ideas.
//
// &'static str
//
// means:
//
//     "a reference to data that lives for the entire program."
//
// T: 'static
//
// means:
//
//     "T contains no borrowed data that expires before the
//      entire program."
//
// An owned String satisfies `T: 'static` even though the String
// itself is eventually dropped:
//
//     String: 'static
//
// because it owns its data rather than borrowing something short-lived.

// -----------------------------------------------------------------------------
// 3.15 THE BIG PICTURE
// -----------------------------------------------------------------------------
//
// Ownership:
//
//     Who owns the data?
//
// Borrowing:
//
//     Who may access the data without owning it?
//
// Lifetime:
//
//     How long is that access guaranteed to remain valid?
//
// Move:
//
//     Transfer ownership.
//
// Clone:
//
//     Create another owned value.
//
// Copy:
//
//     Implicitly duplicate a value when the type permits it.
//
// Return ownership:
//
//     Give the caller control of the value's lifetime.
//
// Return a reference:
//
//     Keep ownership elsewhere and describe how long the
//     caller may safely access the borrowed data.
//
//
//
// The fundamental lifetime invariant:
//
//     A reference must never outlive the data it references.
//
//
// Most of the time Rust infers the lifetimes for us.
//
// Explicit lifetime annotations become important when we need
// to communicate relationships such as:
//
//     "this returned reference comes from this input"
//
//     "this reference must outlive that reference"
//
//     "this struct contains data borrowed for this lifetime"
