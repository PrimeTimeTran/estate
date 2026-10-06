use super::*;

#[derive(Debug)]
pub enum AgentMode {
	Chat,
	Tool,
}
#[derive(PartialEq, Clone)]
pub enum AgentStatus {
	Done,
	Waiting,
	Thinking,
	Error(String),
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AgentObservation {
	Current { message: String },
	RunCommand { result: ShellResult },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AgentAction {
	#[serde(rename = "finish")]
	Finish { message: String },
	#[serde(rename = "current")]
	Current { message: String },
	#[serde(rename = "run_command")]
	RunCommand { command: String },
	#[serde(rename = "context")]
	Context { path: Option<String> },
}

impl Agent {
	pub fn new() -> Self {
		Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(WorkspaceContext::default()),
		}
	}

	pub fn with_workspace(workspace: WorkspaceContext) -> Self {
		Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(workspace),
		}
	}

	pub fn with_cwd(cwd: impl Into<PathBuf>) -> Self {
		Self::with_workspace(WorkspaceContext::from_cwd(cwd))
	}

	pub fn with_ctx(ctx: AgentContext, _session: &AiSession) -> Result<Self> {
		Ok(Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(ctx.workspace),
		})
	}
	// pub async fn from_session(&self, task: AgentTask, session: &AiSession) -> Result<TaskResult> {
	// 	let ctx = AgentContext::from_session_with_workspace(&session, &self.workspace.clone())?;
	// 	let agent = Agent::with_ctx(ctx, session)?;
	// 	agent.run_agent_loop(task, self.event_tx.clone()).await
	// }
}

impl Default for Agent {
	fn default() -> Self {
		Self::new()
	}
}
impl AgentContext {
	pub fn new(user_prompt: String) -> Self {
		Self {
			prompt: user_prompt.clone(),
			task: AgentTask::new(user_prompt),
			workspace: WorkspaceContext::default(),
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		}
	}
	pub fn with_workspace(user_prompt: String, workspace: WorkspaceContext) -> Self {
		Self {
			prompt: user_prompt.clone(),
			task: AgentTask::new(user_prompt),
			workspace,
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		}
	}
	pub fn from_session(session: &AiSession) -> Result<Self> {
		let workspace = WorkspaceContext::from_session(session)?;
		Self::from_session_with_workspace(session, &workspace)
	}

	pub fn from_session_with_workspace(
		session: &AiSession,
		workspace: &WorkspaceContext,
	) -> Result<Self> {
		Ok(Self {
			prompt: session.prompt.clone(),
			task: AgentTask::new(session.prompt.clone()),
			workspace: workspace.clone(),
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		})
	}
	// pub fn from_session(session: &AiSession) -> Result<Self> {
	// let ws = WorkspaceContext::from_session(session)?;
	// Ok(Self {
	// id: uuid::Uuid::new_v4().to_string(),
	// tools: AgentTools::default(),
	// workspace: Arc::new(ws),
	// })
	// }
}

fn preview(value: impl std::fmt::Debug, max_len: usize) -> String {
	let value = format!("{value:?}");

	if value.len() > max_len {
		format!(
			"{}... [truncated, true_len={}]",
			&value[..max_len],
			value.len()
		)
	} else {
		value
	}
}
impl Agent {
	pub async fn run_agent_loop(
		&self,
		task: AgentTask,
		event_tx: UnboundedSender<RuntimeEvent>,
	) -> Result<TaskResult> {
		let mut steps = 0;
		let max_steps = 10;
		let mut ctx = AgentContext::with_workspace(task.prompt.clone(), (*self.workspace).clone());
		section!("CONTEXT");
		println!(
			"ctx.prompt ({} chars, {} lines):\n{}",
			ctx.prompt.len(),
			ctx.prompt.lines().count(),
			preview_lines(&ctx.prompt, PROMPT_PREVIEW_LINES)
		);
		println!("ctx.workspace:\n{}", ctx.workspace);
		println!("ctx.history ({} entries):", ctx.history.len());
		for (i, entry) in ctx.history.iter().take(5).enumerate() {
			println!("  [{}] {}", i + 1, preview(&format!("{entry:?}"), 500));
		}
		if ctx.history.len() > 5 {
			println!("  ... {} more entries", ctx.history.len() - 5);
		}
		let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Thinking {
			task: task.clone(),
		}));
		let mode: AgentMode = self.pick_mode(&ctx).await?;
		if matches!(mode, AgentMode::Chat) {
			let response = prompt_chat(&ctx).await?;
			let result = TaskResult::completed_chat(task.id, ctx, response);
			let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Finished {
				result: result.clone(),
			}));
			return Ok(result);
		}
		loop {
			steps += 1;
			if steps > max_steps {
				return Ok(TaskResult::failed(
					task.id,
					ctx,
					"Infinite loop",
					Some("Agent exceeded maximum reasoning steps".into()),
				));
			}
			let action = self.pick_action(&ctx).await?;
			match action {
				AgentAction::Current { message } => {
					let now = chrono::Local::now().format("%Y-%m-%d").to_string();
					let response = format!("Context update: The current date is {}. {}", now, message);
					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Working {
						task: task.clone(),
						message: response.clone(),
					}));
					ctx
						.history
						.push(AgentObservation::Current { message: response });
				}
				AgentAction::Finish { message } => {
					if ctx.history.is_empty() {
						return Err(anyhow!(
							"Agent attempted to finish without performing any work"
						));
					}
					let result = TaskResult::completed_with_summary(task.id, ctx, message);
					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Finished {
						result: result.clone(),
					}));
					return Ok(result);
				}
				AgentAction::RunCommand { command } => {
					let shell_command = ShellCommand::shell(command.clone());
					let result = self.tools.shell.run(shell_command).await?;

					section!("SHELL RESULT");
					println!("exit: {:?}", result.exit_code);
					println!(
						"stdout ({} chars, {} lines):\n{}",
						result.stdout.len(),
						result.stdout.lines().count(),
						preview_lines(&result.stdout, SHELL_OUTPUT_PREVIEW_LINES)
					);
					println!(
						"stderr ({} chars, {} lines):\n{}",
						result.stderr.len(),
						result.stderr.lines().count(),
						preview_lines(&result.stderr, SHELL_OUTPUT_PREVIEW_LINES)
					);
					ctx.history.push(AgentObservation::RunCommand { result });
				}
				AgentAction::Context { .. } => {
					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Working {
						task: task.clone(),
						message: "Inspecting agent context".into(),
					}));

					// eventually:
					// let context = self.context.inspect()?;

					ctx.history.push(AgentObservation::Current {
						message: "Agent context requested".into(),
					});
				}
				AgentAction::Context { .. } => {
					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Working {
						task: task.clone(),
						message: "Inspecting agent context".into(),
					}));

					// eventually:
					// let context = self.context.inspect()?;

					ctx.history.push(AgentObservation::Current {
						message: "Agent context requested".into(),
					});
				} // 				AgentAction::ReadFile { path } => {
				  // 					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Working {
				  // 						task: task.clone(),
				  // 						message: format!("Reading {path}"),
				  // 					}));
				  // 					let content = self.tools.fs.read(&path)?;
				  // 					ctx
				  // 						.history
				  // 						.push(AgentObservation::ReadFile { path, content });
				  // 				}
				  // 				AgentAction::WriteFile { path, content } => {
				  // 					let _ = event_tx.send(RuntimeEvent::Agent(AgentEvent::Working {
				  // 						task: task.clone(),
				  // 						message: format!("Writing {path}"),
				  // 					}));
				  // 					self.tools.fs.write(&path, &content)?;
				  // 					ctx.history.push(AgentObservation::WriteFile {
				  // 						path,
				  // 						success: true,
				  // 					});
				  // 				}
			}
		}
	}
	async fn pick_mode(&self, ctx: &AgentContext) -> Result<AgentMode> {
		let prompt = build_sys_prompt(DECIDE_PROMPT, &ctx.prompt);
		let raw: LlmMode = prompt_ollama_json(&prompt).await?;
		Ok(match raw.mode.as_str() {
			"tool" => AgentMode::Tool,
			_ => AgentMode::Chat,
		})
	}
	async fn pick_action(&self, ctx: &AgentContext) -> Result<AgentAction> {
		let prompt = build_prompt(ctx);
		let raw = build_action(&prompt).await?;
		let action = AgentAction::try_from(raw)?;
		Ok(action)
	}
}
impl TryFrom<LlmAction> for AgentAction {
	type Error = Error;

	fn try_from(v: LlmAction) -> Result<Self, Self::Error> {
		match v.action.as_str() {
			// 			"read_file" => Ok(Self::ReadFile {
			// 				path: v.path.ok_or_else(|| anyhow!("missing path"))?,
			// 			}),
			//
			// 			"write_file" => Ok(Self::WriteFile {
			// 				path: v.path.ok_or_else(|| anyhow!("missing path"))?,
			// 				content: v.content.ok_or_else(|| anyhow!("missing content"))?,
			// 			}),
			//
			// 			"current" => Ok(Self::Current {
			// 				message: v.message.unwrap_or_default(),
			// 			}),
			"run_command" => Ok(Self::RunCommand {
				command: v.command.ok_or_else(|| anyhow!("missing command"))?,
			}),

			"finish" => Ok(Self::Finish {
				message: v.message.unwrap_or_default(),
			}),

			other => Err(anyhow!("unknown action: {}", other)),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Agent {
	pub id: String,
	pub tools: AgentTools,
	pub workspace: Arc<WorkspaceContext>,
}
#[derive(Clone, Debug)]
pub struct AgentBus {
	pub tx: UnboundedSender<AgentEvent>,
	pub event_tx: UnboundedSender<RuntimeEvent>,
}

#[derive(Debug)]
pub struct AgentContext {
	pub prompt: String,
	pub task: AgentTask,
	pub workspace: WorkspaceContext,
	pub history: Vec<AgentObservation>,

	pub artifacts: Vec<Artifact>,
	pub logs: Vec<String>,
	pub spawned_tasks: Vec<AgentTask>,
}
#[derive(Debug, Deserialize)]
pub struct LlmAction {
	pub action: String,
	pub path: Option<String>,
	pub content: Option<String>,
	pub command: Option<String>,
	pub message: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct LlmMode {
	pub mode: String,
}

pub struct AgentContextInfo {
	pub cwd: PathBuf,
	pub workspace_dir: PathBuf,
	pub project_dir: PathBuf,
	pub settings_file: Option<PathBuf>,
}
pub mod traits {
	use super::*;
	pub trait AgentContext {
		fn task(&self) -> &AgentTask;
		fn workspace(&self) -> &WorkspaceContext;
		fn history(&self) -> &[AgentObservation];
		fn record(&mut self, observation: AgentObservation);
		fn observe(&self) -> AgentContextInfo;
	}
}
