// ## 3. Lifetimes
// - What a lifetime actually represents
// - Lifetime of a value vs lifetime of a reference
// - Reference validity
// - Lifetime annotations (`'a`)
// - Lifetime parameters
// - Elision rules
// - Lifetime constraints
// - Outliving (`'a: 'b`)
// - `'static`
// 
pub fn three() {
	println!("Lifetime: scope annotations");
	let x = 7;
	let y = 9;

	print_one(&x);
	print_multi(&x, &y);

	let z = pass_x(&x, &y);
	print_one(z);

	let mut t = 0;
	add_one(&mut t);
	print_one(&t);
}

// One input reference with lifetime `'a` which must live
// at least as long as the function.
fn print_one<'a>(x: &'a i32) {
	println!("`print_one`: x is {}", x);
}

// Multiple elements with different lifetimes. In this case, it
// would be fine for both to have the same lifetime `'a`, but
// in more complex cases, different lifetimes may be required.
fn print_multi<'a, 'b>(x: &'a i32, y: &'b i32) {
	println!("`print_multi`: x is {}, y is {}", x, y);
}

// Returning references that have been passed in is acceptable.
// However, the correct lifetime must be returned.
fn pass_x<'a, 'b>(x: &'a i32, _: &'b i32) -> &'a i32 {
	x
}

// Mutable references are possible with lifetimes as well.
fn add_one<'a>(x: &'a mut i32) {
	*x += 1;
}

fn valid_output() -> String {
	// The function gives ownership back to the caller
	// the caller of `valid_output` is responsible for the lifetime of the returned `String`
	String::from("foo")
}

// // The following is invalid: `'a` must live longer than the function.
// fn invalid_output<'a>() -> &'a String {
//     &String::from("foo")
// }
// Here, `&String::from("foo")` would create a `String`, followed by a
// reference. Then the data is dropped upon exiting the scope, leaving
// a reference to invalid data to be returned.
