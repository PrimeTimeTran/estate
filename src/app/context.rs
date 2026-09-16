use crate::{e, prelude::*};

// The lifetimes 'a and 'static in your code tell a precise story about memory ownership,
// data borrows, and concurrency safety.
//
// Here is exactly what each lifetime communicates to the Rust compiler and to other developers:
//
// ## 1. What the 'a lifetime tells you
// The 'a lifetime on pub struct AppContext<'a, R: Runtime> tells us that AppContext is a short-lived,
// ephemeral helper object that does not own its data.
//
// * It is a transient view: The struct holds a mutable reference (&'a mut AppRuntime<R>). This means AppContext
// cannot outlive the original AppRuntime struct stored somewhere else on the stack or heap.
//
// * Exclusive Borrowing Lock: Because it borrows the runtime mutably (&mut), for the entire duration of 'a,
// no other part of your program can read or write to that AppRuntime.
//
// * Stack-Bound Usage: You cannot easily store this AppContext inside long-lived background threads or global variables.
// It is designed to be created on the stack (likely inside a frame or tick loop), used to perform some mutations
// (like load_problems), and then immediately dropped so the AppRuntime is unlocked.
//
// * Owned or Globally Valid: Any concrete type you substitute for R must either
// completely own its internal data (contain no references like &'b T) or only contain references that
// live for the entire duration of the program (like &'static str).
//
// * Safe for Spawning Threads: Because your Runtime trait includes a spawn method (fn spawn<F>(&self, future: F)
// where F: ... + 'static), the compiler must guarantee that the runtime itself won't disappear while a background
// thread or async task is running. Requiring R: 'static ensures that clones of your runtime can be
// sent across threads safely without causing "use-after-free" bugs.
//
// * Note on the contrast: While the wrapper AppContext is heavily constrained and short-lived due to 'a,
// the generic type R plugged into it must be infinitely long-lived ('static).
//
// ## Summary of the Architecture
//
// Your lifetime design reveals a classic "Tick Loop" or "Command" pattern:
//
//    1. You have a long-lived, multi-threaded core structure (Runtime and AppRuntime).
//    2. At specific moments (like an update loop or UI frame tick),
// 				you construct a temporary AppContext<'a> to gain exclusive, mutable access to the application state.
//    3. Once that quick operations phase ends, AppContext is dropped, releasing the borrow so the application can
// 				continue running its background async tasks.
//
/// ## [AppContext]
///
/// Exposes runtime capabilities
///
/// ## Methods
///
/// - [app](crate::runtime::app_runtime)
///
pub struct AppContext<'a, C, S>
where
	C: Ctx,
{
	pub context: &'a C,
	pub state: &'a mut S,
	pub event_rx: &'a mut C::EventReceiver,
	pub input: IOState,
	pub last_revision: u64,
}
// ## 2. What the 'static constraint tells you
// The + 'static on impl<'a, R: Runtime + 'static> AppContext<'a, R> tells
// us that the underlying runtime implementation (R) must be completely free of short-lived borrows.
impl<'a, C, S> AppContext<'a, C, S>
where
	C: Ctx,
{
	pub fn load_problems(&mut self) {
		tracing::info!("load_problems");
		// TODO: implement through Context/State
	}

	pub fn sample_problem(&mut self) {
		tracing::info!("sample_problem");
		// TODO: implement through Context/State
	}

	pub fn load_problem(&mut self) {
		tracing::info!("load_problem");
		// TODO: implement through Context/State
	}

	pub fn state_changed(&mut self) -> bool {
		// TODO: determine the revision from the new State abstraction.
		false
	}

	pub fn next_event(&mut self) -> Option<e::Event> {
		self.event_rx.try_recv()
	}
}
