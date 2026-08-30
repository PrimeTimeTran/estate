use crate::{
	app::{Runtime, task, *},
	// lib_native::handler::EventHandler,
	native::daemon::DocCompiler,
	native::{NativeRuntime, *},
	// runtime::NativeRuntime,
	prelude::{event::*, *},
};
use cli::prelude::Context as CliContext;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub(crate) mod channel;
pub(crate) mod handler;
pub(crate) use handler::{EventHandler, *};

#[derive(Debug, Clone)]
pub struct EventBus {
	sender: broadcast::Sender<Event>,
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
	pub fn emit(&self, event: Event) {
		match self.sender.send(event.clone()) {
			Ok(count) => {
				tracing::debug!("📡 Event emitted: {:?} → {} receiver(s)", event.kind, count);
			}
			Err(_) => {
				tracing::debug!("⚠️ Event emitted with NO receivers: {:?}", event.kind);
			}
		}
	}
	pub fn subscribe(&self) -> broadcast::Receiver<Event> {
		self.sender.subscribe()
	}
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
	pub async fn run(self, mut rx: broadcast::Receiver<Event>, runtime: NativeRuntime) {
		while let Ok(event) = rx.recv().await {
			self.dispatch(event, &runtime).await;
		}
	}
	pub async fn dispatch(&self, event: Event, runtime: &NativeRuntime) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
		runtime.event_processed();
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
}
