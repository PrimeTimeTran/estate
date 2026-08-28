use std::vec;

pub fn capability() {
	println!("There exists code that works.");
	single_type_param();

	println!("But it works for only one param.");
	dynamic_problem();

	println!("We can share code with generic types.");
	generic_param_types();
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
pub fn single_type_param() {
	let nums = vec![1, 2, 3];
	println!("Largest: {}", find_largest(nums));
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
