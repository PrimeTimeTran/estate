use crate::{
	app::{Runtime, task, *},
	handler::EventHandler,
	native::{
		daemon::{DocCompiler, NativeRuntime},
		session::Session,
	},
	prelude::*,
};

use cli::prelude::Context as CliContext;
use tokio_util::sync::CancellationToken;

pub(crate) mod handler;

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub struct Event {
	pub id: u64,
	pub kind: EventKind,
	pub source: EventSource,
	pub timestamp: u64,
	// pub flow: Tracer,
}
impl Event {
	pub fn new(source: EventSource, kind: EventKind) -> Self {
		tracing::debug!("new Event {:?}", source);
		// let trace = Tracer::new("event");
		// let mut flow = trace.flow("new");
		// let flow = flow.info("Event::new").unwrap();
		Self {
			id: EVENT_ID.fetch_add(1, Ordering::Relaxed),
			kind,
			source,
			timestamp: crate::util::now(),
		}
	}
	pub fn daemon(kind: EventKind) -> Self {
		Self::new(EventSource::Daemon, kind)
	}
	pub fn cli(kind: EventKind) -> Self {
		Self::new(EventSource::Cli, kind)
	}
	pub fn filesystem(kind: EventKind) -> Self {
		Self::new(EventSource::Filesystem, kind)
	}
	pub fn editor(kind: EventKind) -> Self {
		Self::new(EventSource::Editor, kind)
	}
	pub fn app(kind: EventKind) -> Self {
		Self::new(EventSource::App, kind)
	}
}
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub(crate) enum EventKind {
	SessionStart,
	SessionStop { session: Session },
	WorkspaceIndexed { duration: u64 },
	DaemonStarted,
	DaemonStopped,
	StatusRequested,
	TaskRequested { request: TaskRequest },
	TaskCreated { task_id: Uuid, kind: TaskKind },
	TaskStarted { task_id: TaskId },
	TaskCompleted { task_id: TaskId },
	TaskFailed { task_id: TaskId, error: String },
	TaskStopped { task_id: TaskId },
	TaskDeleted { task_id: TaskId },
	TasksCleared,
	CommandExecuted { command: String },
	FileCreated { inode: Inode, path: String },
	FileModified { inode: Inode, path: String },
	FileDeleted { inode: Inode, path: String },
	EstateDiscovered { inode: Inode, path: String },
	EstateRemoved { inode: Inode, path: String },
	IndexUpdated { files_changed: u64 },
	CacheInvalidated { reason: String },
}
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
}
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
