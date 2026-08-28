use crate::helpers::*;

pub fn constraints() {
	section!("Traits can constrain what a generic type is allowed to do.");

	// 1. Generic Bounds
	// "What capabilities must T provide?"
	generic_bounds();

	// 2. Where Clauses
	// "How can I express complex bounds more clearly?"
	where_clauses();

	// 3. Multiple Bounds
	// "How can a type be required to satisfy several traits?"
	multiple_bounds();

	// 4. Nested Bounds
	// "How can constraints apply through associated or dependent types?"
	nested_bounds();

	// 5. Bounds on impl
	// "When should an implementation exist only for constrained types?"
	bounds_on_impl();

	// 6. Bounds on Associated Items
	// "How can individual methods, types, or constants be conditionally available?"
	bounds_on_associated_items();
}
// ------------------------------------------------------------
// 1. Generic Bounds
// ------------------------------------------------------------
//
// Concept:
// A generic type is unconstrained until we tell Rust what
// capabilities it must provide.
//
// T: PartialOrd
// "T must be comparable."
//
// T: Copy
// "T must be copied."
//
pub fn generic_bounds() {
	println!("\n1. Generic Bounds");

	let nums = vec![1, 5, 2, 9, 3];

	println!("Largest: {}", find_largest(nums));
}

fn find_largest<T: PartialOrd + Copy>(nums: Vec<T>) -> T {
	let mut largest = nums[0];

	for num in nums {
		if num > largest {
			largest = num;
		}
	}

	largest
}

// ------------------------------------------------------------
// 2. Where Clauses
// ------------------------------------------------------------
//
// Concept:
// The same constraints can be moved away from the generic
// declaration and expressed separately with `where`.
//
// This becomes useful as constraints grow.
//
pub fn where_clauses() {
	println!("\n2. Where Clauses");

	let nums = vec![1, 5, 2, 9, 3];

	println!("Largest: {}", find_largest(nums));
}

// fn find_largest<T>(nums: Vec<T>) -> T
// where
// 	T: PartialOrd + Copy,
// {
// 	let mut largest = nums[0];

// 	for num in nums {
// 		if num > largest {
// 			largest = num;
// 		}
// 	}

// 	largest
// }

// ------------------------------------------------------------
// 3. Multiple Bounds
// ------------------------------------------------------------
//
// Concept:
// A generic type can be constrained by several traits.
//
// T: PartialOrd + Copy + Display
//
// This means:
//
//     T
//     ├── can be compared
//     ├── can be copied
//     └── can be displayed
//
pub fn multiple_bounds() {
	println!("\n3. Multiple Bounds");

	print_largest(vec![1, 5, 2, 9, 3]);
	print_largest(vec![1.0, 5.0, 2.0, 9.0]);
}

fn print_largest<T>(nums: Vec<T>) -> T
where
	T: PartialOrd + Copy + std::fmt::Display,
{
	let mut largest = nums[0];

	for num in nums {
		if num > largest {
			largest = num;
		}
	}

	println!("Largest: {largest}");

	largest
}

// ------------------------------------------------------------
// 4. Nested Bounds
// ------------------------------------------------------------
//
// Concept:
// Constraints don't have to describe a single type in isolation.
//
// They can describe relationships between types.
//
// Here:
//
//     Container::Item: Display
//
// means:
//
//     Container must provide an Item type
//     AND
//     that Item type must implement Display.
//
// This is where bounds start becoming relationships.
//
pub fn nested_bounds() {
	println!("\n4. Nested Bounds");

	let numbers = Numbers(vec![1, 2, 3]);

	print_items(numbers);
}

struct Numbers(Vec<i32>);

impl IntoIterator for Numbers {
	type Item = i32;
	type IntoIter = std::vec::IntoIter<i32>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

fn print_items<T>(items: T)
where
	T: IntoIterator,
	T::Item: std::fmt::Display,
{
	for item in items {
		println!("Item: {item}");
	}
}

// ------------------------------------------------------------
// 5. Bounds on impl
// ------------------------------------------------------------
//
// Concept:
// We can constrain an entire implementation.
//
// This means the implementation only exists when T satisfies
// the required trait bound.
//
// Without:
//
//     T: Display
//
// `print()` does not exist for every possible T.
//
pub fn bounds_on_impl() {
	println!("\n5. Bounds on impl");

	let value = Wrapper(42);

	value.print();
}

struct Wrapper<T>(T);

impl<T> Wrapper<T>
where
	T: std::fmt::Display,
{
	fn print(&self) {
		println!("Value: {}", self.0);
	}
}

// ------------------------------------------------------------
// 6. Bounds on Associated Items
// ------------------------------------------------------------
//
// Concept:
// We don't always want to constrain the entire impl.
//
// Individual methods can have their own bounds.
//
// The type itself can exist for ANY T,
// while specific behavior only exists for T satisfying
// additional requirements.
//

pub fn bounds_on_associated_items() {
	println!("\n6. Bounds on Associated Items");

	let value = Boxed(42);

	value.print();
	value.compare(100);
}

struct Boxed<T>(T);

impl<T> Boxed<T> {
	fn print(&self)
	where
		T: std::fmt::Display,
	{
		println!("Value: {}", self.0);
	}

	fn compare(&self, other: T)
	where
		T: PartialOrd,
	{
		println!("Greater: {}", self.0 > other);
	}
}
