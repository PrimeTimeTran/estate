use crate::prelude::*;

impl Default for EventBus {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Runtime> Default for EventDispatcher<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl EventBus {
	pub fn new() -> Self {
		let (sender, _) = broadcast::channel(256);
		Self { sender }
	}
	pub fn emit(&self, event: e::Event) {
		match self.sender.send(event.clone()) {
			Ok(count) => {
				tracing::debug!("📡 Event emitted: {:?} → {} receiver(s)", event.kind, count);
			}
			Err(_) => {
				tracing::debug!("⚠️ Event emitted with NO receivers: {:?}", event.kind);
			}
		}
	}
	pub fn subscribe(&self) -> broadcast::Receiver<e::Event> {
		self.sender.subscribe()
	}
}

/// ## [EventDispatcher]
///
/// Dispatcher
///
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
	/// Register a handler which listens for events which handles business logic
	/// work for that specific event.
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
impl std::hash::Hash for EventBus {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.sender.same_channel(&self.sender).hash(state);
	}
}

/// [EventBus]
///
/// Handles Sending
///
#[derive(Debug, Clone)]
pub struct EventBus {
	sender: broadcast::Sender<e::Event>,
}

/// ## [EventDispatcher]
///
/// Works with an event bus to handle background job system.
///
/// ### Methods
///
/// - [new][EventDispatcher::new]
/// - [register][EventDispatcher::register]
/// - [dispatch][EventDispatcher::dispatch]
/// - [run][EventDispatcher::run]
///
pub struct EventDispatcher<R: Runtime> {
	handlers: Vec<Box<dyn traits::EventHandler<R>>>,
}
