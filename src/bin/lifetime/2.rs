fn failed_borrow<'a>() {
	let _x = 12;
	// let _y: &'a i32 = &_x;
}

pub fn two() {
	println!("Lifetime: passing with clones & borrows");
	let city = "London";

	fn print_borrowed_string(a: &str) {
		// I've borrowed a
		println!("String borrowed: {}", a);
		// And after this function returns, I no longer have access to it
	}
	print_borrowed_string(city);
	// So this previous "move" is returned to the caller scope and no longer throws a "borrow after free" error
	print_borrowed_string(city);

	fn print_string_owned(a: String) {
		println!("String owned: {}", a);
	}

	fn print_string_cloned(a: String) {
		println!("String cloned: {}", a);
	}
	print_string_owned(city.to_owned());
	print_string_cloned(city.to_string().clone());

	println!("main still has access to city: {}", city);

	fn print_copied_i32(x: i32, y: i32) {
		println!("x is {} and y is {}", x, y);
	}

	fn print_refs<'a, 'b>(x: &'a i32, y: &'b i32) {
		println!("x is {} and y is {}", x, y);
	}

	let (four, nine) = (4, 9);
	print_copied_i32(four, nine);
	print_refs(&four, &nine);
	print_refs(&four, &nine);
	failed_borrow();
}
