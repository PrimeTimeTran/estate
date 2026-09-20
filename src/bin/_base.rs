//! # Trait Composition
//!
//! Example syntax and structure of trait composition.
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

/// ## [Implementations][implementations]
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

	/// [Copyable]
	///
	/// [Copy] blocks these fields [String], [Vec<T>]
	///
	/// The following examples are the options remaining.
	///
	#[derive(Clone, Copy)]
	struct Copyable {
		age: u32,
		active: bool,

		/// [String]
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
/// ### Trait composition
/// "What capabilities does this type promise?"
///
/// ###  Dynamic dispatch
/// "Which implementation do I want to choose at runtime?"
///
/// ###  Generic/static dispatch
/// "Which capability do I require at compile time?"
///
/// [traits]: https://doc.rust-lang.org/reference/items/traits.html
mod traits {
	use super::{e, i, s, *};

	/// Super traits increase requirements for the implementor of a trait.
	pub trait BaseTrait {
		fn a_borrowed_read(&self) -> &str;
	}

	/// They can demand  
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

	pub trait Into<T> {}

	// trait ReadMutate: Read + Mutate {}
	// impl<T: Read + Mutate> ReadMutate for T {}
}

// Bring all traits into scope
use self::traits::*;
// Alias these cause a cardinality is sufficient
use self::{enums as e, impls as i, structs as s, traits as t};

fn main() {
	println!("hi")
}

fn main2() {
	e::Foo::Bar;
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
	let a_borrowed_read = bar1.a_borrowed_read();
	println!("- a borrowed read {}", a_borrowed_read);
	let a_mutate_of_name = bar1.a_mutate_of_name("new Bar Name from &str");
	println!("- mutates from borrowed lifetimes {:?}", a_mutate_of_name);
	let spam = bar1.spam();
	println!("- call spam on bar which returns a () {:?}", spam);
}
