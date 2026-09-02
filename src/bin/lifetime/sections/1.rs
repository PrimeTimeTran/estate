use crate::helpers::*;

// ## 1. Memory
//
// - Stack vs Heap
// - Ownership
// - Allocation and deallocation
// - Values vs references
// - Moves
// - Copies
// - Destruction / Drop

pub fn memory_considerations() {
	section!("1. Memory");
	let i = 3;

	// It's used to define more data.
	let borrow1 = &i;

	// It's passed to functions.
	println!("borrow1: {}", borrow1);

	// And can also be borrowed multiple times at once.
	let borrow2 = &i;
	println!("borrow2: {}", borrow2);

	{
		// Data can flow "down" into child scopes.
		let borrow3 = &i;
		println!("borrow3: {}", borrow3);

		// When this scope ends, borrow3 disappears.
	}

	{
		// This can repeat as many times as needed.
		let borrow4 = &i;
		println!("borrow4: {}", borrow4);
	}

	// So far, everything is easy:
	//
	//       i
	//      /|\
	//     / | \
	//    b1 b2 b3...
	//
	// Multiple borrows can point at the same data.

	let borrow5 = &i;
	println!("borrow5: {}", borrow5);

	// But references introduce an important question:
	//
	// "How long is each borrow allowed to exist?"
	//
	// Rust needs to know:
	//
	// 1. Does the borrowed data still exist?
	// 2. Is the reference still being used?
	// 3. Can the owner be moved or mutated yet?
	// 4. Can this reference escape the scope that created it?

	// The fundamental rule:
	//
	//     A reference must never outlive the data it references.
	//
	// Therefore:
	//
	//     data lifetime >= borrow lifetime
	//
	let borrow;
	{
		let x = 10;
		borrow = &x;
		// `x` exists here, so `borrow` is valid.
		println!("{}", borrow);
	}
	// x is gone, but borrow would still exist.
	// Rust rejects this because borrow would outlive x.

	// let invalid = borrow; // ❌
	let mut x = 10;
	let borrow = &x;

	// `borrow` keeps an immutable borrow alive
	// while it is still being used.
	println!("{}", borrow);

	// Once the borrow's lifetime ends,
	// `x` can be mutably borrowed again.
	x += 1;
}
