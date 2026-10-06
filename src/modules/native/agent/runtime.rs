use super::*;
use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct AgentRegistry;

#[derive(Clone, Debug)]
pub struct AgentRuntime {
	pub event_tx: UnboundedSender<RuntimeEvent>,
	pub registry: AgentRegistry,
	pub workspace: WorkspaceContext,
}
impl AgentRuntime {
	pub fn workspace(&self) -> &WorkspaceContext {
		&self.workspace
	}
	pub async fn run_agent_with_sdlc(
		&self,
		task: AgentTask,
		sdlc_dir: impl AsRef<std::path::Path>,
	) -> Result<TaskResult> {
		let agent = Agent::with_cwd(sdlc_dir.as_ref().to_path_buf());
		agent.run_agent_loop(task, self.event_tx.clone()).await
	}
	pub async fn from_session(&self, task: AgentTask, session: &AiSession) -> Result<TaskResult> {
		let ctx = AgentContext::from_session_with_workspace(session, &self.workspace)?;
		let agent = Agent::with_ctx(ctx, session)?;
		agent.run_agent_loop(task, self.event_tx.clone()).await
	}
	pub async fn run_agent(&self, task: AgentTask) -> Result<TaskResult> {
		let cwd = std::env::current_dir()?;
		let agent = Agent::with_cwd(cwd);
		agent.run_agent_loop(task, self.event_tx.clone()).await		// 		// Start completely fresh.
		// 		let agent = Agent::new()
		//
		// 		// Or explicitly scan/load a workspace.
		// 		let workspace = WorkspaceContext::load()?;
		// 		let agent = Agent::with_workspace(workspace);
		//
		// 		// Or bind to an SDLC session.
		// 		let workspace = WorkspaceContext::from_session(&session)?;
		// 		let agent = Agent::with_workspace(workspace);
		// 		agent.run_agent_loop(task, self.event_tx.clone()).await

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
