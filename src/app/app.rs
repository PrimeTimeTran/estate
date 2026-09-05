use crate::{
	prelude::*,
	r#trait::{Context, EventReceiver},
};

// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
// use crate::NativeRuntime;

// #[cfg(all(feature = "web", target_arch = "wasm32"))]
// use crate::WebRuntime;

// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
// use crate::logger::{LogConfig, Tracer};

/// # App
/// We have 3 primary target platforms which we're developing for. They access resources in similar yet
/// unique ways. Native (Desktop/Laptop), Web (Browser/Client/WASM), Server (Api/Backend) all want access to user,
/// anchors (bookmarks), jobs (tasks), and more.
///
/// Generic indirection adds complication but solves problems downstream.
/// Uses Runtime [Runtime](`crate::r#trait::Runtime`).
///
pub struct App<C>
where
	C: Context,
{
	// C is a generic type parameter.
	// Caller supplies it.
	// App<WebApp>
	// App<NativeApp>
	context: C,
}

impl<C> App<C>
where
	C: Context,
{
	/// ## Generic App which bounds a context wrapped App Instance
	///
	/// "Everything in this impl block only exists for App<C> where C implements Context."
	pub fn host(&self) -> &C::Host {
		self.context.host()
	}
	pub fn runtime(&self) -> &C::Runtime {
		self.context.runtime()
	}
}
impl<C: Context> App<C> {
	/// Initialize a new Estate App instance.
	/// - [Native]
	/// - [Server]
	/// - [Web]
	pub fn new() -> Result<Self> {
		Ok(Self { context: C::new()? })
	}
	fn start(&self) -> Result<()> {
		Ok(())
	}
	pub fn foo(&mut self, args: String) -> Result<()> {
		self.context.foo(args)
		// App::<WebApp>::foo();
		// App::<NativeApp>::foo();
	}
	pub fn bar(&mut self, args: String) -> Result<()> {
		self.context.bar(args)
	}
}
impl<C: Context> App<C> {
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
