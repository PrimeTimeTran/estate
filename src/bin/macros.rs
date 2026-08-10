#![allow(warnings)]

use foo::*;

fn main() {
	// A single item
	let bar = "bar";
	identify_foo!(bar);

	let values = vec_doubled![1, 2, 3, 4, 5, 6, 7, 8];
	let result: Result<Vec<_>, ()> = values
		.iter()
		.map(|f| {
			eprintln!("Char {}", f);
			Ok::<_, ()>(f)
		})
		.collect();
	println!("result {:?}", result);

	// A list of items
	let spam = "spam";
	let ham = "ham";
	let ram = "ram";
	identify_foos!(spam, ham, ram);
	// Expands to roughly
	// "Here's a recipe for what to do with each item."
	// vec![spam, ham, ram, jam, dam, scam]
	//       .iter()
	//       .map(|f| {
	//           println!("Char {}", f);
	//           Ok::<_, ()>(f)
	//       });
	// To see the print, use this
	identify_foos!(spam, ham, ram).for_each(|_| {});

	// Or collect results
	let collected_foos: Result<Vec<_>, ()> = identify_foos!(spam, ham, ram).collect();
	println!("collected_foos {:?}", collected_foos);
	// General rule
	// These guys are lazy.
	// .map(...)
	// .filter(...)
	// .take(...)
	// .skip(...)
	//
	// They must be consumed
	// .collect()
	// .for_each(...)
	// .count()
	// .sum()
	// .next()
	// .any(...)
	let squared: Vec<i32> = core::iter::IntoIterator::into_iter(values)
		.map(|i| i * i)
		.collect();
	println!("squared {:?}", squared);

	let evens: Vec<&i32> = squared
		.iter()
		.filter_map(|v| ((v % 2) == 0).then(|| v))
		.collect();
	println!("evens {:?}", evens);

	let evens: Vec<&i32> = squared
		.iter()
		.flat_map(|v| ((v % 2) == 0).then(|| v))
		.collect();
	println!("evens flat_map {:?}", evens);

	let words = vec!["hello", "world"];
	// USING flat_map: Expands each word into its individual characters.
	// Each &str turns into an iterator of chars, which flat_map flattens.
	let chars: Vec<char> = words.iter().flat_map(|s| s.chars()).collect();

	println!("{:?}", chars);
	// Output: ['h', 'e', 'l', 'l', 'o', 'w', 'o', 'r', 'l', 'd']

	let result = count_block_identifiers!({
		let foo = 10;
		let bar = foo + 20;
		if bar > 10 {
			println!("{}", foo);
		}
		let baz = foo + bar;
	});

	println!("{result:?}");
}
