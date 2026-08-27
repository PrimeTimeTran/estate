// A type `Borrowed` which houses a reference to an
// `i32`. The reference to `i32` must outlive `Borrowed`.
#[derive(Debug)]
struct Borrowed<'a>(&'a i32);

// Similarly, both references here must outlive this structure.
#[derive(Debug)]
struct NamedBorrowed<'a> {
	x: &'a i32,
	y: &'a i32,
}

// An enum which is either an `i32` or a reference to one.
#[derive(Debug)]
enum Either<'a> {
	Num(i32),
	Ref(&'a i32),
}

pub fn five() {
	let mut x = 10;
	let mut y: i32 = 20;

	let single = Borrowed(&x);
	let double = NamedBorrowed { x: &x, y: &y };
	let reference = Either::Ref(&x);
	let number = Either::Num(y);

	println!("x is borrowed in {:?}", single);
	println!("x and y are borrowed in {:?}", double);
	println!("y is *not* borrowed in {:?}", number);
	println!("x is borrowed in {:?}", reference);

	println!("Again, double and number");
	println!("x and y are borrowed in {:?}", double);
	println!("y is *not* borrowed in {:?}", number);
	// 1. Fails here due to borrow checker
	// y += 1;
	//
	// 2. Dropping the borrows allows `y` to be mutated
	drop(double);
	// Now it can be mutated
	y += 10;
	println!("y is the same guy updated {:?}", y);
	x += 100;
	println!("x {:?}", x);
	// println!("referenceOfY {:?}", referenceOfY);
	// let mut y = y + 1;
}
