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
		let (tx, _) = broadcast::channel(256);
		tracing::info!("EventBus CREATED");

		Self { tx }
	}

	pub fn emit(&self, event: e::Event) {
		match self.tx.send(event.clone()) {
			Ok(count) => {
				tracing::info!("📡 Event emitted: {:?} → {} receiver(s)", event.kind, count);
			}
			Err(_) => {
				tracing::info!("⚠️ Event emitted with NO receivers: {:?}", event.kind);
			}
		}
	}

	/// Raw Tokio broadcast receiver.
	pub fn subscribe(&self) -> broadcast::Receiver<e::Event> {
		tracing::info!("EventBus subscribe RAW");
		self.tx.subscribe()
	}

	/// Application-level event receiver.
	pub fn subscribe_broadcast(&self, owner: &'static str) -> BroadcastReceiver<e::Event> {
		let id = NEXT_RECEIVER_ID.fetch_add(1, Ordering::Relaxed);

		tracing::info!(id, owner, "EventBus creating broadcast receiver");

		BroadcastReceiver {
			id,
			owner,
			rx: self.tx.subscribe(),
		}
	}

	/// Application-level event sender.
	pub fn sender(&self) -> BroadcastSender<e::Event> {
		BroadcastSender {
			tx: self.tx.clone(),
		}
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

impl std::hash::Hash for EventBus {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.tx.same_channel(&self.tx).hash(state);
	}
}

/// ## [EventBus]
///
/// Enables disparate modules to talk to each other by sending and receiving
/// messages called events.
///
/// <details>
/// <summary>Diagram</summary>
///
/// ```mermaid
/// flowchart TD
///     EB["EventBus"]
///     EB --> RX["subscribe() → BroadcastReceiver"]
///     EB --> TX["sender() → BroadcastSender"]
///
///     RX --> R["Renderer"]
///     TX --> R
///
///     R --> AC["AppContext"]
///     AC --> PV["ProblemView::draw()"]
///
///     PV -->|send| PR["ProblemsRequested"]
///     PR --> EB
///
///     EB --> RT["AppRuntime"]
///     RT --> API["async API request"]
///
///     API -->|success| PL["ProblemsLoaded"]
///     API -->|failure| PF["ProblemsLoadFailed"]
///
///     PL --> EB
///     PF --> EB
///
///     RT --> S["Update application state"]
/// ```
/// </details>
///
/// The issue is it's not big enough. Doesn't scroll?
#[derive(Debug, Clone)]
pub struct EventBus {
	tx: broadcast::Sender<e::Event>,
	// id: usize,
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
