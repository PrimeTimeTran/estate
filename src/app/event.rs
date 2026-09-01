pub use crate::{app::*, prelude::*};

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
	ProblemsLoaded(Vec<StoredProblem>),
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
	ProblemsLoadFailed(String),
	ProblemLoaded(StoredProblem),
	ProblemSampled(StoredProblem),
	ProblemLoadFailed(String),
	ProblemSampleFailed(String),
	SampleProblemsLoading,
	SampleProblemsLoaded(Vec<StoredProblem>),
	SampleProblemsError(String),
}
pub type Klass = EventKind;
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
}

use crate::model::problem::StoredProblem;
use crate::proto::leetcode::types::Problem as ProtoProblem;

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
