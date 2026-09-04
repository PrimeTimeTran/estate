```rust
fn structs() {
	// ## Classic C-Structs(Named Fields)
	//
	// These are the most common structs. They define a type with named fields enclosed in curly braces {}, making data accessible via dot notation
	//
	// Definition
	struct User {
		username: String,
		email: String,
		active: bool,
	}

	// Instantiation
	let user1 = User {
		username: String::from("alice123"),
		email: String::from("alice@example.com"),
		active: true,
	};

	// Accessing data
	println!("Username: {}", user1.username);

	// ## Tuple structs
	//
	// Tuple structs have the names for the struct type, but their individual fields are anonymous. The fields are enclosed in parentheses () and accessed using 0-indexed numbers (struct_name.0). [1] (https://doc.rust-lang.org/book/ch05-01-defining-structs.html), [2] (https://doc.rust-lang.org/reference/types/struct.html), [3] (https://exercism.org/tracks/rust/concepts/structs)
	//
	// Definition
	struct Point(i32, i32, i32);
	struct Color(i32, i32, i32);

	// Instantiation
	let origin = Point(0, 25, 50);
	let black = Color(0, 0, 0);

	// Accessing data
	println!("X coordinate: {}", origin.0);
}

fn unit_struct() {
	use std::marker::PhantomData;
	// Compile-time states
	struct Disconnected;
	struct Connected;

	struct Connection<State> {
		address: String,
		_state: PhantomData<State>, // Tells the compiler to track `State`
	}

	impl Connection<Disconnected> {
		fn connect(self) -> Connection<Connected> {
			Connection {
				address: self.address,
				_state: PhantomData,
			}
		}
	}

	impl Connection<Connected> {
		fn send_data(&self, data: &str) {
			println!("Sending: {}", data);
		}
	}
}
```
