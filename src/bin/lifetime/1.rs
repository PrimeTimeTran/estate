/// Lifetime Primitives
///
/// Static
/// Mutate
/// Reference
/// Pointer
/// Ownership
///		[`.into()`]
///		[`.take()`]
///		[`.unwrap()`]
///		[`.drain()`]
///		[`mem::take()`]
///		[`mem::replace()`]
///		[`.clone()`]
///		[`.to_owned()`]
///		[`.to_string()`]
///		[`.to_vec()`]
///		[`.as_ref()`]
///		[`.as_mut()`]
///		[`.borrow()`]
///		[`.deref()`]
///		[`Arc::try_unwrap()`]
///		[`Box::into_inner`]
///		[`Box::leak`]
///		[`.iter()`]
///		[`.into_iter()`]
///		[`.iter_mut()`]
///		[`Rc::clone()`]
///		[`Arc::clone()`]
///		[`.as_deref()`]
///		[`.map()`]
///		[`.and_then()`]
///		[`.ok()`]
///		[`std::convert::TryInto::try_into()`]
/// 	to_owned()
/// Consume
///
/// Borrow
///
///
///
pub fn abstraction_of_references_and_pointers() {
	let i = 3;
	let borrow1 = &i;
	print!("borrow1: {}", borrow1);
	let borrow2 = &i;
	dbg!("borrow2: {}", borrow2);
	// dbg! macro prefixes prints with
	{
		let borrow3 = &i;
		println!("borrow3: {}", borrow3);
	}
	{
		let borrow4 = &i;
		println!("borrow4: {}", borrow4);
	}
	let borrow5 = &i;
	println!("borrow5: {}", borrow5);
}

// pub fn abstraction_of_references_and_pointers() {
// 	let i = 3;
// 	let borrow1 = &i;
// 	print!("borrow1: {}\n", borrow1);

// 	let borrow2 = &i;
// 	dbg!(borrow2); // Note: dbg! macro prints with file/line location

// 	{
// 		let borrow3 = &i;
// 		println!("borrow3: {}", borrow3);
// 	} // borrow3 goes out of scope here, but `i` and `borrow1`/`borrow2` remain valid.

// 	{
// 		let borrow4 = &i;
// 		println!("borrow4: {}", borrow4);
// 	}

// 	let borrow5 = &i;
// 	println!("borrow5: {}", borrow5);
// }

// fn main() {
// 	abstraction_of_references_and_pointers();
// }
