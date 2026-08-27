// ## 2. Sharing
// - Borrowing
// - Shared references (`&T`)
// - Mutable references (`&mut T`)
// - Aliasing
// - Exclusive access
// - Borrowing rules
// - Why sharing forces new abstractions

// Sharing data across scopes/function boundaries requires
// a set of rules around ownership, borrowing, copying, and lifetimes.
//
// Rust gives us sharing with forms of passing data depending on whether
// we want to transfer ownership, temporarily borrow it, or copy it.
//
// Each approach has tradeoffs worth understanding.
pub fn owning_and_sharing() {
	let city = "London";

	// This creates a `borrow ref`
	// which is sent to the function as a param.
	borrow_string_param(city);

	// The previous call has returned,
	// so `city` can be borrowed again.
	borrow_string_param(city);

	// `city` is a `&str`, so `.to_string()` creates a new owned `String`.
	//
	// The new String is moved into the function and
	// `city` itself is unaffected because we created a separate String.
	string_param_demands_ownership(city.to_string());

	// `.to_owned()` expresses the same ownership transition:
	// create an owned value from borrowed data.
	//
	// For an `&str`, `.to_owned()` also produces a new `String`.
	string_param_demands_ownership(city.to_owned());

	// The type here is not a borrowed ref.
	// It's an `owned` String.
	let city = String::from("New York");

	// # Semi Safe
	// Now we can pass city to ownership demanding String param.
	// But that transfers ownership
	// string_param_demands_ownership(city);

	// we produce a "borrow of moved value: `city`" error.
	// if we try to use `city` again.
	// string_param_demands_ownership(city);

	// # Safe
	// If we use .to_string() first
	string_param_demands_ownership(city.to_string());

	// Or .to_owned()
	string_param_demands_ownership(city.to_owned());

	// We don't produce the use of borrowed move error.
	// and can reuse multiple times.
	string_param_demands_ownership(city.to_owned());

	// `city` is still available because neither
	// function took ownership of `city` itself.
	println!("main still has access to city: {}", city);

	let (six, nine) = (6, 9);

	// `i32` implements `Copy`, so passing these values copies them
	// instead of moving ownership away from `six` and `nine`.
	print_copied_i32(six, nine);

	// Borrow `six` and `nine` for the duration required by each call.
	print_refs(&six, &nine);
	print_refs(&six, &nine);

	failed_borrow();
}

// The type of `a` is a borrrowed reference which happens to ref to a String.
fn borrow_string_param(a: &str) {
	println!("String borrowed: {}", a);
	// `a` is only borrowed for the duration required by this call.
	// The function does not take ownership of the string.
}

fn string_param_demands_ownership(a: String) {
	println!("String owned: {}", a);

	// `a` is owned by this function.
	// When the function returns, `a` is dropped.
}

fn ownership_of_a_clone_counts(a: String) {
	println!("String cloned: {}", a);

	// This function also receives ownership of its `String`.
}

// This function receives copied values.
fn print_copied_i32(x: i32, y: i32) {
	println!("x is {} and y is {}", x, y);
}

// These parameters receive borrowed references.
//
// `'a` and `'b` are lifetime parameters introduced by the function.
// They name the lifetimes associated with the two references.
//
// The references don't copy the `i32` values or take ownership.
fn print_refs<'a, 'b>(x: &'a i32, y: &'b i32) {
	println!("x is {} and y is {}", x, y);
}

// A lifetime parameter can also express a constraint on a reference.
fn failed_borrow<'a>() {
	let _x = 12;

	// This would require `_x` to remain alive for `'a`.
	//
	// But `_x` only exists inside this function's scope.
	// `'a` could represent a lifetime extending beyond that scope,
	// so Rust cannot allow the reference to outlive `_x`.
	//
	// let _y: &'a i32 = &_x;
}
