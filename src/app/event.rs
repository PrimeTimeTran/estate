/// Event System
///
pub use crate::prelude::*;

/// Event Structure
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub struct Event {
	pub id: u64,
	pub kind: EventKind,
	pub source: EventSource,
	pub timestamp: u64,
}
impl Event {
	fn new(source: EventSource, kind: EventKind) -> Self {
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

/// # Create Events
///
pub mod create {
	use crate::app::event::*;
	pub fn daemon(kind: EventKind) -> Event {
		Event::daemon(kind)
	}
	pub fn cli(kind: EventKind) -> Event {
		Event::cli(kind)
	}
	pub fn fs(kind: EventKind) -> Event {
		Event::filesystem(kind)
	}
	pub fn editor(kind: EventKind) -> Event {
		Event::editor(kind)
	}
	/// # Create App Type Events
	///
	/// Events related to application wide state
	pub fn app(kind: EventKind) -> Event {
		Event::app(kind)
	}
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]

pub enum EventKind {
	ApiError(String),
	CacheInvalidated { reason: String },
	CommandExecuted { command: String },
	DaemonStarted,
	DaemonStopped,
	EstateDiscovered { inode: Inode, path: String },
	EstateRemoved { inode: Inode, path: String },
	FileCreated { inode: Inode, path: String },
	FileDeleted { inode: Inode, path: String },
	FileModified { inode: Inode, path: String },
	IndexUpdated { files_changed: u64 },
	Navigate(ViewType),
	ProblemLoaded(StoredProblem),
	ProblemLoadFailed(String),
	ProblemSampled(StoredProblem),
	ProblemSampleFailed(String),
	ProblemsLoaded(Vec<StoredProblem>),
	ProblemsLoadFailed(String),
	SampleProblemsError(String),
	SampleProblemsLoaded(Vec<StoredProblem>),
	SampleProblemsLoading,
	SessionStart,
	SessionStop { session: Session },
	StatusRequested,
	TaskCompleted { task_id: TaskId },
	TaskCreated { task_id: Uuid, kind: TaskKind },
	TaskDeleted { task_id: TaskId },
	TaskFailed { task_id: TaskId, error: String },
	TaskRequested { request: TaskRequest },
	TasksCleared,
	TaskStarted { task_id: TaskId },
	TaskStopped { task_id: TaskId },
	WorkspaceIndexed { duration: u64 },
}

/// # [Klass] (Alias of EventKind)
///
/// Represents full event lifecycle for representing initial, pending,
/// failed, repeated when necessary.
pub type Klass = EventKind;

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
}

// use crate::model::problem::StoredProblem;
// use crate::proto::types::Problem as ProtoProblem;

pub type Problem = ProtoProblem;

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct ProblemLoaded {
	pub id: String,
	pub title: String,
	pub slug: String,
}
impl From<ProtoProblem> for ProblemLoaded {
	fn from(problem: ProtoProblem) -> Self {
		Self {
			id: problem.id,
			title: problem.title,
			slug: problem.slug,
		}
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
	Navigate(crate::ui::ViewType),
	RuntimeEvent,
}
