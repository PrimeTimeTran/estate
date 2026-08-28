use crate::helpers::*;
use std::vec;

pub fn capability() {
	section!("1. Capabilities");

	// 1. Concrete Behavior
	// "We can write behavior for one concrete type."
	single_typed();

	// 2. Repeated Behavior
	// "The same behavior may be needed for other types."
	repeated_behavior();

	// 3. Generic Abstraction
	// "Can we make the type vary?"
	generic_types();

	// 4. Trait Capability
	// "What must the varying type be able to do?"
	trait_capability();

	// 5. Generic + Trait
	// "Can we abstract over the type while requiring capabilities?"
	generic_capability();
}

pub fn single_typed() {
	section!("1. Single Typed");
	println!("The function works, but only for one concrete type.");
	let nums = vec![1, 2, 3];
	println!("Largest: {}", find_largest(nums));
}

fn find_largest(nums: Vec<i32>) -> i32 {
	let mut largest = nums[0];

	for num in nums {
		if num > largest {
			largest = num;
		}
	}

	largest
}

pub fn dynamic_problem() {
	let nums1 = vec![1, 2, 3];
	let nums2 = vec![1.0, 2.0, 3.0];
	println!("Largest: {}", find_largest(nums1));
	// Unfortunately this doesn't work.
	// We could define a float impl but that's annoying
	// println!("Largest: {}", find_largest(nums2));
}

pub fn generic_param_types() {
	fn find_largest<T: PartialOrd + Copy>(nums: Vec<T>) -> T {
		let mut largest = nums[0];
		for num in nums {
			if num > largest {
				largest = num;
			}
		}
		largest
	}
	let nums1 = vec![1, 2, 3];
	let nums2 = vec![1.0, 2.0, 3.0];
	println!("Largest: {}", find_largest(nums1));
	println!("Largest: {}", find_largest(nums2));
}

pub fn repeated_behavior() {
	let nums1 = vec![1, 2, 3];
	let nums2 = vec![1.0, 2.0, 3.0];

	println!("Largest integer: {}", find_largest(nums1));

	// This doesn't work because `find_largest` specifically accepts
	// `Vec<i32>`.
	//
	// println!("Largest float: {}", find_largest(nums2));
	//
	// We could write:
	//
	//     fn find_largest_f64(...)
	//
	// But now we're duplicating behavior.
}
pub fn generic_types() {
	fn find_largest<T>(nums: Vec<T>) -> T {
		// We can't actually compare T yet.
		//
		// The important idea here is:
		//
		//     T = "some type"
		//
		// The function no longer cares whether T is i32,
		// f64, String, or something else.

		todo!()
	}

	println!("A generic T can represent many concrete types.");
}
pub fn trait_capability() {
	println!("A trait describes a capability.");

	fn compare<T: PartialOrd>(a: T, b: T) -> bool {
		a > b
	}

	println!("3 > 2: {}", compare(3, 2));
	println!("3.0 > 2.0: {}", compare(3.0, 2.0));

	// T
	// │
	// └── Generic
	//    "I don't care what the concrete type is."

	// T: PartialOrd
	// │
	// └── Constrained generic
	//    "I don't care what the concrete type is,
	//     as long as it can be compared."
}

pub fn generic_capability() {
	fn find_largest<T: PartialOrd + Copy>(nums: Vec<T>) -> T {
		let mut largest = nums[0];

		for num in nums {
			if num > largest {
				largest = num;
			}
		}

		largest
	}

	let integers = vec![1, 2, 3];
	let floats = vec![1.0, 2.0, 3.0];

	println!("Largest integer: {}", find_largest(integers));
	println!("Largest float: {}", find_largest(floats));
	// Generics determine WHAT varies.
	// Traits determine WHAT that varying type can do.
	//
	// T: PartialOrd + Copy
	//
	// T
	// └── the concrete type may vary
	//
	// PartialOrd
	// └── T must support comparison
	//
	// Copy
	// └── T must support copying
}
