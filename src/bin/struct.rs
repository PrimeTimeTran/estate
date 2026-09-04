fn structs() {
	// ## Classic C-Structs(Named Fields)
	//
	// These are the most common structs. They define a type with named fields enclosed in curly braces {}, making data accessible via dot notation (struct_name.field_name). [1] (https://exercism.org/tracks/rust/concepts/structs), [2] (https://doc.rust-lang.org/rust-by-example/custom_types/structs.html), [3] (https://rustify.rs/glossary/struct), [4] (https://doc.rust-lang.org/std/keyword.struct.html)
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
	// Unit structs are heavily utilized in the Typestate Pattern to represent
	// the state of an object at compile time. By switching out generic
	// parameters representing states, the compiler can prevent you from calling
	// methods in the wrong order.
	//
	// Typestate lets you define different method sets
	// for different compile-time instantiations of the same generic struct.
	use std::marker::PhantomData;
	// Compile-time states
	struct Disconnected;
	struct Connected;
	struct Connection<State> {
		address: String,
		_state: PhantomData<State>,
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
		// fn foo(self)       // consumes/owns self
		// fn foo(&self)      // borrows self immutably
		// fn foo(&mut self)  // borrows self mutably
		fn read(&self) {
			// borrow
		}

		fn modify(&mut self) {
			// mutable borrow
		}
		fn send_data(&self, data: &str) {
			println!("Sending: {}", data);
		}
		fn bandwidth(&self) {
			println!("Bandwidth is a good thing");
		}
		fn disconnect(self) -> Connection<Disconnected> {
			Connection {
				address: self.address,
				_state: PhantomData,
			}
		}
	}
	let conn = Connection::<Disconnected> {
		address: "server.com".into(),
		_state: PhantomData,
	};
	let conn: Connection<Connected> = conn.connect();
	conn.send_data("hello");
	conn.bandwidth();
	let conn = conn.disconnect();
}

fn main() {
	structs();
	unit_struct();
}
