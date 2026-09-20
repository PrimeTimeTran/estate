use crate::prelude::*;

impl<R: Runtime> Default for EventDispatcher<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Runtime> EventDispatcher<R> {
	/// ## [EventDispatcher::dispatch]
	///
	/// Dispatch events to all registered handlers.
	/// The handlers decide for themselves if they're interested in processing it.
	///
	/// This way the handlers can react/filter events which aren't relevant to their domain.
	///
	pub async fn dispatch(&self, event: e::Event, runtime: &R) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
		runtime.event_processed();
	}

	/// ## [EventDispatcher::register]
	///
	/// Initialize handlers which implement business logic for events.
	///
	pub fn register<H>(&mut self, handler: H)
	where
		H: traits::EventHandler<R> + 'static,
	{
		self.handlers.push(Box::new(handler));
	}

	/// ## [EventDispatcher::run]
	///
	/// One to many distribution of events from a single event to multiple handlers
	///
	pub async fn run(self, mut rx: tokio::sync::broadcast::Receiver<e::Event>, runtime: R) {
		while let Ok(event) = rx.recv().await {
			self.dispatch(event, &runtime).await;
		}
	}

	/// ## [EventDispatcher::new]
	///
	/// Instantiate a new dispatcher instance
	///
	/// ```ignore
	/// One event → Many handlers
	///
	///                  ┌──> Handler A
	///                  │
	///           Event ─┼──> Handler B
	///                  │
	///                  └──> Handler C
	/// ```
	pub fn new() -> Self {
		Self {
			handlers: Vec::new(),
		}
	}
}

/// ## [EventDispatcher]
///
/// Component that manages and triggers notifications—called
/// events—when specific actions or state changes occur in an application.
///
/// ### Methods
///
/// - [new][EventDispatcher::new]: Initialize a new dispatcher (used to bind a dispatcher to a specific domain).
/// - [register][EventDispatcher::register]: Binds a handler to a dispatcher
/// - [dispatch][EventDispatcher::dispatch]
/// - [run][EventDispatcher::run]
///
pub struct EventDispatcher<R: Runtime> {
	handlers: Vec<Box<dyn traits::EventHandler<R>>>,
}
