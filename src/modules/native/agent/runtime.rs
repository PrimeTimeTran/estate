use super::{
	Agent, SystemEvent,
	agent_event::{AgentEvent, RuntimeEvent},
};
use crate::{model::task::TaskResult, native::job, prelude::*};

#[derive(Clone, Debug, Default)]
pub struct AgentRegistry;

#[derive(Clone, Debug)]
pub struct AgentRuntime {
	pub event_tx: UnboundedSender<RuntimeEvent>,
	pub registry: AgentRegistry,
}
impl AgentRuntime {
	pub async fn run_agent(&self, task: AgentTask) -> Result<TaskResult> {
		// Start completely fresh.
		// let agent = Agent::new()

		// 		// Or explicitly scan/load a workspace.
		// 		let workspace = WorkspaceContext::load()?;
		// 		let agent = Agent::with_workspace(workspace);
		//
		// 		// Or bind to an SDLC session.
		// 		let workspace = WorkspaceContext::from_session(&session)?;
		// 		let agent = Agent::with_workspace(workspace);
		// 		agent.run_agent_loop(task, self.event_tx.clone()).await

		let cwd = std::env::current_dir()?;
		let agent = Agent::with_cwd(cwd);

		agent.run_agent_loop(task, self.event_tx.clone()).await
	}

	pub async fn spawn_agent(&self, task: AgentTask) {
		let event_tx = self.event_tx.clone();
		tokio::spawn(async move {
			let agent = Agent::new();

			let result = agent.run_agent_loop(task.clone(), event_tx.clone()).await;

			match result {
				Ok(result) => {
					let _ = event_tx.send(RuntimeEvent::System(SystemEvent::TaskCompleted {
						result: result.clone(),
					}));

					for task in result.spawned_tasks {
						let _ = event_tx.send(RuntimeEvent::System(SystemEvent::TaskSpawned { task }));
					}
				}

				Err(e) => {
					let _ = event_tx.send(RuntimeEvent::System(SystemEvent::TaskFailed {
						task_id: task.id.to_string(),
						error: e.to_string(),
					}));
				}
			}
		});
	}
}
