use crate::prelude::{daemon::daemon::*, *};
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
		tracing::info!("new Event {:?}", source);
		Self {
			id: EVENT_ID.fetch_add(1, Ordering::Relaxed),
			kind,
			source,
			timestamp: now(),
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
	// ─────────────────────────────────────────────
	// Daemon
	// ─────────────────────────────────────────────
	DaemonStarted,
	DaemonStopped,
	StatusRequested,
	// ─────────────────────────────────────────────
	// Tasks
	// ─────────────────────────────────────────────
	TaskRequested { request: TaskRequest },
	TaskCreated { task_id: TaskId, name: String },
	TaskStarted { task_id: TaskId },
	TaskCompleted { task_id: TaskId },
	TaskFailed { task_id: TaskId, error: String },
	TaskStopped { task_id: TaskId },
	TaskDeleted { task_id: TaskId },
	TasksCleared,
	// ─────────────────────────────────────────────
	// Commands
	// ─────────────────────────────────────────────
	CommandExecuted { command: String },
	// ─────────────────────────────────────────────
	// Files
	// ─────────────────────────────────────────────
	FileCreated { inode: Inode, path: String },
	FileModified { inode: Inode, path: String },
	FileDeleted { inode: Inode, path: String },
	// ─────────────────────────────────────────────
	// Estate / indexing
	// ─────────────────────────────────────────────
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
#[derive(Clone, Debug)]
pub struct EstateRuntime {
	pub events: EventBus,
	pub state: Arc<RwLock<EstateState>>,
	pub tasks: Arc<RwLock<TaskManager>>,
}
impl Default for EstateRuntime {
	fn default() -> Self {
		Self::new()
	}
}
impl EstateRuntime {
	pub fn new() -> Self {
		tracing::info!("EstateRuntime new");
		Self {
			events: EventBus::new(),
			state: Arc::new(RwLock::new(EstateState::load())),
			tasks: Arc::new(RwLock::new(TaskManager::new())),
		}
	}
	pub fn emit(&self, event: Event) {
		self.events.emit(event);
	}
	pub fn event_processed(&self) {
		let mut state = self.state.write().unwrap();
		state.events_processed += 1;
	}
	// pub fn increment_events_processed() {}
	pub fn start_dispatcher(self: &Arc<Self>) {
		// println!("🔥 start_dispatcher()");
		let runtime = Arc::clone(self);
		// println!("🔥 subscribing to event bus");
		let mut receiver = runtime.events.subscribe();
		// println!("🔥 event bus subscribed");
		let mut dispatcher = EventDispatcher::new();
		dispatcher.register(TaskHandler);
		dispatcher.register(StateHandler);
		dispatcher.register(CommandHandler);
		dispatcher.register(FileWatcherHandler);
		// println!("🔥 handlers registered");
		tokio::spawn(async move {
			// println!("🔥 EVENT DISPATCHER TASK STARTED");
			loop {
				match receiver.recv().await {
					Ok(event) => {
						tracing::info!("🔥 DISPATCHER RECEIVED: {:?}", event.kind);
						dispatcher.dispatch(event, &runtime).await;
					}
					Err(broadcast::error::RecvError::Lagged(count)) => {
						tracing::warn!(count, "event dispatcher lagged");
					}
					Err(broadcast::error::RecvError::Closed) => {
						println!("🔥 EVENT BUS CLOSED");
						break;
					}
				}
			}
		});
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
		match self.sender.send(event.clone()) {
			Ok(count) => {
				tracing::info!("📡 Event emitted: {:?} → {} receiver(s)", event.kind, count);
			}
			Err(_) => {
				tracing::error!("⚠️ Event emitted with NO receivers: {:?}", event.kind);
			}
		}
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
		tracing::info!("📡 received {:?}", event);
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
	pub async fn run(self, mut rx: broadcast::Receiver<Event>, runtime: EstateRuntime) {
		while let Ok(event) = rx.recv().await {
			self.dispatch(event, &runtime).await;
		}
	}
	pub async fn dispatch(&self, event: Event, runtime: &EstateRuntime) {
		for handler in &self.handlers {
			handler.handle(&event, runtime).await;
		}
		runtime.event_processed();
	}
}
pub struct FileWatcherHandler;
#[async_trait::async_trait]
impl EventHandler for FileWatcherHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		if let EventKind::FileModified { inode, path } = &event.kind {
			tracing::info!("📡 FileWatcherHandler handle {:?} ({:?})", event, inode);
			runtime.emit(Event::daemon(EventKind::IndexUpdated { files_changed: 1 }));
		}
	}
}
pub struct TaskHandler;
#[async_trait::async_trait]
impl EventHandler for TaskHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		tracing::info!("📡 EventHandler.handle {:?}", event);
		let EventKind::TaskRequested { request } = &event.kind else {
			return;
		};
		let task_id = match request {
			TaskRequest::Create(kind) => {
				tracing::info!("TaskRequest::Create {:?}", kind);
				let mut tasks = runtime.tasks.write().unwrap();
				let task = tasks.create(kind.clone());
				runtime.emit(Event::daemon(EventKind::TaskCreated {
					task_id: task,
					name: kind.name(),
				}));
				task
			}
			TaskRequest::Run(task_id) => *task_id,
			_ => return,
		};
		let task = {
			let tasks = runtime.tasks.read().unwrap();
			let Some(task) = tasks.get(task_id).cloned() else {
				tracing::warn!(%task_id, "requested task not found");
				return;
			};
			task
		};
		{
			let mut tasks = runtime.tasks.write().unwrap();
			if let Err(error) = tasks.set_status(task_id, TaskStatus::Running) {
				tracing::warn!(%task_id, %error, "failed to set task running");
				return;
			}
		}
		runtime.emit(Event::daemon(EventKind::TaskStarted { task_id }));
		let runtime = runtime.clone();
		tokio::spawn(async move {
			tracing::info!(
				%task_id,
				task = %task.name,
				"task starting"
			);
			match TaskRunner::execute(task.clone()).await {
				Ok(()) => {
					tracing::info!("TaskHandler match TaskRunner::execute {:?}", task);
					runtime.emit(Event::daemon(EventKind::TaskCompleted { task_id }));
				}
				Err(error) => {
					runtime.emit(Event::daemon(EventKind::TaskFailed {
						task_id,
						error: error.to_string(),
					}));
				}
			}
		});
	}
}
pub struct StateHandler;
#[async_trait::async_trait]
impl EventHandler for StateHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		tracing::info!("🔥 StateHandler received: {:?}", event.kind);
		let mut state = runtime.state.write().unwrap();
		match &event.kind {
			EventKind::DaemonStarted => {
				state.starts += 1;
				state.status_checks += 1;
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
			EventKind::TaskCreated { .. } => {
				state.tasks_created += 1;
			}
			EventKind::DaemonStopped => {
				let run_duration = event.timestamp.saturating_sub(state.started_at);
				state.longest_run = state.longest_run.max(run_duration);
			}
			_ => {}
		}
		state.events_processed += 1;
		EstateState::save(&state);
	}
}
pub struct CommandHandler;
#[async_trait::async_trait]
impl EventHandler for CommandHandler {
	async fn handle(&self, event: &Event, runtime: &EstateRuntime) {
		tracing::info!("CommandHandler handler {:?}", event);
		let EventKind::CommandExecuted { command } = &event.kind else {
			tracing::info!("CommandHandler handle {:?}", event);
			return;
		};
		match command.as_str() {
			"TaskRequest::Createtask_create" => {
				tracing::info!("CommandHandler task_create {:?}", event);
				runtime.emit(Event::app(EventKind::TaskRequested {
					request: TaskRequest::Create(TaskKind::SyncBookmarks),
				}));
			}
			"task_list" => {
				let tasks = runtime.tasks.read().unwrap();
				println!("════════════════════════════════════");
				println!("             ESTATE TASKS");
				println!("════════════════════════════════════");
				if tasks.count() == 0 {
					println!("No tasks in memory.");
				} else {
					for task in tasks.list() {
						println!("[{}] {} — {:?}", task.id, task.name, task.status);
					}
				}
				drop(tasks);
				let state = EstateState::load();
				println!();
				println!("──────────── persisted state ────────────");
				println!("starts:           {}", state.starts);
				println!("status checks:    {}", state.status_checks);
				println!("tasks completed:  {}", state.tasks_completed);
				println!("files indexed:    {}", state.files_indexed);
				println!("events processed: {}", state.events_processed);
				println!("longest run:      {}", state.longest_run);
				println!("started at:       {}", state.started_at);
				runtime.emit(Event::daemon(EventKind::StatusRequested));
			}
			"task_clear" => {
				{
					let mut tasks = runtime.tasks.write().unwrap();
					tasks.clear();
				}
				runtime.emit(Event::daemon(EventKind::TasksCleared));
			}
			"dev_info" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					request: TaskRequest::Create(TaskKind::BuildEstatePrototype),
				}));
			}
			"rebuild_index" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					request: TaskRequest::Create(TaskKind::RebuildIndex),
				}));
			}
			"sync_bookmarks" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					request: TaskRequest::Create(TaskKind::SyncBookmarks),
				}));
			}
			"generate_dashboard" => {
				runtime.emit(Event::daemon(EventKind::TaskRequested {
					request: TaskRequest::Create(TaskKind::GenerateView("dashboard".into())),
				}));
			}
			_ => {
				println!("⚠️ unknown command: {command}");
			}
		}
	}
}
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskKind {
	BuildEstatePrototype,
	GenerateView(String),
	RebuildIndex,
	SyncBookmarks,
}
impl TaskKind {
	pub fn name(&self) -> String {
		match self {
			TaskKind::RebuildIndex => "Rebuild Index".into(),
			TaskKind::GenerateView(name) => {
				format!("Generate View: {name}")
			}
			TaskKind::SyncBookmarks => "Sync Bookmarks".into(),
			TaskKind::BuildEstatePrototype => "Build Estate Prototype".into(),
		}
	}
}
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskRequest {
	Create(TaskKind),
	Run(TaskId),
	Stop(TaskId),
	Delete(TaskId),
}
/// "Given this task, actually perform it."
pub struct TaskRunner;
impl TaskRunner {
	pub async fn execute(task: Task) -> anyhow::Result<()> {
		tracing::info!("TaskRunner execute {:?}", task);
		match task.kind {
			TaskKind::RebuildIndex => {
				println!("🔨 rebuilding index");
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				println!("✅ index rebuild complete");
			}
			TaskKind::GenerateView(name) => {
				println!("👁️ generating view: {name}");
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				println!("✅ view generated: {name}");
			}
			TaskKind::SyncBookmarks => {
				tracing::info!("🔖 TaskKind::SyncBookmarks {:?}", task);
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				tracing::info!("✅ bookmark sync complete {:?}", task.id);
			}
			TaskKind::BuildEstatePrototype => {
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
