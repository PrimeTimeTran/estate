#![allow(warnings)]
// https://www.youtube.com/watch?v=3biW5NkNnrk

use std::{
	any::Any,
	cell::RefCell,
	marker::PhantomData,
	ops::Deref,
	rc::{Rc, Weak},
};

/// Enums
///
///
mod e {
	use super::{i, s, t, *};
}

/// Implementations
///
mod i {
	use super::{e, s, t, *};

	impl t::Animal for Dog {
		fn consuming(self) {
			println!(
				"Dog consuming inherent self {}",
				std::any::type_name::<Self>()
			);
		}
		fn mutate(&mut self) {
			println!("Dog mutate self from {}", std::any::type_name::<Self>());
		}
	}
	impl t::Speak for Dog {
		fn sound(&self) {
			println!("Bark")
		}
	}
	impl t::Speak for Cat {
		fn sound(&self) {
			println!("Meow")
		}
	}
	impl t::Speak for Lion {
		fn sound(&self) {
			println!("Roar")
		}
	}

	impl t::Reflect for s::Dog2 {
		fn fields() -> Vec<s::Field> {
			vec![
				s::Field {
					name: "name",
					type_name: std::any::type_name::<String>(),
				},
				s::Field {
					name: "age",
					type_name: std::any::type_name::<u32>(),
				},
				s::Field {
					name: "owners",
					type_name: std::any::type_name::<Vec<String>>(),
				},
			]
		}
	}
}

/// Structs
///
mod s {
	use super::*;
	use super::{e, i, t};

	#[derive(Clone, Default)]
	pub struct Dog;
	#[derive(Clone, Default)]
	pub struct Cat;
	#[derive(Clone, Default)]
	pub struct Lion;

	#[derive(Clone, Default)]
	pub struct Dog2 {
		pub name: String,
		pub age: u32,
		pub owners: Vec<String>,
	}

	#[derive(Clone)]
	pub struct Field {
		pub name: &'static str,
		pub type_name: &'static str,
	}
}

/// Traits
///
mod t {
	use super::{e, i, t, *};
	pub trait Animal {
		fn mutate(&mut self);

		/// ## [Animal::consuming]
		/// Every Animal must provide a consuming method that takes ownership of Self.
		///
		fn consuming(self);

		/// ## [Animal::inherent_sized]
		/// Sized is about the size of the value's type-level layout, not about whether it contains heap data.
		/// ```text
		/// ┌──────────────────────┐
		/// │ String               │
		/// │ ┌──────┬──────┬─────┐│
		/// │ │ ptr  │ len  │ cap ││
		/// │ └──┬───┴──────┴─────┘│
		/// └────│─────────────────┘
		///      │
		///      ▼
		///    heap data
		///    "Buddy"
		/// ```
		///
		/// - Every Animal gets this default method, but only concrete Sized implementations get to use it.
		/// - The Self: Sized effectively removes this method from consideration when you're dealing with:
		/// 	- `dyn Animal`
		fn inherent_sized(self) -> Self
		where
			Self: Sized,
		{
			println!(
				"Animal inherent_sized from trait {}",
				std::any::type_name::<Self>()
			);
			self
			// | Method                                           | Takes ownership? | Returns `Self`? | Default? | Available for `dyn Animal`? |
			// | ------------------------------------------------ | ---------------: | --------------: | -------: | --------------------------: |
			// | `consuming(self)`                                |                ✅ |               ❌ |        ❌ | restricted by object safety |
			// | `inherent_sized(self) -> Self where Self: Sized` |                ✅ |               ✅ |        ✅ |                           ❌ |
			// | `mutate(&mut self)`                              |                ❌ |               ❌ |  depends |                           ✅ |
		}
		fn trait_method_which_borrows(&self) {
			println!(
				"Animal trait_method_which_borrows self from {}",
				std::any::type_name::<Self>()
			);
			self.eat("trait");
			self.sleep();
			self.breath();
		}
		fn eat(&self, msg: &str) {
			println!("Instance eats... {}", msg)
		}
		fn sleep(&self) {
			println!("Instance sleeps...")
		}
		fn breath(&self) {
			println!("Instance breathes...")
		}
	}
	pub trait Speak {
		fn sound(&self);
	}

	pub trait Reflect {
		fn fields() -> Vec<s::Field>;
	}
}

use self::{e::*, i::*, s::*, t::*};

fn main() {
	let dog = Dog::default();
	dog.trait_method_which_borrows();
	/// Both move the value of the caller:
	///
	/// 	- inherent
	/// 	- inherent_sized
	///
	dog.clone().consuming();
	dog.clone().inherent_sized();
	///
	/// So these error
	///
	dog.eat("1");
	dog.sleep();
	dog.breath();

	/// Creates a mutable
	///
	let mut dog = Dog::default();
	dog.eat("2");
	dog.sleep();
	dog.breath();

	/// ## [live_life]: Generic method on any implementor of [Animal]
	///
	/// Enables dynamic dispatch to the Trait method because of
	///
	/// This dispatch is not on a specific instance but the class as defined by `self`
	///
	fn dynamic_live<A: t::Animal>(implementor: A) -> A {
		implementor.trait_method_which_borrows();
		implementor
	}
	/// Mutates it
	///
	let mut dog = dynamic_live(dog);
	dog.eat("3");
	dog.sleep();
	dog.breath();

	/// ## Create a method on any generic struct instance that implements a trait
	///
	/// This dispatch is not on a specific instance but the class as defined by `&self`
	fn makes_noise<Animal: t::Speak>(instance: Animal) {
		instance.sound();
	}
	// makes_noise(dog.clone());
	makes_noise(dog);
	// makes_noise(dog);
	let cat = Cat::default();
	makes_noise(cat);
	let lion = Lion::default();
	makes_noise(lion);

	for field in Dog2::fields() {
		println!("{}: {}", field.name, field.type_name);
	}
}
