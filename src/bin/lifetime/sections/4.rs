// ## 4. Functions
//
// - Borrowed parameters
// - Borrowed return values
// - Input/output lifetime relationships
// - Multiple references
// - Lifetime elision in functions
// - Returning references
// - Why some references cannot be returned

use crate::*;

pub fn functions() {
	section!("4. Functions");

	borrowed_parameter();
	borrowed_return_value();
	input_output_lifetime();
	multiple_references();
	lifetime_elision();
	returning_reference();
	cannot_return_local_reference();
}

// -----------------------------------------------------------------------------
// 4.1 BORROWED PARAMETERS
// -----------------------------------------------------------------------------

fn borrowed_parameter() {
	// A function can borrow a value instead of taking ownership of it.
	//
	// This:
	//
	//     value: &str
	//
	// means:
	//
	//     "I need temporary access to a str."
	//
	// The function does NOT own the string.
	//
	// The caller remains the owner:
	//
	//     caller
	//        │
	//        │ owns
	//        ▼
	//     String
	//        ▲
	//        │ borrowed
	//        │
	//     function
	//
	fn print(value: &str) {
		println!("value: {value}");
	}

	let value = String::from("hello");

	// `print` temporarily borrows `value`.
	print(&value);

	// We still own `value` after the function returns.
	println!("still owned: {value}");
}

// -----------------------------------------------------------------------------
// 4.2 BORROWED RETURN VALUES
// -----------------------------------------------------------------------------

fn borrowed_return_value() {
	// A function can also RETURN a reference.
	//
	// The returned reference does not own the data.
	//
	// Therefore, the data being referenced must remain alive
	// for as long as the returned reference is used.
	//
	// Here the function receives a reference and returns a reference.
	//
	// The lifetime relationship is:
	//
	//     input reference
	//          │
	//          │
	//          ▼
	//     ┌─────────┐
	//     │ function│
	//     └─────────┘
	//          │
	//          │ returned reference
	//          ▼
	//       caller
	//
	fn first_character(value: &str) -> &str {
		&value[..1]
	}

	let value = String::from("hello");

	let first = first_character(&value);

	println!("first: {first}");
}

// -----------------------------------------------------------------------------
// 4.3 INPUT / OUTPUT LIFETIME RELATIONSHIP
// -----------------------------------------------------------------------------

fn input_output_lifetime() {
	// When a function returns a reference, Rust needs to know:
	//
	//     "What does this returned reference borrow from?"
	//
	// Consider:
	//
	//     fn longer(a: &str, b: &str) -> &str
	//
	// There are TWO possible inputs the result could borrow from.
	//
	// The lifetime annotation makes the relationship explicit:
	//
	//     fn longer<'a>(a: &'a str, b: &'a str) -> &'a str
	//
	// This means:
	//
	//     "Both inputs are valid for `'a`,
	//      and the returned reference is also valid for `'a`."
	//
	fn longer<'a>(a: &'a str, b: &'a str) -> &'a str {
		if a.len() > b.len() { a } else { b }
	}

	let a = String::from("hello");
	let b = String::from("hello world");

	let result = longer(&a, &b);

	println!("longer: {result}");
}

// -----------------------------------------------------------------------------
// 4.4 MULTIPLE REFERENCES
// -----------------------------------------------------------------------------

fn multiple_references() {
	// Multiple references can have the SAME lifetime:
	//
	//     fn select<'a>(a: &'a str, b: &'a str) -> &'a str
	//
	// But that does NOT mean `a` and `b` actually live for the
	// same amount of time.
	//
	// `'a` represents the lifetime required by this particular
	// relationship.
	//
	// Rust chooses a lifetime that both references can satisfy.
	//
	fn select<'a>(a: &'a str, b: &'a str) -> &'a str {
		a
		// The function could return `b` instead.
		// Either way, the returned reference must be valid
		// for the lifetime promised by `'a`.
	}

	let first = String::from("first");
	let second = String::from("second");

	let selected = select(&first, &second);

	println!("selected: {selected}");
}

// -----------------------------------------------------------------------------
// 4.5 LIFETIME ELISION
// -----------------------------------------------------------------------------

fn lifetime_elision() {
	// Rust has rules that allow many lifetime annotations
	// to be omitted.
	//
	// For example, this:
	//
	//     fn first(value: &str) -> &str
	//
	// is understood by the compiler as if we had written:
	//
	//     fn first<'a>(value: &'a str) -> &'a str
	//
	// Because there is only ONE input reference, Rust can infer
	// that the returned reference comes from that input.
	//
	fn first(value: &str) -> &str {
		&value[..1]
	}

	let value = String::from("hello");

	let result = first(&value);

	println!("first: {result}");
}

// -----------------------------------------------------------------------------
// 4.6 RETURNING REFERENCES
// -----------------------------------------------------------------------------

fn returning_reference() {
	// A common pattern is:
	//
	//     input  -> borrow -> function -> return borrow
	//
	// The function does not create the data.
	// It selects something from data supplied by the caller.
	//
	// Therefore the returned reference can safely point back
	// into the caller's data.
	//
	fn first_word(value: &str) -> &str {
		value.split_whitespace().next().unwrap_or("")
	}

	let sentence = String::from("hello world");

	// `word` borrows from `sentence`.
	let word = first_word(&sentence);

	println!("word: {word}");

	// `sentence` must remain alive while `word` is used.
	println!("sentence: {sentence}");
}

// -----------------------------------------------------------------------------
// 4.7 WHY SOME REFERENCES CANNOT BE RETURNED
// -----------------------------------------------------------------------------

fn cannot_return_local_reference() {
	// A function CANNOT return a reference to data that the function
	// itself creates locally.
	//
	// This would be invalid:
	//
	//     fn invalid() -> &str {
	//         let value = String::from("hello");
	//
	//         &value
	//     }
	//
	// Why?
	//
	// `value` belongs to the function's scope.
	//
	//     invalid()
	//     │
	//     ├── value
	//     │     │
	//     │     └── String
	//     │
	//     └── function returns
	//           │
	//           ▼
	//         value is dropped
	//
	// The caller would receive a reference to something that
	// has already been destroyed.
	//
	// Rust therefore rejects the function before this can happen.
	//
	// ❌ DO NOT uncomment:
	//
	// fn invalid() -> &str {
	//     let value = String::from("hello");
	//     &value
	// }
	//
	// The solution is to return OWNED data instead:
	//
	fn valid() -> String {
		String::from("hello")
	}

	let value = valid();

	println!("owned result: {value}");
}
