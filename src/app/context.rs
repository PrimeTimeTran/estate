use crate::{
	Executor,
	api::Api,
	app::{state::*, *},
	e,
	prelude::*,
	r#trait::EventReceiver,
};

// The lifetimes 'a and 'static in your code tell a precise story about memory ownership, data borrows, and concurrency safety.
// Here is exactly what each lifetime communicates to the Rust compiler and to other developers:
//
// ## 1. What the 'a lifetime tells you
// The 'a lifetime on pub struct AppContext<'a, R: Runtime> tells us that AppContext is a short-lived, ephemeral helper object that does not own its data.

// * It is a transient view: The struct holds a mutable reference (&'a mut AppRuntime<R>). This means AppContext cannot outlive the original AppRuntime struct stored somewhere else on the stack or heap.
// * Exclusive Borrowing Lock: Because it borrows the runtime mutably (&mut), for the entire duration of 'a, no other part of your program can read or write to that AppRuntime.
// * Stack-Bound Usage: You cannot easily store this AppContext inside long-lived background threads or global variables. It is designed to be created on the stack (likely inside a frame or tick loop), used to perform some mutations (like load_problems), and then immediately dropped so the AppRuntime is unlocked.

// * Owned or Globally Valid: Any concrete type you substitute for R must either completely own its internal data (contain no references like &'b T) or only contain references that live for the entire duration of the program (like &'static str).
// * Safe for Spawning Threads: Because your Runtime trait includes a spawn method (fn spawn<F>(&self, future: F) where F: ... + 'static), the compiler must guarantee that the runtime itself won't disappear while a background thread or async task is running. Requiring R: 'static ensures that clones of your runtime can be sent across threads safely without causing "use-after-free" bugs.
// * Note on the contrast: While the wrapper AppContext is heavily constrained and short-lived due to 'a, the generic type R plugged into it must be infinitely long-lived ('static).

// ## Summary of the Architecture
// Your lifetime design reveals a classic "Tick Loop" or "Command" pattern:

//    1. You have a long-lived, multi-threaded core structure (Runtime and AppRuntime).
//    2. At specific moments (like an update loop or UI frame tick), you construct a temporary AppContext<'a> to gain exclusive, mutable access to the application state.
//    3. Once that quick operations phase ends, AppContext is dropped, releasing the borrow so the application can continue running its background async tasks.

// Would you like to look at how to optimize the trait names alongside these lifetimes, or do you want to verify if AppRuntime<R> can be safely shared across your background threads?
// impl<R: Runtime + 'static> AppRuntime<R> {
pub struct AppContext<'a, R: Runtime, E> {
	pub app: &'a mut AppRuntime<R, E>,
	pub last_revision: u64,
	pub event_rx: R::EventReceiver,
	pub input: IOState,
	// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
}

// ## 2. What the 'static constraint tells you
// The + 'static on impl<'a, R: Runtime + 'static> AppContext<'a, R> tells
// us that the underlying runtime implementation (R) must be completely free of short-lived borrows.
// impl<'a, R: Runtime + 'static, E> AppContext<'a, R, E> {
// 	pub fn load_problems(&mut self) {
// 		self.app.load_problems();
// 	}
// }

impl<'a, R: Runtime, E> AppContext<'a, R, E> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.app.state()
	}

	pub fn state_changed(&mut self) -> bool {
		let revision = self.app.runtime().state().revision();
		if revision != self.last_revision {
			self.last_revision = revision;
			true
		} else {
			false
		}
	}
	// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub fn next_event(&mut self) -> Option<e::Event> {
		self.event_rx.try_recv()
	}
}
