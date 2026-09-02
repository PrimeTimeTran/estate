// ## 10. Lifetime Patterns
//
// - Borrowing a value
// - Sharing data
// - Temporarily borrowing
// - Returning borrowed data
// - Storing borrowed data
// - Owning data instead
// - Passing ownership across boundaries
pub fn ten() {
	{
		// Make a `string` literal and print it:
		let static_string = "I'm in read-only memory";
		println!("static_string: {}", static_string);

		// When `static_string` goes out of scope, the reference
		// can no longer be used, but the data remains in the binary.
	}

	{
		// Make an integer to use for `coerce_static`:
		let lifetime_num = 9;

		// Coerce `NUM` to lifetime of `lifetime_num`:
		let coerced_static = coerce_static(&lifetime_num);

		println!("coerced_static: {}", coerced_static);
	}

	println!("NUM: {} stays accessible!", NUM);
}

// // A reference with 'static lifetime:
// let s: &'static str = "hello world";

// // 'static as part of a trait bound:
// fn generic<T>(x: T) where T: 'static {}

// Make a constant with `'static` lifetime.
static NUM: i32 = 18;

// Returns a reference to `NUM` where its `'static`
// lifetime is coerced to that of the input argument.
fn coerce_static<'a>(_: &'a i32) -> &'a i32 {
	&NUM
}

// 2.
// extern crate rand;
// use rand::Fill;

// fn random_vec() -> &'static [u64; 100] {
//     let mut rng = rand::rng();
//     let mut boxed = Box::new([0; 100]);
//     boxed.fill(&mut rng);
//     Box::leak(boxed)
// }

// fn main() {
//     let first: &'static [u64; 100] = random_vec();
//     let second: &'static [u64; 100] = random_vec();
//     assert_ne!(first, second)
// }

// // 3.
// use std::fmt::Debug;

// fn print_it(input: impl Debug + 'static) {
//     println!("'static value passed in is: {:?}", input);
// }

// fn main() {
//     // i is owned and contains no references, thus it's 'static:
//     let i = 5;
//     print_it(i);

//     // oops, &i only has the lifetime defined by the scope of
//     // main(), so it's not 'static:
//     print_it(&i);
// }
