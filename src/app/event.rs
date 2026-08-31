pub use crate::{app::*, prelude::*};

// must leave pub prefix or it breaks other places
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub(crate) struct Event {
	pub id: u64,
	pub kind: EventKind,
	pub source: EventSource,
	pub timestamp: u64,
}
impl Event {
	pub fn new(source: EventSource, kind: EventKind) -> Self {
		tracing::debug!("new Event {:?}", source);
		// let trace = Tracer::new("event");
		// let mut flow = trace.flow("new");
		// let flow = flow.info("Event::new").unwrap();
		Self {
			id: crate::data::EVENT_ID.fetch_add(1, Ordering::Relaxed),
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
pub enum EventKind {
	SessionStart,
	SessionStop { session: Session },
	WorkspaceIndexed { duration: u64 },
	Navigate(ViewType),
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
