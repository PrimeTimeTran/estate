// ## 4. Functions
// - Borrowed parameters
// - Borrowed return values
// - Input/output lifetime relationships
// - Multiple references
// - Lifetime elision in functions
// - Returning references
// - Why some references cannot be returned

pub fn four() {
	println!("Lifetime: Unit Struct");
	let mut owner = Owner(18, 0, 0);

	owner.add_one();
	owner.add_two();
	owner.add_three();
	owner.print();
}

struct Owner(i32, i32, i32);

impl Owner {
	fn add_one(&mut self) {
		self.0 += 1;
	}

	fn add_two(&mut self) {
		self.1 += 1;
	}

	fn add_three(&mut self) {
		self.2 += 1;
	}

	fn print(&self) {
		println!("Values: {}, {}, {}", self.0, self.1, self.2);
	}
}
