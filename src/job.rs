// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use crate::{agent::AgentContext, prelude::*, router, ve};

pub type TaskId = Uuid;
#[derive(Debug, Clone)]
pub struct Task {
	pub id: TaskId,
	pub name: String,
	pub kind: TaskKind,
	pub status: TaskStatus,
}

#[derive(Debug)]
pub struct TaskManager {
	pub _watcher: notify::RecommendedWatcher,
	pub dirty: bool,
	pub error: Option<String>,
	pub last_loaded: Option<SystemTime>,
	pub rx: tokio::sync::mpsc::Receiver<()>,
	pub state: Option<EstateState>,
	pub state_path: PathBuf,
	pub tasks: HashMap<TaskId, Task>,
}
impl TaskManager {
	pub fn create(&mut self, kind: TaskKind) -> TaskId {
		let id = Uuid::now_v7();

		let task = Task {
			id,
			name: kind.name(),
			kind,
			status: TaskStatus::Pending,
		};

		self.tasks.insert(id, task);
		self.dirty = true;

		id
	}

	pub fn get(&self, id: TaskId) -> Option<&Task> {
		self.tasks.get(&id)
	}

	fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
		self.tasks.get_mut(&id)
	}

	pub fn count(&self) -> usize {
		self.tasks.len()
	}

	pub fn list(&self) -> impl Iterator<Item = &Task> {
		self.tasks.values()
	}

	pub fn set_status(&mut self, id: TaskId, status: TaskStatus) -> anyhow::Result<()> {
		let task = self
			.tasks
			.get_mut(&id)
			.ok_or_else(|| anyhow::anyhow!("task {id} not found"))?;

		task.status = status;

		Ok(())
	}

	pub fn save(&mut self) -> anyhow::Result<()> {
		todo!("save")
	}

	pub fn clear(&mut self) -> bool {
		if self.tasks.is_empty() {
			return false;
		}

		self.tasks.clear();
		self.dirty = true;
		true
	}

	pub fn delete(&mut self, id: TaskId) -> Option<Task> {
		let task = self.tasks.remove(&id);

		if task.is_some() {
			self.dirty = true;
		}

		task
	}
}
#[derive(Clone, Debug, PartialEq, Eq)]
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
	pub task_id: String,
	pub status: TaskStatus,
	pub summary: Option<String>,
	pub artifacts: Vec<Artifact>,
	pub logs: Vec<String>,
	pub spawned_tasks: Vec<AgentTask>,
	pub chat: Option<String>,
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
			task_id,
			status: TaskStatus::Completed,
			summary: None,
			artifacts: ctx.artifacts,
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			chat: Some(chat),
		}
	}

	pub fn completed_with_summary(task_id: String, ctx: AgentContext, summary: String) -> Self {
		Self {
			task_id,
			status: TaskStatus::Completed,
			summary: Some(summary),
			artifacts: ctx.artifacts,
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			chat: None,
		}
	}

	pub fn failed(
		task_id: String,
		ctx: AgentContext,
		reason: impl Into<String>,
		summary: Option<String>,
	) -> Self {
		Self {
			task_id,
			status: TaskStatus::Failed(reason.into()),
			summary,
			artifacts: ctx.artifacts,
			logs: ctx.logs,
			spawned_tasks: ctx.spawned_tasks,
			chat: None,
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
