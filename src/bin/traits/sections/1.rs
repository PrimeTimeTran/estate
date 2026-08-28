use std::vec;

pub fn one() {
	println!("We have code that works on a type");
	let nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
	let mut largest = nums[0];
	for num in nums {
		if num > largest {
			largest = num;
		}
	}
	println!("Largest: {}", largest);
}
