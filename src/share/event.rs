use crate::prelude::*;

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
	TaskRequested {
		request: TaskRequest,
	},
	TaskCreated {
		task_id: TaskId,
		name: String,
	},
	TaskStarted {
		task_id: TaskId,
	},
	TaskCompleted {
		task_id: TaskId,
	},
	TaskFailed {
		task_id: TaskId,
		error: String,
	},
	TaskStopped {
		task_id: TaskId,
	},
	TaskDeleted {
		task_id: TaskId,
	},
	TasksCleared,
	// ─────────────────────────────────────────────
	// Commands
	// ─────────────────────────────────────────────
	CommandExecuted {
		command: String,
	},
	// ─────────────────────────────────────────────
	// Files
	// ─────────────────────────────────────────────
	FileCreated {
		inode: Inode,
		path: String,
	},
	FileModified {
		inode: Inode,
		path: String,
	},
	FileDeleted {
		inode: Inode,
		path: String,
	},
	// ─────────────────────────────────────────────
	// Estate / indexing
	// ─────────────────────────────────────────────
	EstateDiscovered {
		inode: Inode,
		path: String,
	},
	EstateRemoved {
		inode: Inode,
		path: String,
	},
	IndexUpdated {
		files_changed: u64,
	},
	CacheInvalidated {
		reason: String,
	},
}
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Inode;

pub type TaskId = Uuid;

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskRequest {
	Create(TaskKind),
	Run(TaskId),
	Stop(TaskId),
	Delete(TaskId),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Hash, Serialize)]
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
			TaskKind::GenerateView(name) => { format!("Generate View: {name}") }
			TaskKind::SyncBookmarks => "Sync Bookmarks".into(),
			TaskKind::BuildEstatePrototype => "Build Estate Prototype".into(),
		}
	}
}
