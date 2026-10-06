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
	#[serde(rename = "run_command")]
	Context,
}

fn build_prompt(ctx: &AgentContext) -> String {
	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history);
	let prompt = build_sys_action(
		ACTION_PROMPT_EXECUTION,
		&[&ctx.prompt, &workspace, &history],
	);
	section!("BUILT PROMPT");
	println!(
		"prompt ({} lines, {} chars):\n{}",
		prompt.lines().count(),
		prompt.len(),
		preview_lines(&prompt, PROMPT_PREVIEW_LINES)
	);
	prompt
}

async fn build_action(prompt: &str) -> Result<LlmAction> {
	let client = reqwest::Client::new();
	let system_prompt: &str = JSON_PROMPT_EXECUTION;
	let payload = serde_json::json!({
			"model": "qwen3:8b",
			"system": system_prompt,
			"prompt": prompt,
			"stream": false,
			"format": "json"
	});
	let res = client
		.post(crate::AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await?
		.json::<serde_json::Value>()
		.await?;
	let response = res["response"].as_str().unwrap_or("");
	let action: serde_json::Value = serde_json::from_str(response)?;
	println!(
		"action choice   {}",
		action["message"].as_str().unwrap_or("")
	);
	let response_text = res["response"].as_str().unwrap_or("{}");
	let raw: LlmAction = serde_json::from_str(response_text)?;
	Ok(raw)
}
pub fn format_workspace(workspace: &WorkspaceContext) -> String {
	let mut output = String::new();

	output.push_str(&format!("CWD: {}\n", workspace.cwd.to_string_lossy()));

	if workspace.files.is_empty() {
		output.push_str("FILES: none discovered\n");
	} else {
		output.push_str("FILES:\n");

		for file in &workspace.files {
			output.push_str(&format!("- {}\n", file.path));
		}
	}

	output
}
fn format_history(history: &[AgentObservation]) -> String {
	if history.is_empty() {
		return "No actions have been performed yet.".into();
	}

	let mut output = String::new();

	for (index, observation) in history.iter().enumerate() {
		output.push_str(&format!("{}. {:?}\n", index + 1, observation));
	}

	output
}
pub async fn prompt_chat(ctx: &AgentContext) -> Result<String> {
	let prompt = structured_prompt_chat(ctx);
	let result = ollama_generate(&prompt, None, false).await?;
	Ok(result)
}
pub async fn prompt_ollama_json<T>(prompt: &str) -> Result<T>
where
	T: DeserializeOwned,
{
	let result = ollama_generate(prompt, Some("You are a helpful assistant"), true).await?;
	Ok(serde_json::from_str(&result)?)
}
pub async fn ollama_generate(prompt: &str, system: Option<&str>, json: bool) -> Result<String> {
	let client = reqwest::Client::new();
	let mut payload = serde_json::json!({
			"model": "qwen3:8b",
			"prompt": prompt,
			"stream": false,
	});
	if let Some(sys_msg) = system {
		payload["system"] = serde_json::json!(sys_msg);
	}
	if json {
		payload["format"] = serde_json::json!("json");
	}
	let response = client
		.post(crate::AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await?;
	let res: serde_json::Value = response.json().await?;
	res["response"]
		.as_str()
		.map(|s| s.to_string())
		.ok_or_else(|| anyhow!("Failed to parse response field from Ollama"))
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
		let workspace = WorkspaceContext::from_cwd(cwd);

		Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(workspace),
		}
	}
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

	pub fn from_session(session: &SdlcSession) -> Result<Self> {
		let workspace = WorkspaceContext::from_session(session)?;

		Ok(Self {
			prompt: session.goal.clone(),
			task: AgentTask::from_session(session)?,
			workspace,
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		})
	}
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
		// let mut ctx = AgentContext::new(task.prompt.clone(), (*self.workspace).clone());
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
	async fn from_session(session: &Session) -> Result<Agent> {
		todo!("from_session")
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

pub fn structured_prompt_chat(ctx: &AgentContext) -> String {
	format!(
		r#"
			You are a helpful assistant.
			User request:
			{}
			History:
			{}
			Respond normally. No JSON. Just text.
		"#,
		ctx.prompt,
		format_history(&ctx.history)
	)
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
