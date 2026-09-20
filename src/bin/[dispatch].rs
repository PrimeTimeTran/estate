// https://www.youtube.com/watch?v=3biW5NkNnrk
// JetBrains
// - shift+shift
// 	- All, types, files, symbols, actions, text
// - cmd+e: Recent Files
// 	-
// 	-
// - cmd+e: Recent Files
// - left opt + f1: select in
// - ctrl+tab: the switcher
// - ctrl+ctrl: run anything
// - cmd+f12: code structure popup
// - cmd+cmd (zen mode): shows side rails
//
#![allow(warnings)]

use std::{
	any::Any,
	cell::RefCell,
	marker::PhantomData,
	ops::Deref,
	rc::{Rc, Weak},
};

use self::traits::*;
use self::{enums as e, structs as s};

fn main() {
	// Global context
	let context = s::Context::default();
	let state = s::State::default();

	// ## Type Param Techniques
	{
		use crate::structs::Foo;
		let foo1 = Foo::new(context, state);
		let foo2 = Foo::new_with_type::<s::Context, s::State>(context, state);
		let foo3 = Foo::<s::Context, s::State>::new_where(context, state);
		let foo_from_converted = Foo::<s::Context, s::State>::convert_from::<s::C, s::S>(s::C, s::S);
		let foo_from_identity =
			Foo::<s::Context, s::State>::convert_from::<s::Context, s::State>(context, state);
	}

	// Person's Lifetime
	{
		let mut tom = s::Person::new(context, state, String::from("Tom"), e::Gender::Male);
		let mut ava = s::Person::new(context, state, String::from("Ava"), e::Gender::Female);

		println!("{}'s state {:?}", tom.name, tom.state());
		println!("{}'s is_single {}", tom.name, tom.is_single());
		println!("{}'s state {:?}", ava.name, ava.state());
		println!("{}'s is_single {}", ava.name, ava.is_single());

		// Work Context
		{
			let work_tom = s::Work::new(s::CtxWork::default(), s::StateWork::default(), tom.clone());
			println!("Work Tom's name: {:?}", work_tom.name);
			println!(
				"Work Tom's birth/legal/person name: {:?}",
				work_tom.person.name
			);
		}

		// Family/Dating/Legacy Context
		{
			let mut p1 = s::Person::new(
				s::Context::default(),
				s::Single,
				String::from("Tom"),
				e::Gender::Male,
			);
			let mut p2 = s::Person::new(
				s::Context::default(),
				s::Single,
				String::from("Ava"),
				e::Gender::Female,
			);
			println!("{} is_single {}", p1.name, p1.is_single());
			println!("{} is_single {}", p2.name, p2.is_single());

			// ## [3] methods available:
			// - p1.date_of_where(&mut p2);
			// - p1.date_of_single(&mut p2);
			// - p1.date_of_runtime(&mut p2);
			//
			{
				// p1.date_of_where(&mut p2);
				// println!("post date_of_where");
				// println!("{} is_single  {}", p1.name, p1.is_single());
				// println!("{} is_single {}", p2.name, p2.is_single());

				// p1.date_of_single(&mut p2);
				// println!("post date_of_single");
				// println!("{} is_single {}", p1.name, p1.is_single());
				// println!("{} is_single {}", p2.name, p2.is_single());

				p1.date_of_runtime(&mut p2);
				println!("post date_of_runtime");
				println!("{} is_single  {}", p1.name, p1.is_single());
				println!("{} is_single {}", p2.name, p2.is_single());
			}

			if let Some(tom) = tom.as_single() {
				// Only this is available
				// tom.date_of_lifetimed_singleref();
				tom.date_of_lifetimed_singleref(&mut ava);
			}
			tom.date_of_runtime(&mut ava);
			println!("{} is {:?}", tom.name(), tom.martial_status);
			println!("{} is {:?}", ava.name(), ava.martial_status);

			// Marriage Context
			{
				let (tom, ava, family) = tom.marry(ava);

				// And they can discuss with spouse, but must borrow themselves as they're married (part of family)
				tom.borrow().discuss_with_spouse("Where should we live?");
				ava.borrow().discuss_with_spouse("Should we have children?");
				println!(
					"{} is {:?}",
					tom.borrow().name(),
					tom.borrow().martial_status
				);
				println!(
					"{} is {:?}",
					ava.borrow().name(),
					ava.borrow().martial_status
				);

				// ─────────────────────────────────────────
				// They have a child
				// ─────────────────────────────────────────

				let child = family.borrow_mut().have_child("Alice", e::Gender::Female);

				println!(
					"{} is a parent {:?}",
					tom.borrow().name(),
					tom.borrow().martial_status
				);
				println!(
					"{} is a parent {:?}",
					ava.borrow().name(),
					ava.borrow().martial_status
				);
				println!("{} is a child {:?}", child.name(), child.martial_status);
				// tom.consider_request("Can I stay up late?");
				// tom.discipline_child("Alice", "She stayed up too late");
				// println!("{}'s children: {:?}", tom.name(), tom.children());
			}
		}
	}
}

mod enums {
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub enum Gender {
		#[default]
		Male,
		Female,
	}
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Spouse {
		Husband,
		Wife,
	}
	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
	pub enum RelationshipStatus {
		#[default]
		Single,
		Dating,
		Married,
		// Divorced,
	}
}

mod impls {
	use super::{e, s, *};

	impl<C, S> Deref for s::Child<C, S> {
		type Target = s::Person<s::CtxPerson, s::StatePerson>;
		fn deref(&self) -> &Self::Target {
			&self.person
		}
	}

	impl RelationshipState for s::Single {
		fn status() -> e::RelationshipStatus {
			e::RelationshipStatus::Single
		}
	}
	impl RelationshipState for s::Dating {
		fn status() -> e::RelationshipStatus {
			e::RelationshipStatus::Dating
		}
	}

	impl<C, S> s::Person<C, S>
	where
		S: RelationshipState,
	{
		pub fn foobar(&self) {
			println!("Instance Function")
		}
		pub fn associated_function() {}
	}
	impl<C, S> s::Person<C, S> {
		pub fn as_single(&mut self) -> Option<s::SingleRef<'_, C, S>> {
			if self.martial_status == e::RelationshipStatus::Single {
				Some(s::SingleRef { person: self })
			} else {
				None
			}
		}
	}

	impl<C, S: RelationshipState> s::Person<C, S> {
		pub fn relationship_status(&self) -> e::RelationshipStatus {
			S::status()
		}
	}

	/// Compile-time capability:
	/// `date_of_where` is available whenever `S: CanDate`.
	/// Flexible across any state that grants the capability, but does not verify
	/// that the person's current runtime state actually matches that capability.
	///
	impl CanDate for s::Single {}
	impl CanMarry for s::Dating {}
	impl CanDivorce for s::Married {}

	impl<C, S> s::Person<C, S>
	where
		S: CanDate,
	{
		pub fn date_of_where(&mut self, other: &mut s::Person<C, S>) {
			println!("date_of_where");
		}
	}
	impl<C> s::Person<C, s::Single> {
		pub fn date_of_single(&mut self, other: &mut s::Person<C, s::Single>) {
			println!("date_of_single");
			if self.martial_status == e::RelationshipStatus::Single
				&& other.martial_status == e::RelationshipStatus::Single
			{
				self.martial_status = e::RelationshipStatus::Dating;
				other.martial_status = e::RelationshipStatus::Dating;
			}
		}
	}

	impl<C, S> s::Child<C, S> {
		fn new(context: C, state: S, person: s::Person<s::CtxPerson, s::StatePerson>) -> Self {
			Self {
				context,
				state,
				person,
				parents: vec![],
			}
		}
	}

	impl<C> s::Family<C, s::Married> {
		pub fn spouse_of(
			&self,
			person: &s::Person<C, s::Married>,
		) -> Option<&s::Person<C, s::Married>> {
			todo!("spouse of")
		}
	}

	impl<C: Clone, S> s::Family<C, S> {
		pub fn new(context: C, state: S) -> Self {
			Self {
				context,
				state,
				members: vec![],
			}
		}
		pub fn add_member(
			&mut self,
			person: Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			family: Weak<RefCell<s::Family<s::CtxFamily, s::StateFamily>>>,
		) {
			let member = Rc::new(RefCell::new(s::FamilyMember::new(
				s::Context::default(),
				s::State::default(),
				person.clone(),
				family,
			)));
			person.borrow_mut().families.push(Rc::downgrade(&member));
			self.members.push(member);
		}
		pub fn have_child(
			&mut self,
			name: impl Into<String>,
			gender: e::Gender,
		) -> s::Child<s::Context, s::State> {
			let context = s::CtxPerson::default();
			let state = s::StatePerson::default();
			let person = s::Person::new(context.clone(), state.clone(), name.into(), gender);
			let context = s::Context::default();
			let state = s::State::default();
			let child = s::Child::new(context, state, person);
			child
		}
	}

	impl<C, S> s::FamilyMember<C, S> {
		pub fn new(
			context: C,
			state: S,
			person: Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			family: Weak<RefCell<s::Family<s::CtxFamily, s::StateFamily>>>,
		) -> Self {
			Self {
				context,
				state,
				person,
				family,
			}
		}
	}

	impl<C: Clone> s::Person<C, s::Dating> {}
	impl<C: Clone> s::Person<C, s::Married> {}

	/// =======================================================
	/// impl<C, S> s::Person<C, S> {}
	/// vs
	/// impl<C: Clone, S: Clone> s::Person<C, S> {}
	/// =======================================================
	/// Multiple inherent impl blocks organize
	/// methods according to the trait bounds they require.
	///
	impl<C: Clone, S: Clone> s::Person<C, S> {
		pub fn is_single(&self) -> bool {
			self.martial_status == e::RelationshipStatus::Single
		}
		pub fn marry(
			self,
			partner: s::Person<C, S>,
		) -> (
			Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			Rc<RefCell<s::Family<s::CtxFamily, s::StateFamily>>>,
		) {
			// Keep the actual people we're going to put into the family.
			let tom = Rc::new(RefCell::new(
				self
					.in_ctx(s::CtxPerson::default())
					.in_state(s::StatePerson::default())
					.to_married(),
			));
			let ava = Rc::new(RefCell::new(
				partner
					.in_ctx(s::CtxPerson::default())
					.in_state(s::StatePerson::default())
					.to_married(),
			));
			let family = Rc::new(RefCell::new(s::Family::new(
				s::CtxFamily::default(),
				s::StateFamily::default(),
			)));
			let family_weak = Rc::downgrade(&family);
			family
				.borrow_mut()
				.add_member(tom.clone(), family_weak.clone());
			family.borrow_mut().add_member(ava.clone(), family_weak);
			(tom, ava, family)
		}
	}

	impl<C, S> s::Person<C, S>
	where
		S: Dating,
	{
		pub fn dating(self) {}
	}
	/// Runtime capability:
	/// `date_of_runtime` is available on every Person and decides at runtime whether
	/// dating is allowed. Most flexible and naturally imperative, but the compiler
	/// cannot restrict access based on the person's current runtime state.
	///
	/// In other words the method is always available and the business logic handles whether "date"
	/// does affect a change in a person.
	impl<C, S> s::Person<C, S> {
		pub fn date_of_runtime(&mut self, other: &mut s::Person<C, S>) {
			println!("date_of_runtime");
			if self.martial_status == e::RelationshipStatus::Single
				&& other.martial_status == e::RelationshipStatus::Single
			{
				self.martial_status = e::RelationshipStatus::Dating;
				other.martial_status = e::RelationshipStatus::Dating;
			}
		}
		pub fn families(&self) -> &Vec<Weak<RefCell<s::FamilyMember<s::Context, s::State>>>> {
			&self.families
		}
		pub fn in_ctx<NC>(self, context: NC) -> s::Person<NC, S> {
			// ~1. Stable
			s::Person {
				context,
				state: self.state,
				name: self.name,
				gender: self.gender,
				families: self.families,
				children: self.children,
				martial_status: self.martial_status,
			}
			// ~2. Unstable Nightly
			// #![feature(type_changing_struct_update)]
			// https://github.com/rust-lang/rust/issues/86555
			// s::Person { context, ...self }
		}
		pub fn in_state<NS>(self, state: NS) -> s::Person<C, NS> {
			s::Person {
				state,
				children: self.children,
				context: self.context,
				families: self.families,
				gender: self.gender,
				martial_status: self.martial_status,
				name: self.name,
			}
		}
		pub fn state(&self) -> &S {
			&self.state
		}
		pub fn to_married(mut self) -> Self {
			self.martial_status = e::RelationshipStatus::Married;
			self
		}
	}

	impl Parent for s::Parent<s::Context, s::State> {
		fn children<C, S>(&self) -> &[s::Child<C, S>] {
			// There are two items in my completion panel when I type
			// self.children.
			//
			// What would u say the difference between these two are?
			// self.children(); // (as Person)
			// self.children(); // (as Parent)

			// When I command click on children above, they both take me to the same thing.
			// The children property on the Parent Struct.
			//
			// pub struct Parent<C, S> {
			// 	pub context: C,
			// 	pub state: S,
			// 	pub spouse: Rc<Person<CtxPerson, StatePerson>>,
			// 	pub children: Vec<Rc<Child<CtxChild, StateChild>>>,
			// }

			todo!("impl t::Parent for s::Parent<s::Context, s::State> children")
		}
		fn consider_request(&self, request: &str) -> bool {
			todo!("")
		}
		fn discipline_child<C, S>(&mut self, child: &s::Child<C, S>, reason: &str) {
			todo!("")
		}
	}
	/// "For every possible C and S, s::Person<C, S> implements Person."
	///
	/// "A Person that participates in my dynamic-dispatch/Any system must own
	/// its state and context, or otherwise contain only static data."
	///
	/// Fixes:
	/// impl<C, S> t::Person for s::Person<C, S>
	///
	/// Errored:
	/// "the parameter type `C` may not live long enough"
	///
	impl<C: 'static, S: 'static> Person for s::Parent<C, S> {
		/// trait method
		fn name(&self) -> &str {
			// Self::name(&self);
			// an inherent method on Parent.
			s::Parent::<C, S>::name(self)
		}

		fn state(&self) -> &dyn Any {
			&self.state
		}

		fn as_any(&self) -> &dyn Any {
			self
		}
	}

	impl<C: 'static, S: 'static> Person for s::Person<C, S> {
		fn name(&self) -> &str {
			&self.name
		}
		fn state(&self) -> &dyn Any {
			&self.state
		}
		fn as_any(&self) -> &dyn Any {
			self
		}
	}
	impl<C: 'static, S: 'static> Spouse for s::Person<C, S> {
		fn name(&self) -> &str {
			"Dearest"
		}
		fn spouse(&self) -> Option<String> {
			let member = self.families.first()?.upgrade()?;
			let member = member.borrow();
			let family = member.family.upgrade()?;
			let family = family.borrow();
			family.members.iter().find_map(|member| {
				let member = member.borrow();
				let person = member.person.borrow();
				if person.name != self.name {
					Some(person.name.clone())
				} else {
					None
				}
			})
		}
		fn discuss_with_spouse(&self, topic: &str) {
			let me = self.name.clone();
			let spouse = self.spouse().expect("Married person should have a spouse");
			println!("{me} discussing with {spouse}: {topic}");
		}
	}

	impl Single for s::Single {
		fn date(&self) {
			todo!("date")
		}
		fn accept_date(&self) {
			todo!("accept_date")
		}
		fn get_to_know(&self) {
			todo!("get_to_know")
		}
	}
	/// Scoped capability:
	/// `date_of_lifetimed_singleref` is only available after a runtime check creates
	/// a `SingleRef`. It preserves the same Person while temporarily narrowing the
	/// available API: maximum state-awareness without converting the underlying value.
	impl<'a, C, S> s::SingleRef<'a, C, S> {
		pub fn date_of_lifetimed_singleref(self, other: &mut s::Person<C, S>) {
			println!("date_of_lifetimed_singleref");
			self.person.martial_status = e::RelationshipStatus::Dating;
			other.martial_status = e::RelationshipStatus::Dating;
		}
	}

	impl<C, S> s::Work<C, S> {
		pub fn new(context: C, state: S, person: s::Person<s::Context, s::State>) -> Self {
			let callsign = if person.name() == "Tom" {
				"Maverick".to_string()
			} else {
				person.name().to_string()
			};

			let person = s::Person::new(
				s::CtxWork::default(),
				s::StateWork::default(),
				person.name,
				person.gender,
			);

			Self {
				context,
				state,
				person,
				name: callsign.clone(),
				callsign,
			}
		}
	}
	impl s::Work<s::CtxWork, s::StateWork> {
		// specifically available to this instantiation
		pub fn execute(&self) {
			// ...
		}
	}

	// =======================================================
	// "exact type" vs "family of types"
	// =======================================================
	impl<C, S> s::Foo<C, S> {
		pub fn new(context: C, state: S) -> Self {
			Self {
				context,
				state,
				_context: PhantomData,
				_state: PhantomData,
			}
		}
		pub fn new_where(context: s::Context, state: s::State) -> Self
		where
			C: From<s::Context>,
			S: From<s::State>,
		{
			Self {
				context: context.into(),
				state: state.into(),
				_context: PhantomData,
				_state: PhantomData,
			}
		}

		pub fn new_with_type<NC, NS>(context: C, state: S) -> Self {
			Self {
				context,
				state,
				_context: PhantomData,
				_state: PhantomData,
			}
		}
		pub fn convert_from<NC, NS>(context: NC, state: NS) -> Self
		where
			C: From<NC>,
			S: From<NS>,
		{
			Self {
				context: context.into(),
				state: state.into(),
				_context: PhantomData,
				_state: PhantomData,
			}
		}
	}
	/// This implementation applies specifically to
	/// Person<C, Single>.
	impl<C> s::Foo<C, s::State> {}
	/// This implementation applies to
	/// Person<C, S> for any type S that implements the Single trait.
	impl<C, S> s::Foo<C, S>
	where
		S: Single,
	{
		pub fn single_traits_implemented() {}
	}
}

mod structs {
	use super::{e, *};

	impl From<C> for Context {
		fn from(_: C) -> Self {
			Context
		}
	}

	impl From<S> for State {
		fn from(_: S) -> Self {
			State {
				martial_status: e::RelationshipStatus::default(),
			}
		}
	}

	/// ## [C]: Context filler
	///
	/// Non-trivial apps eventually end up modeling context & state.
	///
	/// With that in mind from the beginning with the goal of
	/// building robust abstractions & implementations we use generic context [C] and [S]
	/// for all structs.
	///
	/// At the cost of added verboseness we can achieve typestate safety much more easily later downstream.
	///
	pub struct C;

	/// ## [S] State filler
	///
	pub struct S;

	/// ## [Context]: Typestate placeholder
	///
	/// Type safety is a global property of a programming language
	/// or code that prevents invalid operations on data types, whereas
	/// typestate is a specific design pattern that encodes an object's current
	/// lifecycle state into its type to restrict operations at compile time.
	///
	/// When a context stabilizes swap the generic [C] for
	/// a concrete implementation of Context in order to reap typestate safety benefits.
	///
	#[derive(Clone, Copy, Debug, Default)]
	pub struct Context;

	#[derive(Clone, Copy, Debug, Default)]
	pub struct State {
		pub martial_status: e::RelationshipStatus,
	}

	#[derive(Clone, Debug)]
	pub struct Child<C, S> {
		pub context: C,
		pub state: S,
		pub person: Person<CtxPerson, StatePerson>,
		pub parents: Vec<Parent<CtxParent, CtxParent>>,
	}

	#[derive(Clone, Default, Debug)]
	pub struct CtxChild;
	#[derive(Clone, Default, Debug)]
	pub struct CtxFamily;
	#[derive(Clone, Copy, Debug, Default)]
	pub struct CtxWork;
	#[derive(Clone, Default, Debug)]
	pub struct CtxParent;
	#[derive(Clone, Default, Debug)]
	pub struct CtxPerson;

	#[derive(Clone, Default, Debug)]
	pub struct StateChild;
	#[derive(Clone, Default, Debug)]
	pub struct StateFamily;
	#[derive(Clone, Default, Debug)]
	pub struct StateParent;
	#[derive(Clone, Default, Debug)]
	pub struct StatePerson {
		pub martial_status: e::RelationshipStatus,
	}
	#[derive(Clone, Copy, Debug, Default)]
	pub struct StateWork;

	/// In the context of me being a child of my parents,
	/// The C and S of family is generic. But the C and S of "members", me, should be different.
	/// So should I do that here by hardcoding, or should I make this a where clause again...?
	///
	/// 	Family
	///   │
	///   │ 1 : many
	///   ▼
	/// FamilyMember
	///
	#[derive(Clone, Debug, Default)]
	pub struct Family<CtxFamily, StateFamily> {
		pub context: CtxFamily,
		pub state: StateFamily,
		pub members: Vec<Rc<RefCell<FamilyMember<Context, State>>>>,
	}
	/// Person
	///    ▲
	///    │
	///    │ many
	///    │
	/// FamilyMember
	///    │
	///    │ many
	///    │
	///    ▼
	/// Family
	#[derive(Clone, Default, Debug)]
	pub struct FamilyMember<C, S> {
		pub context: C,
		pub state: S,
		pub person: Rc<RefCell<Person<CtxPerson, StatePerson>>>,
		pub family: Weak<RefCell<Family<CtxFamily, StateFamily>>>,
	}

	#[derive(Clone, Default, Debug)]
	pub struct Person<C, S> {
		pub context: C,
		pub state: S,
		pub name: String,
		pub gender: e::Gender,
		pub martial_status: e::RelationshipStatus,

		/// Multiple things can own this Family, and those owners can obtain
		/// mutable access to it at runtime.
		pub families: Vec<Weak<RefCell<FamilyMember<Context, State>>>>,
		pub children: Vec<Rc<RefCell<Child<Context, State>>>>,
	}
	impl<C, S> Person<C, S> {
		/// Both ava.name & ava.name() work
		/// But the representation of an object should not necessarily be part of its public API.
		///
		/// Adding this fixes that issue.
		///
		/// But then a person's contextual "name" behavior breaks/conflicts.
		///
		/// - A persons parent name, "dad"
		/// - A persons work name, "maverick"
		/// - A persons spouses name, "honey"
		///
		pub fn name(&self) -> &str {
			&self.name
		}
		// Although name is private, it can be edited here because
		// the struct and the impl are in the same mod
		pub fn new(context: C, state: S, name: String, gender: e::Gender) -> Self {
			Self {
				context,
				state,
				name,
				gender,
				children: vec![],
				families: Vec::new(),
				martial_status: e::RelationshipStatus::Single,
			}
		}
	}

	#[derive(Clone, Debug)]
	pub struct Parent<C, S> {
		pub context: C,
		pub state: S,
		pub spouse: Rc<Person<CtxPerson, StatePerson>>,
		pub children: Vec<Rc<Child<CtxChild, StateChild>>>,
	}
	pub struct Spouse<C, S> {
		pub context: C,
		pub state: S,
		pub person: Person<Context, State>,
		pub partner: Person<Context, State>,
		pub family: Family<Context, State>,
	}

	// #[derive(Clone, Copy, Debug, Default)]
	// pub struct State {
	// 	pub martial_status: e::RelationshipStatus,
	// }

	#[derive(Clone, Copy, Debug, Default)]
	pub struct MarriageContext;

	#[derive(Clone, Copy, Debug, Default)]
	pub struct Single;

	/// "For the duration of this borrow, I have a runtime-verified Single capability."
	pub struct SingleRef<'a, C, S> {
		pub person: &'a mut Person<C, S>,
	}
	#[derive(Clone, Copy, Debug, Default)]
	pub struct Dating;
	#[derive(Clone, Copy, Debug, Default)]
	pub struct Married;

	#[derive(Debug)]
	pub struct Work<C, S> {
		pub context: C,
		pub state: S,
		pub callsign: String,
		pub person: Person<CtxWork, StateWork>,
		pub name: String,
	}

	pub struct WorkPerson<C, S> {
		pub person: Person<C, S>,
		pub context: CtxWork,
		pub state: StateWork,
		pub callsign: String,
	}

	pub struct Foo<C, S> {
		pub _context: PhantomData<C>,
		pub _state: PhantomData<S>,
		pub context: C,
		pub state: S,
	}
}

mod traits {
	use super::{e, s, *};

	pub trait Child {}
	pub trait Family {
		type Context;
		type State;
		fn have_child(
			&mut self,
			name: impl Into<String>,
			gender: e::Gender,
		) -> s::Child<Self::Context, Self::State>;
	}
	pub trait Person: Any {
		fn name(&self) -> &str;
		fn state(&self) -> &dyn Any;
		fn as_any(&self) -> &dyn Any;
		// fn as_parent(&self) -> Option<&dyn Parent> {
		// 	None
		// }
		fn children(&self) {}
	}

	pub trait Parent: Person {
		fn children<C, S>(&self) -> &[s::Child<C, S>];
		fn consider_request(&self, request: &str) -> bool;
		fn discipline_child<C, S>(&mut self, child: &s::Child<C, S>, reason: &str);
	}

	pub trait RelationshipState {
		fn status() -> e::RelationshipStatus;
	}

	pub trait CanDate {}
	pub trait CanMarry {}
	pub trait CanDivorce {}

	pub trait Single {
		fn date(&self);
		fn accept_date(&self);
		fn get_to_know(&self);
	}
	pub trait Dating {
		fn dating(&self) {}
	}
	pub trait Spouse: Person {
		fn name(&self) -> &str;
		fn spouse(&self) -> Option<String>;
		fn discuss_with_spouse(&self, topic: &str);
	}
	pub trait Context: Any {}
	pub trait State: Any {}
	pub trait MarriedState: State {
		fn spouse_name(&self) -> &str;
	}
}

fn type_conversion() {
	/// ============================================================
	/// 1. INFERENCE + COERCION
	/// ============================================================
	///
	/// Not every "conversion" you see in Rust is actually a
	/// conversion. Sometimes the compiler simply:
	///
	///   • infers the type of value from surrounding constraints
	///   • performs a language-defined coercion
	///
	/// Integer literals are initially type-flexible:
	///
	///     let n = 10;
	///
	/// If nothing else constrains `n`, Rust defaults it to i32.
	///
	/// But here:
	///
	///     let x: i64 = n;
	///
	/// the expected type of `x` constrains `n`, so the compiler
	/// infers the original literal as i64:
	///
	///     let n: i64 = 10;
	///     let x: i64 = n;
	///
	/// NO i32 -> i64 conversion happened.
	///
	/// Coercion is different: Rust can implicitly change certain
	/// related types in specific situations.
	///
	/// For example, &[T; N] can coerce to &[T], and &String can
	/// deref-coerce to &str.
	fn inference_and_coercion() {
		// Integer inference:
		let n = 10;
		let x: i64 = n;

		// `n` was inferred as i64 because of `x`.
		println!("n = {n}, x = {x}");

		// Borrowing:
		//
		// `&s` borrows `s`.
		//
		// With no expected type, Rust simply infers:
		//
		//     let slice = &s;
		//     // slice: &String

		let s = String::from("hello");
		let slice = &s;
		println!("slice = {slice}");

		// Deref coercion:
		//
		// When the expected type is &str, Rust can use
		// String's Deref<Target = str> implementation:
		//
		//     &String → &str

		let s = String::from("hello");
		let slice: &str = &s;
		println!("slice = {slice}");

		// Deref coercion isn't something that automatically happens whenever
		// you take a reference to a String. It happens when type checking
		// requires a compatible target type.

		// Unsizing coercion:
		let array = [1, 2, 3];

		// &[i32; 3] -> &[i32]
		let slice: &[i32] = &array;

		println!("slice = {slice:?}");
	}

	// ============================================================
	// 2. `as` CASTING
	// ============================================================
	//
	// `as` performs an explicit cast.
	//
	// Unlike the previous example, the source type is already
	// established:
	//
	//     let n: i32 = 10;
	//
	// Therefore:
	//
	//     n as i64
	//
	// really is an i32 -> i64 cast.
	//
	// `as` is commonly used for primitive numeric conversions
	// and certain pointer casts.
	//
	// It is NOT the same mechanism as From/Into.
	fn as_casts() {
		let n: i32 = 10;

		// Explicit numeric cast:
		let x: i64 = n as i64;

		println!("n = {n}, x = {x}");

		// `as` can also perform conversions where information may
		// be lost. For example:
		let large: i32 = 300;
		let small: u8 = large as u8;

		println!("300 as u8 = {small}");

		// This is fundamentally different from:
		//
		//     let small: u8 = large.into();
		//
		// because Into/From expresses a trait-based conversion,
		// while `as` is the language's explicit casting syntax.
	}

	// ============================================================
	// 3. From / Into / TryFrom / TryInto
	// ============================================================
	//
	// From represents an infallible conversion:
	//
	//     From<A> for B
	//
	// means:
	//
	//     A -> B
	//
	// and Rust automatically gives A:
	//
	//     Into<B>
	//
	// Therefore:
	//
	//     B::from(a)
	//
	// and:
	//
	//     a.into()
	//
	// represent the same conversion.
	//
	// TryFrom is the fallible counterpart:
	//
	//     A -> Result<B, Error>
	//
	// and therefore:
	//
	//     TryInto<B>
	//
	// is also available.

	fn trait_conversions() {
		// --------------------------------------------------------
		// From / Into
		// --------------------------------------------------------

		let n: i32 = 10;

		// Explicit destination type:
		let x = i64::from(n);

		println!("x = {x}");

		// Into relies on the destination type to determine T:
		let n: i32 = 10;
		let x: i64 = n.into();

		println!("x = {x}");

		// Conceptually:
		//
		//     impl From<i32> for i64
		//                 │
		//                 └── gives i32::into() -> i64
		//
		// You normally implement From rather than implementing
		// Into directly.

		// --------------------------------------------------------
		// TryFrom / TryInto
		// --------------------------------------------------------

		let n: i64 = 100;

		// i64 -> u8 can fail because u8 only holds 0..=255.
		let x: u8 = u8::try_from(n).unwrap();

		println!("x = {x}");

		// Same operation through TryInto:
		let n: i64 = 100;

		let x: u8 = n.try_into().unwrap();

		println!("x = {x}");

		// Unlike From:
		//
		//     From<i64> for u8
		//
		// would imply that every i64 can become a u8 without failure.
		//
		// Instead:
		//
		//     TryFrom<i64> for u8
		//
		// returns:
		//
		//     Result<u8, Error>
		//
		// because some values cannot be represented.
	}

	// ============================================================
	// 4. BORROWING / VIEW CONVERSIONS
	// ============================================================
	//
	// AsRef and AsMut are different from From/Into.
	//
	// They don't generally consume one type and create another.
	// They provide a borrowed view:
	//
	//     AsRef<T>
	//         A -> &T
	//
	//     AsMut<T>
	//         A -> &mut T
	//
	// This is extremely useful for generic APIs.
	//
	// `Deref` is closely related because deref coercion lets Rust
	// automatically turn things such as:
	//
	//     &String
	//
	// into:
	//
	//     &str
	//
	// when the expected type requires it.

	fn borrowing_conversions() {
		// --------------------------------------------------------
		// AsRef
		// --------------------------------------------------------

		fn print_text<T: AsRef<str>>(value: T) {
			println!("{}", value.as_ref());
		}

		let owned = String::from("hello");

		// String can be viewed as &str:
		print_text(owned);

		// &str also works:
		print_text("hello");

		// The important idea is that AsRef isn't saying:
		//
		//     String -> str
		//
		// as an owned conversion.
		//
		// It says:
		//
		//     String -> &str
		//
		// as a borrowed view.

		// --------------------------------------------------------
		// AsMut
		// --------------------------------------------------------

		fn modify_text<T: AsMut<str>>(value: &mut T) {
			let text: &mut str = value.as_mut();

			text.make_ascii_uppercase();
		}

		let mut text = String::from("hello");

		modify_text(&mut text);

		println!("{text}"); // HELLO

		println!("{text}");

		// --------------------------------------------------------
		// Deref coercion
		// --------------------------------------------------------

		let text = String::from("hello");

		// Rust automatically deref-coerces:
		//
		//     &String
		//        ↓
		//     &str
		//
		fn takes_str(value: &str) {
			println!("{value}");
		}

		takes_str(&text);
	}

	// Run the four demonstrations.
	inference_and_coercion();
	as_casts();
	trait_conversions();
	borrowing_conversions();
}

mod state_context_paradigm {
	/// "Foo is parameterized by the conditions under which Foo exists and behaves."
	///
	/// state: The raw, current data values of an object or application.
	/// - To track what the current condition of something is.
	/// - Highly mutable; changes constantly based on user actions or logic.
	/// - Internal: What the specific module or object knows about itself.
	///
	/// context: The surrounding environment, configurations, or metadata.
	/// - To explain where, why, or how something is operating.
	/// - Relatively stable; acts as a wrapper or container for operations.
	/// - External / Pervasive: The broader system constraints impacting the object.

	/// A field's type tells Rust what type of thing the relationship
	/// contains. A where clause tells Rust what properties that type must have.
	///
	struct Foo<C, S> {
		state: S,
		context: C,
	}
	// ## 1. When Context Drives State (The Rules & Environment)
	// From a design and initialization perspective, context always drives state. Context defines the sandbox.
	// It dictates how state is allowed to behave, what values are legal, and what state should be initialized.

	// * The Rule Maker: If the context is Production Environment, the state variable is_debug_mode is forced to false.
	// * The Permission Filter: If the user context is Guest, the application state available_features_list is restricted to only 3 items instead of 100.
	// * The Interrupter: In Go or Rust async programming, if a network Context times out or is canceled, it instantly
	// forces the database worker's state from Processing to Aborted.

	// Summary: Context acts as the parent constraints. State must comply with the rules of the context it lives in.
	// ------------------------------
	// ## 2. When State Drives Context (The Catalyst for Change)
	// From a runtime and behavioral perspective, accumulated state changes eventually shift the context. When individual data points hit a certain threshold, the entire environment morphs.

	// * The Threshold Shift: A user's account state variable failed_login_attempts increments from 4 to 5. This state change instantly flips the application context from Standard Login Screen to Secured Captcha Lockout.
	// * The Business Logic Evolution: An e-commerce order state moves from Items Assembled to Ship. The context of that order now shifts from an "Inventory Management" context to a "Logistics & Delivery Tracking" context.
	// * The Physical World: In a video game, when the player's coordinate states cross a specific boundary, the game engine unloads the "Forest" context and loads the "Dungeon" context.

	// Summary: State acts as the evolutionary data. When state changes drastically enough, it forces a context switch.
	// ------------------------------
	// ## The Continuous Loop
	// In a well-designed system, they dance in a perpetual circle:

	// [ Context ] ---> (Configures & Restricts) ---> [ State ]
	//     ^                                              |
	//     |                                              v
	//     +--------- (Mutates & Eventually Triggers) ----+

	//    1. You log into an app. Your Context is now "Authenticated User."
	//    2. Because of this context, you are allowed to modify your profile State (e.g., changing your email address).
	//    3. You change your email to an unverified one. This State change triggers a system event.
	//    4. The system updates your Context back to "Unverified User," locking down your permissions until you click a link.

	// ## How to apply this to your design
	// When writing code (like your Rust module), ask yourself:

	// * Is this data a rule/constraint/environment? Make it Context (pass it down, use it to check boundaries).
	// * Is this data an independent, fluctuating value? Make it State (track it locally, update it often).

	// Are you trying to design a specific system right now—like a state machine, an API router, or a game loop—where you are struggling to decide if a piece of data belongs in the context or the state?

	// +-------------------------------------------------------------+

	// | CONTEXT (The Environment)                                   |
	// | e.g., Device Type, Region, Feature Flags, Network Status     |
	// |                                                             |
	// |   +-----------------------------------------------------+   |
	// |   | STATE MACHINE (The Core Logic)                      |   |
	// |   | e.g., Current State: [Loading] -> [Active]          |   |
	// |   |                                                     |   |
	// |   |   +---------------------------------------------+   |   |
	// |   |   | INTERNAL STATE DATA (The Payload)           |   |   |
	// |   |   | e.g., Cache files, Form Inputs, UI Items    |   |   |
	// |   +---+---------------------------------------------+---+   |
	// +-------------------------------------------------------------+

	// A generic parameter should be propagated
	// only across a relationship when the participating abstractions genuinely share that dimension.

	// Person<C,S>
	// Family<C,S>
	// FamilyMember<C,S>

	// appropriate when

	// Person state == Family state == Membership state

	// This means there are multiple state dimensions.
	// Each model has its own state.
	// Person<C,S₁>
	// Family<C,S₂>
	// FamilyMember<C,?>
}
