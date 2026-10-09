use super::*;
use serde::{Deserialize, Serialize};

impl Agent {
	pub fn new() -> Self {
		Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(CtxWorkspace::default()),
		}
	}
	async fn pick_mode(&self, ctx: &AgentCtx) -> Result<AgentMode> {
		let prompt = build_sys_prompt(DECIDE_PROMPT, &ctx.prompt.as_deref().unwrap_or(""));
		let raw: LlmMode = prompt_ollama_json(&prompt).await?;

		Ok(match raw.mode.as_str() {
			"tool" => AgentMode::Tool,
			_ => AgentMode::Chat,
		})
	}
	async fn pick_action(&self, ctx: &AgentCtx) -> Result<AgentAction> {
		let prompt = build_prompt(ctx);
		let raw = build_action(&prompt).await?;
		let action = AgentAction::try_from(raw)?;
		Ok(action)
	}
	pub async fn run_agent(&self, prompt: &str) -> Result<String> {
		let task = AgentTask::new(prompt.to_string());
		let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
		let result = self.run_agent_loop(task, event_tx).await?;
		Ok(
			result
				.chat
				.or(result.summary)
				.unwrap_or_else(|| "Agent completed".to_string()),
		)
	}
	pub async fn run_agent_loop(
		&self,
		task: AgentTask,
		event_tx: UnboundedSender<RuntimeEvent>,
	) -> Result<TaskResult> {
		let max_steps = 20;
		let max_action_selection_errors = 3;

		let ctx = AgentCtx::with_workspace(task.prompt.clone(), (*self.workspace).clone());
		let mut run = AgentRun::new(ctx);
		let mut consecutive_selection_errors = 0;

		section!("AGENT run_agent_loop CONTEXT");

		let prompt = run.ctx.prompt.as_deref().unwrap_or("");
		println!(
			"ctx.prompt ({} chars, {} lines):\n{}",
			prompt.len(),
			prompt.lines().count(),
			preview_lines(prompt, PROMPT_PREVIEW_LINES)
		);
		println!("ctx.workspace:\n{}", run.ctx.workspace);
		println!("ctx.history ({} entries):", run.ctx.history.len());

		let event = RuntimeEvent::Agent(AgentEvent::Thinking { task: task.clone() });

		if let Err(error) = crate::agent::log::append_agent_event(&event) {
			eprintln!("Failed to write agent event log: {error:#}");
		}
		let _ = event_tx.send(event);

		let mode = self.pick_mode(&run.ctx).await?;

		if matches!(mode, AgentMode::Chat) {
			let response = prompt_chat(&run.ctx).await?;
			return Ok(TaskResult::completed_chat(task.id, run.ctx, response));
		}

		// Baseline for the entire agent run. Per-command snapshots below let us
		// attribute changes to individual actions; this baseline captures the
		// complete set of changes made during this run.
		let run_workspace_before = WSSnapshot::capture(&self.workspace.cwd)?;

		for step in 1..=max_steps {
			println!(
				"AGENT STEP {step}/{max_steps}; history={}; commands={}; errors={}; selection_errors={}",
				run.ctx.history.len(),
				run.commands.len(),
				run.errors.len(),
				consecutive_selection_errors,
			);

			let action = match self.pick_action(&run.ctx).await {
				Ok(action) => {
					consecutive_selection_errors = 0;
					action
				}
				Err(error) => {
					consecutive_selection_errors += 1;

					let message = format!("Action selection failed: {error:#}");
					eprintln!("{message}");

					run.record_error(AgentError {
						step: Some(step),
						kind: "action_selection".into(),
						message: message.clone(),
						raw_response: None,
						recoverable: consecutive_selection_errors < max_action_selection_errors,
						timestamp: chrono::Utc::now(),
					});

					if consecutive_selection_errors >= max_action_selection_errors {
						let failure = format!(
							"{message}\n\
							Reached {max_action_selection_errors} consecutive action-selection failures. \
							Check the raw model response and LlmAction schema."
						);

						return Ok(TaskResult::failed(
							task.id,
							run.ctx,
							"Action selection failed repeatedly",
							Some(failure),
						));
					}

					run.ctx.history.push(AgentObservation::Current {
						message: format!(
							"INVALID ACTION RESPONSE\n\
							Reason: {message}\n\n\
							The response was rejected before a valid action was selected. \
							No command from this response was executed.\n\n\
							Return exactly ONE valid JSON object:\n\
							{{\"action\":\"run_command\",\"command\":\"one non-interactive command\"}}\n\
							{{\"action\":\"finish\",\"message\":\"summary of work and verification\"}}\n\n\
							Only `run_command` and `finish` are supported. Put the shell command \
							in `command`, not in `action`. Use bounded, non-interactive commands. \
							Do not start an interactive shell, REPL, or persistent foreground process. \
							Do not repeat successful commands without a reason or retry failed \
							commands unchanged without understanding the failure.\n\
							Return the corrected action now."
						),
					});

					continue;
				}
			};

			match action {
				AgentAction::Current { message } => {
					let now = chrono::Local::now().format("%Y-%m-%d").to_string();
					let response = format!("Context update: The current date is {now}. {message}");

					let event = RuntimeEvent::Agent(AgentEvent::Working {
						task: task.clone(),
						message: response.clone(),
					});

					if let Err(error) = crate::agent::log::append_agent_event(&event) {
						eprintln!("Failed to write agent event log: {error:#}");
					}
					let _ = event_tx.send(event);

					run
						.ctx
						.history
						.push(AgentObservation::Current { message: response });
				}

				AgentAction::Context { path } => {
					let message = match path {
						Some(path) => format!(
							"Context requested for path: {path}. \
							Inspect it using a supported command, respecting the task's scope."
						),
						None => format!(
							"Context requested. Current history entries: {}. \
							Use supported commands to inspect only what the task authorizes.",
							run.ctx.history.len()
						),
					};

					let event = RuntimeEvent::Agent(AgentEvent::Working {
						task: task.clone(),
						message: message.clone(),
					});

					if let Err(error) = crate::agent::log::append_agent_event(&event) {
						eprintln!("Failed to write agent event log: {error:#}");
					}
					let _ = event_tx.send(event);

					run.ctx.history.push(AgentObservation::Current { message });
				}

				AgentAction::RunCommand { command } => {
					println!("COMMAND: {command}");

					// Capture immediately before execution so changes can be
					// attributed to this command rather than the entire run.
					let workspace_before = WSSnapshot::capture(&self.workspace.cwd)?;

					let mut shell_command = ShellCommand::shell(command.clone());
					shell_command.cwd = Some(self.workspace.cwd.clone());

					let result = match self.tools.shell.run(shell_command).await {
						Ok(result) => result,
						Err(error) => {
							// Even if the shell tool errors, the command may have
							// modified files before failing. Reconcile those changes.
							let workspace_after = WSSnapshot::capture(&self.workspace.cwd)?;

							let changed_paths =
								reconcile_agent_artifacts(&mut run.ctx, &workspace_before, &workspace_after)?;

							let message = format!(
								"Shell execution failed for `{command}`: {error:#}\n\
								Observed workspace changes:\n{}",
								format_changed_files(&changed_paths),
							);
							eprintln!("{message}");

							run.record_error(AgentError {
								step: Some(step),
								kind: "command_execution".into(),
								message: message.clone(),
								raw_response: None,
								recoverable: true,
								timestamp: chrono::Utc::now(),
							});

							run.ctx.history.push(AgentObservation::Current {
								message: format!(
									"{message}\n\n\
									The shell tool did not return a completed result. \
									Do not assume there were no side effects. Inspect the \
									observed changes and error before choosing the next action. \
									Do not retry unchanged without justification."
								),
							});

							continue;
						}
					};

					section!("SHELL RESULT");
					println!("command: {command}");
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

					// Reconcile the filesystem regardless of exit status. A failing
					// command can still leave useful files or partial modifications.
					let workspace_after = WSSnapshot::capture(&self.workspace.cwd)?;

					let changed_paths =
						reconcile_agent_artifacts(&mut run.ctx, &workspace_before, &workspace_after)?;

					println!(
						"FILES CHANGED BY ACTION:\n{}",
						format_changed_files(&changed_paths),
					);

					let success = result.exit_code == Some(0);

					run.record_command(CommandRecord {
						step,
						command: command.clone(),
						exit_code: result.exit_code,
						stdout: result.stdout.clone(),
						stderr: result.stderr.clone(),
						success,
						error: if success {
							None
						} else {
							Some(format!("Command exited with status {:?}", result.exit_code))
						},
						timestamp: chrono::Utc::now(),
					});

					if !success {
						run.record_error(AgentError {
							step: Some(step),
							kind: "command_exit".into(),
							message: format!(
								"Command `{command}` exited with status {:?}.\n\
								Stderr:\n{}",
								result.exit_code,
								preview(&result.stderr, 1000),
							),
							raw_response: None,
							recoverable: true,
							timestamp: chrono::Utc::now(),
						});
					}

					// Give the next model call observed filesystem evidence rather
					// than relying on the model's claims about what it wrote.
					run.ctx.history.push(AgentObservation::Current {
						message: format!(
							"Runtime filesystem reconciliation after command `{command}`:\n\
							{}\n\
							These changes were observed by the runtime. Inspect and verify \
							the relevant files before claiming completion.",
							format_changed_files(&changed_paths),
						),
					});

					run
						.ctx
						.history
						.push(AgentObservation::RunCommand { result });
				}

				AgentAction::Finish { message } => {
					// Capture the final diff too: this includes changes that occurred
					// outside the last individual command's snapshot interval.
					let workspace_final = WSSnapshot::capture(&self.workspace.cwd)?;

					let final_changed_paths =
						reconcile_agent_artifacts(&mut run.ctx, &run_workspace_before, &workspace_final)?;

					// A successful command alone is not evidence of task completion.
					let validation = validate_agent_completion(
						&self.workspace.cwd,
						&run.ctx.artifacts,
						&run.commands,
						&final_changed_paths,
					)?;

					if !validation.is_valid() {
						let errors = validation.errors();

						let feedback = format!(
							"COMPLETION REJECTED: runtime validation failed.\n\
							You claimed the task was complete, but these checks failed:\n{}\n\n\
							Correct the actual files, then inspect and verify them. \
							Do not simply repeat the completion claim.",
							errors.join("\n"),
						);

						eprintln!("{feedback}");

						run.record_error(AgentError {
							step: Some(step),
							kind: "completion_validation".into(),
							message: feedback.clone(),
							raw_response: None,
							recoverable: true,
							timestamp: chrono::Utc::now(),
						});

						run
							.ctx
							.history
							.push(AgentObservation::Current { message: feedback });

						continue;
					}

					let final_message = format!(
						"{message}\n\nRuntime-observed changed files:\n{}",
						format_changed_files(&final_changed_paths),
					);

					let result = TaskResult::completed_with_summary(task.id, run.ctx, final_message);

					let event = RuntimeEvent::Agent(AgentEvent::Finished {
						result: result.clone(),
					});

					if let Err(error) = crate::agent::log::append_agent_event(&event) {
						eprintln!("Failed to write agent event log: {error:#}");
					}
					let _ = event_tx.send(event);

					return Ok(result);
				}
			}
		}

		let message = format!(
			"Agent exceeded {max_steps} reasoning steps; commands={}, errors={}.",
			run.commands.len(),
			run.errors.len(),
		);

		run.record_error(AgentError {
			step: Some(max_steps),
			kind: "step_limit".into(),
			message: message.clone(),
			raw_response: None,
			recoverable: false,
			timestamp: chrono::Utc::now(),
		});

		Ok(TaskResult::failed(
			task.id,
			run.ctx,
			"Step limit exceeded",
			Some(message),
		))
	}
	pub fn with_cwd(cwd: impl Into<PathBuf>) -> Self {
		Self::with_workspace(CtxWorkspace::from_cwd(cwd))
	}
	pub fn with_ctx(ctx: AgentCtx, _session: &AiSession) -> Result<Self> {
		Ok(Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(ctx.workspace),
		})
	}
	pub fn with_workspace(workspace: CtxWorkspace) -> Self {
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
impl AgentRun {
	pub fn new(ctx: AgentCtx) -> Self {
		Self {
			ctx,
			commands: Vec::new(),
			errors: Vec::new(),
		}
	}
	pub fn record_command(&mut self, record: CommandRecord) {
		self.commands.push(record);
	}
	pub fn record_error(&mut self, error: AgentError) {
		self.errors.push(error);
	}
	pub fn record_observation(&mut self, observation: AgentObservation) {
		self.ctx.history.push(observation);
	}
	pub fn record_log(&mut self, message: impl Into<String>) {
		self.ctx.logs.push(message.into());
	}
	pub fn step_count(&self) -> usize {
		self.commands.len() + self.errors.len()
	}
	pub fn has_successful_command(&self) -> bool {
		self.commands.iter().any(|command| command.success)
	}
	pub fn has_errors(&self) -> bool {
		!self.errors.is_empty()
	}
	pub fn last_command(&self) -> Option<&CommandRecord> {
		self.commands.last()
	}
	pub fn last_error(&self) -> Option<&AgentError> {
		self.errors.last()
	}
	pub fn into_context(self) -> AgentCtx {
		self.ctx
	}
}
impl AgentTask {
	pub fn new(prompt: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			prompt,
		}
	}
	pub fn from_session(session: &AiSession) -> Result<Self> {
		Ok(Self {
			id: Uuid::new_v4(),
			prompt: session.prompt.clone(),
		})
	}
}

impl AgentCtx {
	pub fn new(user_prompt: String) -> Self {
		Self {
			prompt: Some(user_prompt.clone()),
			task: Some(AgentTask::new(user_prompt)),
			workspace: CtxWorkspace::default(),
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		}
	}
	pub fn init() -> Self {
		Self {
			prompt: None,
			task: None,
			workspace: CtxWorkspace::default(),
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		}
	}
	pub fn with_workspace(user_prompt: String, workspace: CtxWorkspace) -> Self {
		Self {
			prompt: Some(user_prompt.clone()),
			task: Some(AgentTask::new(user_prompt)),
			workspace,
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		}
	}
	pub fn from_session(session: &AiSession) -> Result<Self> {
		let workspace = CtxWorkspace::from_session(session)?;
		Self::from_session_with_workspace(session, &workspace)
	}
	pub fn from_session_with_workspace(
		session: &AiSession,
		workspace: &CtxWorkspace,
	) -> Result<Self> {
		Ok(Self {
			prompt: Some(session.prompt.clone()),
			task: Some(AgentTask::new(session.prompt.clone())),
			workspace: workspace.clone(),
			history: Vec::new(),
			artifacts: Vec::new(),
			logs: Vec::new(),
			spawned_tasks: Vec::new(),
		})
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
	pub workspace: Arc<CtxWorkspace>,
}
#[derive(Clone, Debug)]
pub struct AgentBus {
	pub tx: UnboundedSender<AgentEvent>,
	pub event_tx: UnboundedSender<RuntimeEvent>,
}
pub struct AgentContextInfo {
	pub cwd: PathBuf,
	pub workspace_dir: PathBuf,
	pub project_dir: PathBuf,
	pub settings_file: Option<PathBuf>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AgentCtx {
	pub prompt: Option<String>,
	pub task: Option<AgentTask>,
	pub workspace: CtxWorkspace,
	pub history: Vec<AgentObservation>,
	pub artifacts: Vec<Artifact>,
	pub logs: Vec<String>,
	pub spawned_tasks: Vec<AgentTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
	pub ctx: AgentCtx,
	pub commands: Vec<CommandRecord>,
	pub errors: Vec<AgentError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
	/// One-based execution order within this agent run.
	pub step: usize,

	/// The command submitted to the shell.
	pub command: String,

	/// Process exit code, if one was produced.
	pub exit_code: Option<i32>,

	/// Captured standard output.
	pub stdout: String,

	/// Captured standard error.
	pub stderr: String,

	/// Whether the command completed successfully.
	pub success: bool,

	/// Error encountered while launching or executing the command, if any.
	pub error: Option<String>,

	/// Timestamp when the command record was created.
	pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentError {
	/// One-based agent step where the error occurred, if known.
	pub step: Option<usize>,

	/// Category of failure, e.g. "action_parse", "command", or "tool".
	pub kind: String,

	/// Human-readable error description.
	pub message: String,

	/// Raw model response, if the error occurred while parsing an action.
	pub raw_response: Option<String>,

	/// Whether the error was recoverable.
	pub recoverable: bool,

	/// Timestamp when the error was recorded.
	pub timestamp: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AgentTask {
	pub id: Uuid,
	pub prompt: String,
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
//
// mod traits {
//  	use super::*;
// 	pub trait CtxAgent {
// 		fn task(&self) -> &AgentTask;
// 		fn workspace(&self) -> &CtxWorkspace;
// 		fn history(&self) -> &[AgentObservation];
// 		fn record(&mut self, observation: AgentObservation);
// 		fn observe(&self) -> AgentContextInfo;
// 	}
// }

fn format_changed_files(paths: &[std::path::PathBuf]) -> String {
	if paths.is_empty() {
		return "(no filesystem changes detected)".to_owned();
	}

	let mut paths = paths
		.iter()
		.map(|path| path.display().to_string())
		.collect::<Vec<_>>();

	paths.sort();
	paths.dedup();

	paths
		.into_iter()
		.map(|path| format!("- {path}"))
		.collect::<Vec<_>>()
		.join("\n")
}

use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct AgentCompletionValidation {
	errors: Vec<String>,
}

impl AgentCompletionValidation {
	pub fn is_valid(&self) -> bool {
		self.errors.is_empty()
	}

	pub fn errors(&self) -> &[String] {
		&self.errors
	}
}

pub fn validate_agent_completion(
	workspace_root: &Path,
	artifacts: &[Artifact],
	commands: &[CommandRecord],
	changed_paths: &[PathBuf],
) -> Result<AgentCompletionValidation> {
	let mut errors = Vec::new();
	let mut paths: Vec<PathBuf> = artifacts
		.iter()
		.filter_map(|artifact| match artifact {
			Artifact::FileRead { path, .. } | Artifact::FileWrite { path } => Some(PathBuf::from(path)),
			Artifact::Observation(_) | Artifact::ToolOutput(_) => None,
		})
		.collect();

	paths.extend(changed_paths.iter().cloned());
	paths.sort();
	paths.dedup();

	if !changed_paths.is_empty() && !commands.iter().any(|command| command.success) {
		errors.push(
			"Files changed, but no command completed successfully. \
			 Run an appropriate verification command before finishing."
				.to_owned(),
		);
	}

	Ok(AgentCompletionValidation { errors })
}

fn reconcile_agent_artifacts(
	ctx: &mut AgentCtx,
	before: &WSSnapshot,
	after: &WSSnapshot,
) -> Result<Vec<PathBuf>> {
	let changes = before.diff(after);
	let changed_paths = parse_git_status_paths(&changes.git_status);

	for path in &changed_paths {
		let absolute_path = if path.is_absolute() {
			path.clone()
		} else {
			ctx.workspace.cwd.join(path)
		};

		let path_string = absolute_path.to_string_lossy().into_owned();

		if absolute_path.is_file() {
			ctx.workspace.add_file(absolute_path.clone())?;

			let artifact = Artifact::FileWrite {
				path: path_string.clone(),
			};

			// Avoid duplicate write artifacts for the same path.
			if !ctx.artifacts.iter().any(|existing| {
				matches!(
					existing,
					Artifact::FileWrite { path } if path == &path_string
				)
			}) {
				ctx.artifacts.push(artifact);
			}
		} else {
			// Remove stale workspace entries for deleted files.
			ctx
				.workspace
				.files
				.retain(|file| Path::new(&file.path) != absolute_path);

			// Remove write artifacts for files that no longer exist.
			ctx.artifacts.retain(|artifact| {
				!matches!(
					artifact,
					Artifact::FileWrite { path } if path == &path_string
				)
			});
		}
	}

	Ok(changed_paths)
}

fn parse_git_status_paths(status: &str) -> Vec<PathBuf> {
	let mut paths = Vec::new();

	for line in status.lines() {
		// `git status --short` starts with two status columns.
		if line.len() < 4 {
			continue;
		}

		let status_code = &line[..2];
		let path = line[3..].trim();

		if path.is_empty() || status_code == "!!" {
			continue;
		}

		// For renames, porcelain output is generally "old -> new".
		let path = path
			.rsplit_once(" -> ")
			.map(|(_, new_path)| new_path)
			.unwrap_or(path)
			.trim_matches('"');

		paths.push(PathBuf::from(path));
	}

	paths.sort();
	paths.dedup();
	paths
}
