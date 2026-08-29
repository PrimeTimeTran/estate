// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use crate::{
	app::{EstateState, *},
	native::{agent::AgentContext, task_manager::WaterfallChart},
	prelude::*,
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
	rx: tokio::sync::mpsc::Receiver<()>,
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

#[derive(Debug)]
pub struct TaskManager {
	runtime: TaskManagerRuntime,
	pub state: TaskManagerState,
	pub waterfall: WaterfallChart,
}

impl TaskManager {
	pub fn new() -> Self {
		let path = PathBuf::from(STATE_PATH);
		Self::from_path(path).unwrap()
	}
	pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
		let state_path = path.into();
		let runtime = TaskManagerRuntime::new(&state_path)?;
		let mut state = TaskManagerState {
			state_path,
			..Default::default()
		};
		let mut manager = Self {
			state,
			runtime,
			waterfall: WaterfallChart::default(),
		};
		manager.reload();

		Ok(manager)
	}
	pub fn reload(&mut self) {
		match EstateState::load_from_path(&self.state.state_path) {
			Ok(state) => {
				self.state.state = Some(state);
				self.state.dirty = false;
				self.state.error = None;
				self.state.last_loaded = fs::metadata(&self.state.state_path)
					.and_then(|metadata| metadata.modified())
					.ok();
			}
			Err(error) => {
				self.state.error = Some(error.to_string());
				self.state.dirty = true;
			}
		}
	}
	pub fn poll_changes(&mut self) -> bool {
		#[cfg(not(target_arch = "wasm32"))]
		{
			if self.runtime.rx.try_recv().is_ok() {
				self.reload();
				return true;
			}
		}

		false
	}
}
impl TaskManager {
	pub fn create(&mut self, kind: TaskKind) -> TaskId {
		let id = Uuid::new_v4();
		let task = Task {
			id,
			name: kind.name(),
			kind,
			status: TaskStatus::Pending,
		};
		self.state.tasks.insert(id, task);
		self.state.dirty = true;
		id
	}
	pub fn get(&self, id: TaskId) -> Option<&Task> {
		self.state.tasks.get(&id)
	}
	fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
		self.state.tasks.get_mut(&id)
	}
	pub fn count(&self) -> usize {
		self.state.tasks.len()
	}
	pub fn list(&self) -> impl Iterator<Item = &Task> {
		self.state.tasks.values()
	}
	pub fn set_status(&mut self, id: TaskId, status: TaskStatus) -> Result<()> {
		let task = self
			.state
			.tasks
			.get_mut(&id)
			.ok_or_else(|| anyhow::anyhow!("task {id} not found"))?;

		task.status = status;

		Ok(())
	}
	pub fn save(&mut self) -> Result<()> {
		todo!("save")
	}
	pub fn clear(&mut self) -> bool {
		if self.state.tasks.is_empty() {
			return false;
		}

		self.state.tasks.clear();
		self.state.dirty = true;
		true
	}
	pub fn delete(&mut self, id: TaskId) -> Option<Task> {
		let task = self.state.tasks.remove(&id);

		if task.is_some() {
			self.state.dirty = true;
		}

		task
	}
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
	Pending,
	Running,
	Completed,
	Failed(String),
	Stopped,
	Interrupted,
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
