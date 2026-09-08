use crate::{prelude::*, r#trait::Context};

// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
// use crate::NativeRuntime;

// #[cfg(all(feature = "web", target_arch = "wasm32"))]
// use crate::WebRuntime;

// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
// use crate::logger::{LogConfig, Tracer};

/// ## [App]
///
/// ### Generics
///
/// - [Context]
/// - [State]
///
/// ### Methods
///
/// -
///  
/// ### Traits

pub struct App<C, S> {
	pub context: C,
	_state: PhantomData<S>,
}

impl<C, S> App<C, S> {
	fn fooo(&self) {
		// Available regardless of C or S
	}
}
impl<C, S> App<C, S> {
	fn stateless() {}
}
impl<C, S> App<C, S>
where
	C: Context<S>,
{
	fn stateful_method(&self) {
		self.context.runtime();
	}
}

impl<C, S> App<C, S> {
	fn foo1(&self) {
		todo!("Inherent Methods")
	}
}

impl<C> App<C, Connected> {
	fn spam(&self) {
		// Only exists on App<C, Connected>
	}
}

impl<C> App<C, Disconnected> {
	fn bar2(&self) {
		// Only exists on App<C, Disconnected>
	}
}

/// # [App]
/// We have 3 primary target platforms which we're developing for. They access resources in similar yet
/// unique ways. Native (Desktop/Laptop), Web (Browser/Client/WASM), Server (Api/Backend) all want access to user,
/// anchors (bookmarks), jobs (tasks), and more.
///
/// Generic indirection adds complication but solves problems downstream.
/// Uses Runtime [Runtime](`crate::r#trait::Runtime`).
///
impl<C, S> App<C, S>
where
	C: Ctx<S>,
{
	/// ## [App::new]
	/// - [Native]
	/// - [Server]
	/// - [Web]
	pub fn new(context: C) -> Result<Self> {
		Ok(Self {
			context,
			_state: PhantomData,
			// state: crate::app::state::State::new(),
		})
	}
	pub fn host(&self) -> &C::Host {
		self.context.host()
	}
	pub fn runtime(&self) -> &C::Runtime {
		self.context.runtime()
	}
}

impl<C, S> App<C, S>
where
	C: Ctx<S>,
{
	pub fn start(&self) -> Result<()> {
		Ok(())
	}
	/// # Running
	/// Ensure estate builds before running
	///
	/// ## Native
	/// - **build:**
	///
	///     `cargo build --bin native --features native`
	///
	/// - **run:**
	///
	///     `cargo run --bin native --no-default-features --features native`
	///
	/// ## Server
	/// - **build:**
	///
	///     `cargo build --bin server --no-default-features --features native`
	///
	/// - **run/start:**
	///
	///     `cargo run --bin server --no-default-features --features native`
	///
	/// ## Web
	/// - **build**
	///
	///   `cargo build --bin web --no-default-features --features web --target wasm32-unknown-unknown`
	///
	/// - **run**
	///
	///   `cargo run --bin native --no-default-features --features native`
	pub fn run(&mut self, args: C::Args) -> Result<()> {
		self.context.run(args)
	}
}
impl<C, S> App<C, S>
where
	C: Ctx<S>,
{
	pub fn foo(&mut self, args: String) -> Result<()> {
		self.context.foo(args)
	}
	pub fn bar(&mut self, args: String) -> Result<()> {
		self.context.bar(args)
	}
}

impl<C, S> App<C, S>
where
	C: Ctx<S>,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("App")
			.field("context", &"<Context>")
			// .field("foo", &self.foo(String::from("Hi")))
			// .field("bar", &self.bar(String::from("bar")))
			.finish()
	}
}
// impl<C> fmt::Debug for App<C>
// where
// 	C: Context,
// 	C::Host: fmt::Debug,    // Requires the associated Host type to be Debug
// 	C::Runtime: fmt::Debug, // Requires the associated Runtime type to be Debug
// {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		f.debug_struct("App")
// 			.field("host", self.host()) // Uses your existing public helper method
// 			.field("runtime", self.runtime()) // Uses your existing public helper method
// 			.finish()
// 	}
// }
