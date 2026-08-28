use std::vec;

pub fn constraints() {
	println!("We want to share functionality between two different types.");
	let nums = vec![1, 2, 3, 4, 5];
	let mut largest = nums[0];
	println!("Largest: {}", find_largest(nums, &mut largest));

	let nums = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
	let mut largest = nums[0];
	println!("Largest: {}", find_largest(nums, &mut largest));
}

fn find_largest<T: PartialOrd + Copy>(nums: Vec<T>, largest: &mut T) -> T {
	for num in nums {
		if num > *largest {
			*largest = num;
		}
	}
	*largest
}
