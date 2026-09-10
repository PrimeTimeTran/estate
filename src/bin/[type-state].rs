#![allow(warnings)]
// https://www.youtube.com/watch?v=3biW5NkNnrk

use std::{
	any::Any,
	cell::RefCell,
	marker::PhantomData,
	ops::Deref,
	rc::{Rc, Weak},
};

mod enums {
	use super::{i, s, t, *};
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
		Divorced,
	}
}

mod impls {
	use super::{e, s, t, *};
	impl<C, S> Deref for s::Child<C, S> {
		type Target = s::Person<s::CtxPerson, s::StatePerson>;
		fn deref(&self) -> &Self::Target {
			&self.person
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
			let mut person = s::Person::new(context.clone(), state.clone(), name.into(), gender);
			let context = s::Context::default();
			let state = s::State::default();
			let child = s::Child::new(context, state, person);
			child
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

	/// =======================================================
	/// Martial Transition
	/// =======================================================
	///
	impl<C: Clone, S: Clone> s::Person<C, S> {
		pub fn marry(
			self,
			partner: s::Person<C, S>,
		) -> (
			Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			Rc<RefCell<s::Person<s::CtxPerson, s::StatePerson>>>,
			Rc<RefCell<s::Family<s::CtxFamily, s::StateFamily>>>,
		) {
			// Keep the actual people we're going to put into the family.
			let p1 = Rc::new(RefCell::new(
				self
					.in_ctx(s::CtxPerson::default())
					.in_state(s::StatePerson::default())
					.to_married(),
			));
			let p2 = Rc::new(RefCell::new(
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
				.add_member(p1.clone(), family_weak.clone());
			family.borrow_mut().add_member(p2.clone(), family_weak);
			(p1, p2, family)
		}

		pub fn breakup() {}
	}
	impl<C: Clone> s::Person<C, s::Married> {
		pub fn divorce() {}
	}
	impl<C, S> s::Person<C, S> {
		pub fn date(&mut self, other: &mut s::Person<C, S>) {
			if self.martial_status == e::RelationshipStatus::Single
				&& other.martial_status == e::RelationshipStatus::Single
			{
				self.martial_status = e::RelationshipStatus::Dating;
				other.martial_status = e::RelationshipStatus::Dating;
			}
		}
	}
	impl<C, S> s::Person<C, S> {
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
		pub fn to_married(mut self) -> Self {
			self.martial_status = e::RelationshipStatus::Married;
			self
		}
		pub fn in_ctx<NC>(self, context: NC) -> s::Person<NC, S> {
			s::Person {
				context,
				state: self.state,
				name: self.name,
				gender: self.gender,
				families: self.families,
				children: self.children,
				martial_status: self.martial_status,
			}
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
	}
	impl<C, S> s::Person<C, S> {
		pub fn name(&self) -> &str {
			&self.name
		}
		pub fn state(&self) -> &S {
			&self.state
		}
		pub fn families(&self) -> &Vec<Weak<RefCell<s::FamilyMember<s::Context, s::State>>>> {
			&self.families
		}
	}
	impl t::Parent for s::Parent<s::Context, s::State> {
		fn children<C, S>(&self) -> &[s::Child<C, S>] {
			todo!("")
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
	/// its state and context, or otherwise contain only 'static data."
	///
	/// Fixes:
	/// impl<C, S> t::Person for s::Person<C, S> {
	///
	/// Errored:
	/// "the parameter type `C` may not live long enough"
	///
	impl<C: 'static, S: 'static> t::Person for s::Parent<C, S> {
		fn name(&self) -> &str {
			s::Parent::<C, S>::name(self)
		}

		fn state(&self) -> &dyn Any {
			&self.state
		}

		fn as_any(&self) -> &dyn Any {
			self
		}
	}
	impl<C: 'static, S: 'static> t::Person for s::Person<C, S> {
		fn name(&self) -> &str {
			&self.name
		}
		fn as_any(&self) -> &dyn Any {
			self
		}
		fn state(&self) -> &dyn Any {
			&self.state
		}
	}
	impl<C: 'static, S: 'static> t::Spouse for s::Person<C, S> {
		// fn spouse(&self) -> Option<String> {
		// 	None
		// }
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
		fn name(&self) -> &str {
			"Dearest"
		}
	}
	impl<C, S> s::Work<C, S> {
		pub fn new(
			context: C,
			state: S,
			person: s::Person<s::Context, s::State>,
			callsign: String,
		) -> Self {
			let person = s::Person {
				context: s::CtxWork::default(),
				state: s::StateWork::default(),
				children: vec![],
				families: vec![],
				gender: person.gender,
				martial_status: person.martial_status,
				name: person.name,
			};

			Self {
				context,
				state,
				person,
				name: callsign.clone(),
				callsign,
			}
		}
	}
}

mod structs {
	use super::{e, i, t, *};

	#[derive(Clone, Default, Debug)]
	pub struct CtxParent;
	#[derive(Clone, Default, Debug)]
	pub struct StateParent;
	#[derive(Clone, Default, Debug)]
	pub struct CtxChild;
	#[derive(Clone, Default, Debug)]
	pub struct StateChild;
	#[derive(Clone, Default, Debug)]
	pub struct CtxPerson;
	#[derive(Clone, Default, Debug)]
	pub struct StatePerson {
		pub martial_status: e::RelationshipStatus,
	}
	#[derive(Clone, Default, Debug)]
	pub struct CtxFamily;
	#[derive(Clone, Default, Debug)]
	pub struct StateFamily;

	#[derive(Clone, Debug)]
	pub struct Child<C, S> {
		pub context: C,
		pub state: S,
		pub person: Person<CtxPerson, StatePerson>,
		pub parents: Vec<Parent<CtxParent, CtxParent>>,
	}

	#[derive(Clone, Copy, Debug, Default)]
	pub struct Household;

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
		// pub state2: StatePerson,
		pub name: String,
		pub gender: e::Gender,
		pub martial_status: e::RelationshipStatus,

		/// Multiple things can own this Family, and those owners can obtain
		/// mutable access to it at runtime.
		pub families: Vec<Weak<RefCell<FamilyMember<Context, State>>>>,
		pub children: Vec<Rc<RefCell<Child<Context, State>>>>,
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

	#[derive(Clone, Copy, Debug, Default)]
	pub struct Context;

	#[derive(Clone, Copy, Debug, Default)]
	pub struct State {
		pub martial_status: e::RelationshipStatus,
	}

	#[derive(Clone, Copy, Debug, Default)]
	pub struct MarriageContext;

	#[derive(Clone, Copy, Debug, Default)]
	pub struct Single;
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

	#[derive(Clone, Copy, Debug, Default)]
	pub struct CtxWork;

	#[derive(Clone, Copy, Debug, Default)]
	pub struct StateWork;

	pub struct WorkPerson<C, S> {
		pub person: Person<C, S>,
		pub context: CtxWork,
		pub state: StateWork,
		pub callsign: String,
	}
}

mod traits {
	use super::{e, i, s, *};
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

	pub trait Spouse: Person {
		fn name(&self) -> &str;
		fn spouse(&self) -> Option<String>;
		fn discuss_with_spouse(&self, topic: &str);
	}
	pub trait Parent: Person {
		fn children<C, S>(&self) -> &[s::Child<C, S>];
		fn consider_request(&self, request: &str) -> bool;
		fn discipline_child<C, S>(&mut self, child: &s::Child<C, S>, reason: &str);
	}
	pub trait State: Any {}
	pub trait MarriedState: State {
		fn spouse_name(&self) -> &str;
	}
}

use self::traits::*;
use self::{enums as e, impls as i, structs as s, traits as t};

fn main() {
	use t::*;
	let context = s::Context::default();
	let state = s::State::default();
	let mut p1 = s::Person::new(
		context.clone(),
		state.clone(),
		String::from("Bob"),
		e::Gender::Male,
	);
	// p1.in_state(state);
	let mut p2 = s::Person::new(context, state, String::from("Christine"), e::Gender::Female);

	// ─────────────────────────────────────────
	// Birth / early life
	// ─────────────────────────────────────────

	println!("{}'s state is {:?}", p1.name(), p1.state());
	println!("{}'s state is {:?}", p2.name(), p2.state());

	// ─────────────────────────────────────────
	// Work Context
	// ─────────────────────────────────────────

	let work_context = s::CtxWork::default();
	let work_state = s::StateWork::default();

	let work_bob = s::Work::new(
		work_context,
		work_state,
		p1.clone(),
		String::from("Maverick"),
	);
	println!("Work Bob's government name {:?}", work_bob.person.name());
	println!("Work Bob's name is {}", work_bob.name);
	println!("Work Bob's call sign {}", work_bob.callsign);

	// Errors because unmarried.
	// p1.discuss_with_spouse("Where should we live?");
	// p2.discuss_with_spouse("Where should we live?");

	// ─────────────────────────────────────────
	// They get married
	// ─────────────────────────────────────────
	p1.date(&mut p2);
	println!("{} is {:?}", p1.name, p1.martial_status);
	println!("{} is {:?}", p2.name, p2.martial_status);
	let (p1, p2, family) = p1.marry(p2);
	println!("{} is {:?}", p1.borrow().name, p1.borrow().martial_status);
	println!("{} is {:?}", p2.borrow().name, p2.borrow().martial_status);

	p1.borrow().discuss_with_spouse("Where should we live?");
	p2.borrow().discuss_with_spouse("Should we have children?");

	// ─────────────────────────────────────────
	// They have a child
	// ─────────────────────────────────────────

	let child = family.borrow_mut().have_child("Alice", e::Gender::Female);

	println!(
		"{} is a parent {:?}",
		p1.borrow().name,
		p1.borrow().martial_status
	);
	println!(
		"{} is a parent {:?}",
		p2.borrow().name,
		p2.borrow().martial_status
	);
	println!("{} is a child {:?}", child.name(), child.martial_status);

	// p1.consider_request("Can I stay up late?");
	// p1.discipline_child("Alice", "She stayed up too late");

	// println!("{}'s children: {:?}", p1.name(), p1.children());
}

fn main2() {
	// 	let p1 = s::Person {
	// 		name: String::from("Ann"),
	// 	};
	// 	println!("p1 {}", p1.name);
	// 	let person: &dyn t::Person = &p1;
	// 	// let parent: &dyn t::Parent = &p1;
	// 	// let spouse: &dyn t::Spouse = &p1;

	// 	// ## Static Dispatch
	// 	// - Hover Children: Go to C & P
	// 	// p1.children;

	// 	// ## Dynamic Dispatch
	// 	// let p2: &(dyn Parent + 'static)
	// 	let p2: &dyn t::Person = &p1;
	// 	// - Hover Children: Self = dyn Parent + 'static
	// 	p2.children();

	// 	// ## Dynamic Dispatch
	// 	// Owning the trait object
	// 	// - Type Erased interface
	// 	let p3: Box<dyn t::Parent> = Box::new(s::Parent {
	// 		name: String::from("Bob"),
	// 		children: vec![],
	// 	});
	// 	// - Hover Children: Self = dyn Parent + 'static
	// 	p3.children();
	// 	p3.children();
	// 	p3.consider_request("Can I go?");
	// 	// let c3 = C {
	// 	// 	name: String::from("child"),
	// 	// 	parents: vec![p3],
	// 	// };
	// 	// p3.discipline_child(&c3, "You broke the rules");
}

fn type_conversion() {
	// i32
	// ↓
	// Into<i64>
	// ↓
	// i64
	let n: i32 = 10;
	let x: i64 = n.into();

	// trait Into<T> {
	//   fn into(self) -> T;
	// }
	// impl From<A> for B {
	//   fn from(a: A) -> B {
	//       ...
	//   }
	// }
	// let b: B = a.into();
}

fn deref() {
	// What Box does have is Deref
	// Rust's dereferencing/coercion machinery allows the method call to work through the Box.
}

mod example {
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
	// Role as classification
	// Role as relationship
	// Role as capability
	struct Foo<S, C> {
		state: S,
		context: C,
	}
	// You hit the nail on the head: it absolutely depends, and it is a continuous loop. It is the software engineering equivalent of "nature vs. nurture."
	// Instead of one driving the other permanently, think of them as an interactive cycle:
	// Context dictates what states are possible or valid, while State transitions eventually change the Context.
	// Here is the breakdown of how they drive each other depending on the lens you look through.
	// ------------------------------
	// ## 1. When Context Drives State (The Rules & Environment)
	// From a design and initialization perspective, context always drives state. Context defines the sandbox. It dictates how state is allowed to behave, what values are legal, and what state should be initialized.

	// * The Rule Maker: If the context is Production Environment, the state variable is_debug_mode is forced to false.
	// * The Permission Filter: If the user context is Guest, the application state available_features_list is restricted to only 3 items instead of 100.
	// * The Interrupter: In Go or Rust async programming, if a network Context times out or is canceled, it instantly forces the database worker's state from Processing to Aborted.

	// Summary: Context acts as the parent constraints. State must comply with the rules of the context it lives in.
	// ------------------------------
	// ## 2. When State Drives Context (The Catalyst for Change)
	// From a runtime and behavioral perspective, accumulated state changes eventually shift the context. When individual data points hit a certain threshold, the entire environment morphs.

	// * The Threshold Shift: A user's account state variable failed_login_attempts increments from 4 to 5. This state change instantly flips the application context from Standard Login Screen to Secured Captcha Lockout.
	// * The Business Logic Evolution: An e-commerce order state moves from Items Assembled to Shipped. The context of that order now shifts from an "Inventory Management" context to a "Logistics & Delivery Tracking" context.
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
	// Each model has it's own state.
	// Person<C,S₁>
	// Family<C,S₂>
	// FamilyMember<C,?>
}
