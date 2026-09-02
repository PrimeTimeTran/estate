// ## 7. Transformation & Duplication
//
// - `.clone()`
// - `.to_owned()`
// - `.to_string()`
// - `Copy`
// - `Clone`
// - Owned vs borrowed representations
// - Converting borrowed data into owned data

pub fn seven() {
	// 1. A reference has a lifetime
	let x = 10;
	let r = &x;

	// 2. You don't normally name that lifetime yourself.
	// Rust infers it.
	let r: &i32 = &x;

	// 3. To explicitly NAME a lifetime,
	// introduce it on the containing item.
	fn print<'a>(r: &'a i32) {
		println!("{}", r);
	}

	// 4. A struct can introduce and carry a lifetime.
	// A lifetime parameter doesn't create a lifetime; it gives
	// a name to a relationship between references and the
	// scopes in which they're valid.
	struct Borrowed<'a> {
		value: &'a i32,
	}

	// 5. `'a` means:
	// "the reference stored here is valid for lifetime 'a"
	//
	// And because Borrowed contains that reference:
	//
	// Borrowed<'a>
	//      │
	//      └── cannot outlive 'a
	// 5. Multiple references can share the same lifetime
	struct NamedBorrowed<'a> {
		x: &'a i32,
		y: &'a i32,
	}

	// `&T`
	// "this is a reference"

	// `&'a T`
	// "this is a reference with a named lifetime"

	// `'a`
	// is NOT a variable
	// is NOT a duration
	// is NOT something you assign to the reference
	//
	// It is a lifetime parameter introduced by:
	//
	// fn foo<'a>(...)
	// struct Foo<'a> { ... }
	// enum Foo<'a> { ... }

	// 6. The borrow now constrains what the owner can do
	let mut x = 10;

	// let borrowed = Borrowed(&x);

	// x cannot be mutably borrowed while `borrowed`
	// still holds its immutable borrow.
	x += 1; // ❌

	/*
		 x owns the value
		 │
		 │ immutable borrow
		 ▼
	Borrowed<'a>
		 │
		 │ must not outlive x
		 ▼
	x's lifetime

	Therefore:

			x ────────────────┐
												│
			borrowed ─────────┘
						 'a
	*/

	// 7. End the borrow
	// drop(borrowed);

	// Now the constraint is gone
	x += 1; // ✅

	// 8. Lifetimes don't copy or extend data.
	// They describe how long a reference is valid.
	//
	//     &'a T
	//       │
	//       └── "this reference is valid for 'a"
	//
	// `T`      = the thing being referenced
	// `&T`     = borrowed access to it
	// `'a`     = how long that access is valid

	// And if you want the **really compact progression** for your notes, I'd make it:

	// Lifetime progression:
	//
	// &T
	// ↓
	// &'a T
	// ↓
	// struct Foo<'a> { value: &'a T }
	// ↓
	// Foo<'a> cannot outlive T
	// ↓
	// While Foo holds the borrow, T is constrained
	// ↓
	// Drop Foo → borrow ends → T can be mutated
}
