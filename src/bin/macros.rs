use foo::*;
use macros_decl::{ declarative_macro };
use macros_proc::{ attribute_macro, Hello, procedural_macro };

fn main() {
	_declarative();
	// procedural();
}

fn _declarative() {
	let _var = "foo";
	declarative_macro!(_var);
	identify_foo!(bar);

	let values = vec_doubled![1, 2, 3];

	let result1: Result<Vec<_>, ()> = values
		.iter()
		.map(|f| {
			eprintln!("Token {}", f);
			Ok::<_, ()>(f)
		})
		.collect();

	println!("result1 {:?}", result1);
	// A list of items
	let spam = "spam";
	let ham = "ham";
	let ram = "ram";
	let result2 = identify_foos!(spam, ham, ram);
	println!("result2 {:?}", result2);
	// Errors:
	// "Here's a recipe for what to do with each item."
	// let result3 = vec![spam, ham, ram]
	// 	.iter()
	// 	.map(|f| {
	// 		println!("Char {}", f);
	// 		Ok::<_, ()>(f)
	// 	});
	// println!("result3 {:?}", result3);

	// To see the print, use this
	identify_foos!(spam, ham, ram).for_each(|_| {});
	// Or collect results
	let result4: Result<Vec<_>, ()> = identify_foos!(spam, ham, ram).collect();
	println!("result4 {:?}", result4);

	// ## Lazy vs Consumed evaluation
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

	// 1. Simple Syntax
	let squared: Vec<i32> = values
		.iter()
		.map(|i| i * i)
		.collect();
	// 2. Verbose
	// let squared: Vec<i32> = core::iter::IntoIterator
	// 	::into_iter(values)
	// 	.map(|i| i * i)
	// 	.collect();
	println!("squared {:?}", squared);

	// ## Filter Map vs Flat Map
	let evens: Vec<&i32> = squared
		.iter()
		.filter_map(|v| (v % 2 == 0).then(|| v))
		.collect();
	println!("filter_map evens {:?}", evens);

	let evens: Vec<&i32> = squared
		.iter()
		.flat_map(|v| (v % 2 == 0).then(|| v))
		.collect();
	println!("flat_map evens {:?}", evens);

	let words = vec!["hello", "world"];
	// USING flat_map: Expands each word into its individual characters.
	// Each &str turns into an iterator of chars, which flat_map flattens.
	let chars: Vec<char> = words
		.iter()
		.flat_map(|s| s.chars())
		.collect();
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

fn _procedural() {
	procedural_macro!("Hello spam ham");

	#[attribute_macro]
	fn hello() {
		println!("Hello!");
	}

	#[derive(Hello)]
	struct Person;

	fn main() {
		hello();
		Person::hello();
	}
	main()
}
