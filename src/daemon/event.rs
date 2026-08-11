use crate::daemon::daemon::*;
use crate::prelude::*;

// Events = facts that happened
// Handlers = reactions to facts
// Tasks = units of work
// Commands = requests to do something
/*
		Tokio provides multiple channel types because they solve different
		communication problems.

		------------------------------------------------------------
		mpsc = Multi Producer Single Consumer
		------------------------------------------------------------

		Many things send work to ONE worker.

		Example:
				CLI
					\
				Editor ---> Task Queue ---> Index Worker
					/
				Watcher

		Use when:
				- jobs should be processed once
				- order matters
				- you have a worker queue

		Example:
				"Please rebuild this index"


		------------------------------------------------------------
		broadcast = Multi Producer Multi Consumer
		------------------------------------------------------------

		Many things send events.
		Many listeners receive the same event.

		Example:

				File Watcher
							|
							v
					EventBus
					/   |    \
				 /    |     \
			Logger Index UI

		Use when:
				- multiple systems need to react
				- event is a fact that happened

		Example:
				"README.md changed"


		------------------------------------------------------------
		watch = Single Value State Updates
		------------------------------------------------------------

		One value changes.
		Listeners only care about the latest value.

		Example:

				Daemon Status
							|
							v
					watch channel
							|
					UI / menu bar

		Old values do not matter.

		Example:
				"Daemon is currently healthy"


		Estate architecture:

				Commands
						|
						v
					mpsc
				(do work)

				Events
						|
						v
				broadcast
				(announce facts)

				State
						|
						v
					watch
				(latest snapshot)
*/

pub async fn event_loop(mut rx: broadcast::Receiver<Event>, runtime: EstateRuntime) {
	let mut dispatcher = EventDispatcher::new();
	dispatcher.register(LogHandler);
	dispatcher.register(FileWatcherHandler);
	while let Ok(event) = rx.recv().await {
		dispatcher.dispatch(event, &runtime).await;
	}
}
static EVENT_ID: AtomicU64 = AtomicU64::new(1);

fn now() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_secs()
}

///--------------------------------------------------------------------------------
///#      Hi
///--------------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
	pub id: u64,
	pub timestamp: u64,
	pub source: EventSource,
	pub kind: EventKind,
}

impl Event {
	pub fn new(source: EventSource, kind: EventKind) -> Self {
		Self {
			id: EVENT_ID.fetch_add(1, Ordering::Relaxed),
			timestamp: now(),
			source,
			kind,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
	DaemonStarted,
	StatusRequested,
	CommandExecuted { command: String },
	FileCreated { path: String },
	FileModified { path: String },
	FileDeleted { path: String },
	EstateDiscovered { path: String },
	EstateRemoved { path: String },
	IndexUpdated { files_changed: u64 },
	CacheInvalidated { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource {
	Daemon,
	Cli,
	Filesystem,
	Editor,
}

#[derive(Debug, Clone)]
pub struct EstateRuntime {
	pub events: EventBus,
	pub state: daemon::DaemonState,
}
impl Default for EstateRuntime {
	fn default() -> Self {
		Self::new()
	}
}

impl EstateRuntime {
	pub fn new() -> Self {
		let mut state = DaemonState::load();
		state.starts += 1;
		state.started_at = DaemonState::now();
		DaemonState::save(&state);
		Self {
			events: EventBus::new(),
			state,
		}
	}

	pub fn emit(&self, event: Event) {
		self.events.emit(event);
	}
}

#[derive(Debug, Clone)]
pub struct EventBus {
	sender: broadcast::Sender<Event>,
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
		// Ignore error if there are no listeners.
		let _ = self.sender.send(event);
	}
	pub fn subscribe(&self) -> broadcast::Receiver<Event> {
		self.sender.subscribe()
	}
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime);
}

pub struct LogHandler;

#[async_trait::async_trait]
impl EventHandler for LogHandler {
	async fn handle(&self, event: &Event, _runtime: &EstateRuntime) {
		println!("📡 received {:?}", event);
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
	pub async fn dispatch(&self, event: Event, runtime: &EstateRuntime) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
	}
}

pub struct FileWatcherHandler;

#[async_trait::async_trait]
impl EventHandler for FileWatcherHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		if let EventKind::FileModified { path } = &event.kind {
			println!("reindexing {}", path);
			runtime.emit(Event::daemon(EventKind::IndexUpdated { files_changed: 1 }));
		}
	}
}
#[derive(Debug)]
pub enum Task {
	RebuildIndex,
	GenerateView(String),
	SyncBookmarks,
}

pub struct TaskRunner;

impl TaskRunner {
	pub async fn execute(task: Task) {
		match task {
			Task::RebuildIndex => {
				println!("building index");
			}

			Task::GenerateView(name) => {
				println!("generating {}", name);
			}

			Task::SyncBookmarks => {
				println!("syncing bookmarks");
			}
		}
	}
}
