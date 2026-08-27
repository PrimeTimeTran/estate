	
	// ## 6. Ownership Transfer
	// - `.into()`
	// - `.take()`
	// - `.unwrap()`
	// - `.drain()`
	// - `mem::take()`
	// - `mem::replace()`
	// - Move vs borrow
	// - Ownership as an alternative to lifetime complexity
pub fn six() {
	let b: Borrowed = Default::default();
	println!("b is {:?}", b);
}


// A struct with annotation of lifetimes.
#[derive(Debug)]
struct Borrowed<'a> {
	x: &'a i32,
}

// Annotate lifetimes to impl.
impl<'a> Default for Borrowed<'a> {
	fn default() -> Self {
		Self { x: &10 }
	}
}

