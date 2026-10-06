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
impl AgentRuntime {
	pub fn new(
		event_tx: UnboundedSender<RuntimeEvent>,
		registry: AgentRegistry,
		workspace: WorkspaceContext,
	) -> Self {
		Self {
			event_tx,
			registry,
			workspace,
			// metrics: RuntimeMetrics::new(),
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeMetrics {
	// ── Interaction ──────────────────────────────────────────────
	pub prompts_sent: u64,
	pub responses_received: u64,

	// ── Files ────────────────────────────────────────────────────
	pub files_read: u64,
	pub files_written: u64,
	pub files_created: u64,
	pub files_deleted: u64,

	// ── Execution ────────────────────────────────────────────────
	pub commands_run: u64,
	pub tests_run: u64,
	pub tests_passed: u64,
	pub tests_failed: u64,
	pub tool_calls: u64,

	// ── Attempts / reliability ──────────────────────────────────
	pub attempts: u64,
	pub retries: u64,
	pub failures: u64,
	pub timeouts: u64,
	pub cancellations: u64,

	pub stuck_count: u64,
	pub recovery_count: u64,

	pub invalid_actions: u64,
	pub rejected_actions: u64,

	// ── Build / test failures ────────────────────────────────────
	pub compile_failures: u64,
	pub test_failures: u64,

	// ── Tokens / context ─────────────────────────────────────────
	pub tokens_input: u64,
	pub tokens_output: u64,

	pub context_tokens: u64,
	pub disclosed_tokens: u64,
	pub prompt_tokens: u64,
	pub history_tokens: u64,
	pub workspace_tokens: u64,

	// ── Discovery ────────────────────────────────────────────────
	pub discovery_runs: u64,
	pub rules_evaluated: u64,
	pub rules_matched: u64,

	// ── Timing ───────────────────────────────────────────────────
	pub elapsed_ms: u64,
	pub command_durations_ms: Vec<u64>,
	pub tool_durations_ms: Vec<u64>,
	pub stage_durations_ms: Vec<u64>,
}

impl RuntimeMetrics {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn record_command(&mut self, duration_ms: u64) {
		self.commands_run += 1;
		self.command_durations_ms.push(duration_ms);
	}

	pub fn record_tool_call(&mut self, duration_ms: u64) {
		self.tool_calls += 1;
		self.tool_durations_ms.push(duration_ms);
	}

	pub fn record_stage(&mut self, duration_ms: u64) {
		self.stage_durations_ms.push(duration_ms);
	}

	pub fn record_test(&mut self, passed: bool) {
		self.tests_run += 1;

		if passed {
			self.tests_passed += 1;
		} else {
			self.tests_failed += 1;
			self.test_failures += 1;
		}
	}

	pub fn record_compile_failure(&mut self) {
		self.compile_failures += 1;
		self.failures += 1;
	}

	pub fn record_failure(&mut self) {
		self.failures += 1;
	}

	pub fn record_timeout(&mut self) {
		self.timeouts += 1;
	}

	pub fn record_retry(&mut self) {
		self.retries += 1;
	}

	pub fn record_attempt(&mut self) {
		self.attempts += 1;
	}

	pub fn record_recovery(&mut self) {
		self.recovery_count += 1;
	}

	pub fn record_discovery(&mut self, rules_evaluated: u64, rules_matched: u64) {
		self.discovery_runs += 1;
		self.rules_evaluated += rules_evaluated;
		self.rules_matched += rules_matched;
	}

	pub fn record_tokens(&mut self, input: u64, output: u64) {
		self.tokens_input += input;
		self.tokens_output += output;
	}

	pub fn record_file_read(&mut self) {
		self.files_read += 1;
	}

	pub fn record_file_write(&mut self) {
		self.files_written += 1;
	}

	pub fn record_file_create(&mut self) {
		self.files_created += 1;
		self.files_written += 1;
	}

	pub fn record_file_delete(&mut self) {
		self.files_deleted += 1;
	}

	// ── Derived metrics ──────────────────────────────────────────

	pub fn tokens_per_attempt(&self) -> u64 {
		if self.attempts == 0 {
			return 0;
		}

		(self.tokens_input + self.tokens_output) / self.attempts
	}

	pub fn files_per_attempt(&self) -> u64 {
		if self.attempts == 0 {
			return 0;
		}

		(self.files_read + self.files_written) / self.attempts
	}

	pub fn tool_calls_per_attempt(&self) -> u64 {
		if self.attempts == 0 {
			return 0;
		}

		self.tool_calls / self.attempts
	}

	pub fn test_pass_rate(&self) -> f64 {
		if self.tests_run == 0 {
			return 0.0;
		}

		self.tests_passed as f64 / self.tests_run as f64
	}

	pub fn rule_match_rate(&self) -> f64 {
		if self.rules_evaluated == 0 {
			return 0.0;
		}

		self.rules_matched as f64 / self.rules_evaluated as f64
	}
}
