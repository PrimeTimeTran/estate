//! # Trait Composition
//!
//! Syntax examples of how to compose traits in a robust way.
//!
#![allow(warnings)]

use std::{
	any::Any,
	cell::RefCell,
	marker::PhantomData,
	ops::Deref,
	rc::{Rc, Weak},
};

/// ## [Enums][enums]
///
/// Name spaced enums to prevent name collisions.
///
/// [enums]: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
mod enums {
	use super::{i, s, t, *};

	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub enum Foo {
		#[default]
		Bar,
		Spam,
	}
}

/// ## [Implementations][implementations]:
///
/// Name spaced implementations to prevent name collisions.
///
/// [implementations]: https://doc.rust-lang.org/reference/items/implementations.html
mod impls {
	use super::{e, s, t, *};

	impl s::Foo {
		pub fn new(name: &str) -> Self {
			Self {
				name: name.to_string(),
			}
		}
	}
	impl t::BaseTrait for s::Foo {
		fn a_borrowed_read(&self) -> &str {
			&self.name
		}
	}
	impl t::Foo for s::Foo {
		fn a_borrowed_string_mutates(&mut self, new_name: &str) -> &Self {
			self.name = new_name.to_string();
			self
		}
		fn a_mutate_of_name(&mut self, new_name: &str) -> &Self {
			self.name = new_name.to_string();
			self
		}
		fn spam(self) {
			println!("- Foos's spam {}", self.name);
		}
	}

	impl<'a> s::Bar<'a> {
		pub fn new(name: &'a str) -> Self {
			Self { name }
		}
	}
	impl<'a> t::Jam<'a> for s::Bar<'a> {
		fn a_borrowed_read(&self) -> &str {
			self.name
		}
		fn a_mutate_of_name(&mut self, new_name: &'a str) -> &Self {
			self.name = new_name;
			self
		}

		fn spam(self) {
			println!("... Jam's spam {}", self.name)
		}
	}

	impl t::Read for s::Foo {
		fn read(&self) -> &str {
			&self.name
		}
	}

	impl t::Mutate for s::Foo {
		fn mutate(&mut self, s: &str) {
			self.name = s.into();
		}
	}

	pub fn foo() {
		let matchable = e::Foo::Bar;
		match matchable {
			e::Foo::Bar => {}
			e::Foo::Spam => {}
		}
	}
}

/// ## [Structs][structs]
///
/// Name spaced structs to prevent name collisions.
///
/// [structs]: https://doc.rust-lang.org/reference/items/structs.html
mod structs {
	use super::{e, i, t, *};

	#[derive(Clone, Debug, Default, Eq, PartialEq)]
	pub struct Foo {
		pub name: String,
	}

	#[derive(Clone, Debug, Default, Eq, PartialEq)]
	pub struct Bar<'a> {
		pub name: &'a str,
	}

	#[derive(Clone, Copy)]
	struct Copyable {
		age: u32,
		active: bool,

		/// String
		/// 	↓
		/// &'static str
		name: &'static str,

		/// Vec<&'static str>
		///   ↓
		/// &'static [&'static str]
		items: &'static [&'static str],
	}
}

/// ## [Traits][traits]
///
/// Name spaced traits to prevent name collisions.
///
/// [traits]: https://doc.rust-lang.org/reference/items/traits.html
mod traits {
	use super::{e, i, s, *};
	/// Trait composition
	/// 		↓
	/// "What capabilities does this type promise?"
	///
	/// Dynamic dispatch
	/// 		↓
	/// "Which implementation do I want to choose at runtime?"
	///
	/// Generic/static dispatch
	/// 		↓
	/// "Which capability do I require at compile time?"

	pub trait BaseTrait {
		fn a_borrowed_read(&self) -> &str;
	}

	pub trait Foo: BaseTrait {
		// fn a_borrowed_read(&self) -> &str;
		fn a_borrowed_string_mutates(&mut self, new_name: &str) -> &Self;
		fn a_mutate_of_name(&mut self, new_name: &str) -> &Self;
		fn spam(self);
	}

	pub trait Jam<'a> {
		fn a_borrowed_read(&self) -> &str;
		fn a_mutate_of_name(&mut self, new_name: &'a str) -> &Self;
		fn spam(self);
	}

	pub trait Read {
		fn read(&self) -> &str;
	}

	pub trait Mutate {
		fn mutate(&mut self, s: &str);
	}

	// trait ReadMutate: Read + Mutate {}
	// impl<T: Read + Mutate> ReadMutate for T {}
}

use self::{e::*, i::*, s::*, t::*};
use self::{enums as e, impls as i, structs as s, traits as t};

fn main() {
	e::Foo::Bar;
	i::foo;
	type traitable = dyn t::Foo;

	let mut foo1 = s::Foo::default();
	foo1.name = "new Foo".to_string();
	println!("- mutate instance fields {:?}", foo1.name);
	println!(
		"- read from borrowed self instance  {:?}",
		foo1.a_borrowed_read()
	);
	foo1.a_borrowed_string_mutates("new &str Name ");
	println!(
		"- mutate using a borrowed $str {:?}",
		foo1.a_borrowed_read()
	);

	let mut bar1 = s::Bar::new("new Bar");
	let name = bar1.a_borrowed_read();
	println!("- a borrowed read {}", name);
	let name = bar1.a_mutate_of_name("new Bar Name from &str");
	println!("- mutates from borrowed lifetimes {:?}", name);
	// let name = bar1.a_borrowed_string_mutates("new Bar Name from &str");
	// println!("- mutates from borrowed lifetimes {:?}", name);
	let name = bar1.spam();
	println!("- call spam on bar which returns a () {:?}", name);
}
