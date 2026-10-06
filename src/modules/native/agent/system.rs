use super::*;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum SystemEvent {
	SpawnAgent { agent_id: String },
	TaskAdd { task_id: String },
	TaskSpawned { task: AgentTask },
	TaskQueued { task_id: String },
	TaskStarted { task_id: String },
	TaskCompleted { result: TaskResult },
	TaskFailed { task_id: String, error: String },
	AgentSpawned { agent_id: String },
	AgentFinished { agent_id: String },
	TaskGroupFinished { group_id: String },
	AllIdle,
}
#[derive(Debug)]
pub struct AgentSystem {
	pub bus: AgentBus,
	pub runtime: AgentRuntime,
	pub event_rx: UnboundedReceiver<RuntimeEvent>,
}
impl AgentSystem {
	pub fn new() -> Self {
		let (cmd_tx, _cmd_rx) = unbounded_channel::<AgentEvent>();
		let (event_tx, event_rx) = unbounded_channel::<RuntimeEvent>();
		let bus = AgentBus {
			tx: cmd_tx,
			event_tx: event_tx.clone(),
		};
		let runtime = AgentRuntime {
			event_tx,
			workspace: WorkspaceContext::default(),
			registry: AgentRegistry::default(),
		};
		Self {
			bus,
			runtime,
			event_rx,
		}
	}
	pub fn cwd(&mut self, cwd: impl Into<PathBuf>) -> &mut Self {
		self.runtime.workspace.cwd = cwd.into();
		self
	}
		pub fn add_file(&mut self, path: impl Into<PathBuf>) -> Result<()> {
			self.runtime.workspace.add_file(path)
		}
	pub fn add_session(&mut self, session: &AiSession) -> Result<&mut Self> {
		// self.runtime.add_session(session)?;
		Ok(self)
	}
}
pub fn new_agent_system() -> (AgentBus, AgentRuntime, UnboundedReceiver<RuntimeEvent>) {
	let (cmd_tx, cmd_rx) = unbounded_channel::<AgentEvent>();
	let (event_tx, event_rx) = unbounded_channel::<RuntimeEvent>();
	let bus = AgentBus {
		tx: cmd_tx,
		event_tx: event_tx.clone(),
	};
	let runtime = AgentRuntime {
		event_tx,
		workspace: WorkspaceContext::default(),
		registry: AgentRegistry::default(),
	};
	(bus, runtime, event_rx)
}

pub async fn handle_event(event: AgentEvent, runtime: &AgentRuntime) {
	match event {
		AgentEvent::NewTask { task } => {
			runtime.spawn_agent(task).await;
		}
		AgentEvent::Finished { result } => {
			let _ = runtime
				.event_tx
				.send(RuntimeEvent::System(SystemEvent::TaskCompleted { result }));
		}
		_ => {}
	}
}
