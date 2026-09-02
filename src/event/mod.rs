use crate::{e, native::prelude::*, ui};

pub(crate) mod channel;
pub(crate) mod handler;
pub(crate) use handler::EventHandler;

pub trait EventReceiver {
	fn try_recv(&mut self) -> Option<e::Event>;
}
pub struct EventDispatcher {
	handlers: Vec<Box<dyn EventHandler>>,
}
impl Default for EventDispatcher {
	fn default() -> Self {
		Self::new()
	}
}
impl EventDispatcher {
	pub fn new() -> Self {
		Self {
			handlers: Vec::new(),
		}
	}
	pub fn register<H>(&mut self, handler: H)
	where
		H: EventHandler + 'static,
	{
		self.handlers.push(Box::new(handler));
	}
	pub async fn run(
		self,
		mut rx: tokio::sync::broadcast::Receiver<e::Event>,
		runtime: NativeRuntime,
	) {
		while let Ok(event) = rx.recv().await {
			self.dispatch(event, &runtime).await;
		}
	}
	pub async fn dispatch(&self, event: e::Event, runtime: &NativeRuntime) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
		runtime.event_processed();
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

#[derive(Debug)]
pub enum AppEvent {
	Shutdown,
	ModifiersChanged {
		alt: bool,
		command: bool,
		ctrl: bool,
		shift: bool,
	},
	CursorPosition {
		x: f64,
		y: f64,
	},
	TickClock(String),
	AppEvent,
	Navigate(ui::ViewType),
	RuntimeEvent,
}
