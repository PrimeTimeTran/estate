use crate::prelude::{daemon::daemon::*, *};
use std::sync::{Arc, RwLock};

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

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
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
	pub fn app(kind: EventKind) -> Self {
		Self::new(EventSource::App, kind)
	}
}

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventKind {
	DaemonStarted,
	DaemonStopped,
	StatusRequested,
	TaskRequested { task: Task },
	TaskCompleted { task: Task },
	CommandExecuted { command: String },
	FileCreated { path: String },
	FileModified { path: String },
	FileDeleted { path: String },
	EstateDiscovered { path: String },
	EstateRemoved { path: String },
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

#[derive(Clone, Debug)]
pub struct EstateRuntime {
	pub events: EventBus,
	pub state: Arc<RwLock<EstateState>>,
}

impl Default for EstateRuntime {
	fn default() -> Self {
		Self::new()
	}
}

impl EstateRuntime {
	pub fn new() -> Self {
		Self {
			events: EventBus::new(),
			state: Arc::new(RwLock::new(EstateState::load())),
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

pub struct TaskHandler;
#[async_trait::async_trait]
impl EventHandler for TaskHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		let EventKind::TaskRequested { task } = &event.kind else {
			return;
		};

		println!("🔥 TaskHandler received: {:?}", task);

		let task = task.clone();
		let runtime = runtime.clone();

		tokio::spawn(async move {
			println!("🔥 TaskRunner starting: {:?}", task);
			match TaskRunner::execute(task.clone()).await {
				Ok(()) => {
					println!("✅ TaskRunner completed: {:?}", task);
					runtime.emit(Event::daemon(EventKind::TaskCompleted { task }));
				}
				Err(error) => {
					eprintln!("❌ TaskRunner failed: {error:?}");
				}
			}
		});
	}
}

pub struct StateHandler;
#[async_trait::async_trait]
impl EventHandler for StateHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		println!("🔥 StateHandler received: {:?}", event.kind);

		let mut state = runtime.state.write().unwrap();

		match &event.kind {
			EventKind::DaemonStarted => {
				state.starts += 1;
				state.started_at = event.timestamp;
			}

			EventKind::StatusRequested => {
				state.status_checks += 1;
			}

			EventKind::IndexUpdated { files_changed } => {
				state.files_indexed += files_changed;
			}

			EventKind::TaskCompleted { .. } => {
				state.tasks_completed += 1;
			}

			EventKind::CommandExecuted { .. } => {
				state.tasks_created += 1;
			}

			_ => {}
		}

		EstateState::save(&state);
	}
}

pub struct CommandHandler;
#[async_trait::async_trait]
impl EventHandler for CommandHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		let EventKind::CommandExecuted { command } = &event.kind else {
			return;
		};

		println!("🔥 CommandHandler received: {command}");

		match command.as_str() {
			"dev_info" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					task: Task::BuildEstatePrototype,
				}));
			}

			"rebuild_index" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					task: Task::RebuildIndex,
				}));
			}

			"sync_bookmarks" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					task: Task::SyncBookmarks,
				}));
			}

			"generate_dashboard" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					task: Task::GenerateView("dashboard".into()),
				}));
			}

			_ => {
				println!("⚠️ unknown command: {command}");
			}
		}
	}
}

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum Task {
	RebuildIndex,
	GenerateView(String),
	SyncBookmarks,
	BuildEstatePrototype,
}

/// "Given this task, actually perform it."
pub struct TaskRunner;

impl TaskRunner {
	pub async fn execute(task: Task) -> anyhow::Result<()> {
		println!("🔥 TaskRunner execute: {:?}", task);

		match task {
			Task::RebuildIndex => {
				println!("🔨 rebuilding index");

				// TODO: perform index rebuild
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;

				println!("✅ index rebuild complete");
			}

			Task::GenerateView(name) => {
				println!("👁️ generating view: {name}");

				// TODO: generate the requested view
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;

				println!("✅ view generated: {name}");
			}

			Task::SyncBookmarks => {
				println!("🔖 syncing bookmarks");

				// TODO: synchronize bookmarks
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;

				println!("✅ bookmark sync complete");
			}

			Task::BuildEstatePrototype => {
				println!("🚧 starting BuildEstatePrototype");

				for i in 1..=10 {
					tokio::time::sleep(std::time::Duration::from_secs(1)).await;

					println!("🚧 prototype task: {i}/10");
				}

				println!("✅ BuildEstatePrototype complete");
			}
		}

		Ok(())
	}
}

fn now() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_secs()
}

static EVENT_ID: AtomicU64 = AtomicU64::new(1);
