use std::io::{self, Read};

fn main() {
	let mut input = String::new();
	io::stdin().read_to_string(&mut input).unwrap();

	let mut lines = input.lines();

	let nums: Vec<i32> = lines
		.next()
		.unwrap()
		.trim_matches(['[', ']'])
		.split(',')
		.map(|x| x.trim().parse().unwrap())
		.collect();

	let target: i32 = lines.next().unwrap().trim().parse().unwrap();

	// Two Sum
	for i in 0..nums.len() {
		for j in (i + 1)..nums.len() {
			if nums[i] + nums[j] == target {
				println!("[{}, {}]", i, j);
				return;
			}
		}
	}
}

// use std::io::{self, Read};

// fn main() {
// 	let mut input = String::new();
// 	io::stdin().read_to_string(&mut input).unwrap();

// 	let mut lines = input.lines();

// 	let nums: Vec<i32> = lines
// 		.next()
// 		.unwrap()
// 		.trim_matches(['[', ']'])
// 		.split(',')
// 		.map(|x| x.trim().parse().unwrap())
// 		.collect();

// 	let target: i32 = lines.next().unwrap().parse().unwrap();

// 	println!("[{}, {}]", 0, 1);
// }
