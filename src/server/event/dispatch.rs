use crate::{e, native::prelude::*, ui};

pub(crate) mod channel;
pub(crate) mod handler;

pub struct EventDispatcher<R: Runtime> {
	handlers: Vec<Box<dyn EventHandler<R>>>,
}
impl<R: Runtime> Default for EventDispatcher<R> {
	fn default() -> Self {
		Self::new()
	}
}

impl<R: Runtime> EventDispatcher<R> {
	pub fn new() -> Self {
		// # Fan Out
		// One event → Many handlers
		//                  ┌──> Handler A
		//                  │
		// Event           ─┼──> Handler B
		//                  │
		//                  └──> Handler C
		Self {
			handlers: Vec::new(),
		}
	}
	pub async fn run(self, mut rx: tokio::sync::broadcast::Receiver<e::Event>, runtime: R) {
		while let Ok(event) = rx.recv().await {
			self.dispatch(event, &runtime).await;
		}
	}
	pub async fn dispatch(&self, event: e::Event, runtime: &R) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
		runtime.event_processed();
	}
	pub fn register<H>(&mut self, handler: H)
	where
		H: EventHandler<R> + 'static,
	{
		self.handlers.push(Box::new(handler));
	}
}

#[derive(Debug, Clone)]
pub struct EventBus {
	sender: broadcast::Sender<e::Event>,
}
impl std::hash::Hash for EventBus {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.sender.same_channel(&self.sender).hash(state);
	}
}
impl Default for EventBus {
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
