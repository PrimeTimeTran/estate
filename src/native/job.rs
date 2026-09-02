// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use crate::{
	app::{state::EstateState, *},
	native::{agent::AgentContext, prelude::*},
	ui::TaskManagerScreen,
};

use notify::{Event, EventKind};

#[derive(Debug, Clone, Eq, Deserialize, PartialEq, Serialize)]
pub struct Task {
	pub id: TaskId,
	pub name: String,
	pub kind: TaskKind,
	pub status: TaskStatus,
}

#[derive(Debug)]
pub struct TaskManagerRuntime {
	watcher: notify::RecommendedWatcher,
	pub rx: tokio::sync::mpsc::Receiver<()>,
}
impl TaskManagerRuntime {
	pub fn new(path: &Path) -> Result<Self> {
		let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
		let mut watcher = RecommendedWatcher::new(
			move |res: Result<Event, notify::Error>| {
				if let Ok(event) = res {
					if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
						let _ = tx.blocking_send(());
					}
				}
			},
			Config::default(),
		)?;
		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		}

		Ok(Self { watcher, rx })
	}
}
#[derive(Debug, Default)]
pub struct TaskManagerState {
	pub dirty: bool,
	pub error: Option<String>,
	pub last_loaded: Option<SystemTime>,
	pub state: Option<EstateState>,
	pub state_path: PathBuf,
	pub tasks: HashMap<TaskId, Task>,
}

#[derive(Debug, Clone)]
pub struct AgentTask {
	pub id: String,
	pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
	pub artifacts: Vec<Artifact>,
	pub chat: Option<String>,
	pub logs: Vec<String>,
	pub spawned_tasks: Vec<AgentTask>,
	pub status: TaskStatus,
	pub summary: Option<String>,
	pub task_id: String,
}
pub struct TaskContext {
	pub task_id: String,
	pub artifacts: Vec<Artifact>,
	pub logs: Vec<String>,
	pub spawned_tasks: Vec<AgentTask>,
}
impl TaskResult {
	pub fn completed_chat(task_id: String, ctx: AgentContext, chat: String) -> Self {
		Self {
			artifacts: ctx.artifacts,
			chat: Some(chat),
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			status: TaskStatus::Completed,
			summary: None,
			task_id,
		}
	}
	pub fn completed_with_summary(task_id: String, ctx: AgentContext, summary: String) -> Self {
		Self {
			artifacts: ctx.artifacts,
			chat: None,
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			status: TaskStatus::Completed,
			summary: Some(summary),
			task_id,
		}
	}
	pub fn failed(
		task_id: String,
		ctx: AgentContext,
		reason: impl Into<String>,
		summary: Option<String>,
	) -> Self {
		Self {
			artifacts: ctx.artifacts,
			chat: None,
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			status: TaskStatus::Failed(reason.into()),
			summary,
			task_id,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Artifact {
	FileRead { path: String, content: String },
	FileWrite { path: String },
	Observation(String),
	ToolOutput(String),
}
