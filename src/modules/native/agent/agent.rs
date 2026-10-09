use super::*;

impl Agent {
	pub fn new() -> Self {
		Self {
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(CtxWorkspace::default()),
			files_seen: 0,
			tokens_seen: 0,
		}
	}
	fn emit_thinking(&self, task: &AgentTask, event_tx: &UnboundedSender<RuntimeEvent>) {
		let event = RuntimeEvent::Agent(AgentEvent::Thinking { task: task.clone() });

		if let Err(error) = crate::agent::log::append_agent_event(&event) {
			eprintln!("Failed to write agent event log: {error:#}");
		}

		if let Err(error) = event_tx.send(event) {
			eprintln!("Failed to emit thinking event: {error}");
		}
	}
	fn guard_action(
		&self,
		action: &AgentAction,
		run: &AgentRun,
		guard: &mut AgentGuard,
	) -> Result<GuardDecision> {
		let command = match action {
			AgentAction::RunCommand { command } => Some(command.as_str()),
			AgentAction::RunProgram { program, args, .. } => match program.as_str() {
				"sh" | "bash" | "zsh" => args
					.iter()
					.position(|arg| arg == "-c")
					.and_then(|index| args.get(index + 1))
					.map(String::as_str),
				_ => None,
			},

			_ => None,
		};
		let Some(command) = command else {
			return Ok(GuardDecision::Allow);
		};
		let command = command.trim();
		if command.is_empty() {
			return Ok(GuardDecision::Reject("Command cannot be empty.".into()));
		}
		if command.len() > 10_000 {
			return Ok(GuardDecision::Reject(
				"Command exceeds the 10000-character limit. \
				Split unusually large commands into smaller operations."
					.into(),
			));
		}
		let count = guard
			.recent_commands
			.entry(command.to_string())
			.or_insert(0);
		*count += 1;
		if *count > 2 {
			return Ok(GuardDecision::Reject(
				"This exact command has already been proposed multiple times. \
				Inspect prior results and change your approach."
					.into(),
			));
		}

		// Compare against executed commands. Permit one retry, since a
		// command may legitimately need to be rerun after a transient failure.
		let previous_runs = run
			.commands
			.iter()
			.filter(|record| record.command.trim() == command)
			.count();

		if previous_runs >= 2 {
			return Ok(GuardDecision::Reject(
				"This exact command has already been executed twice. \
				Inspect its recorded output before attempting another approach."
					.into(),
			));
		}

		Ok(GuardDecision::Allow)
	}
	fn reject_action(&self, run: &mut AgentRun, step: usize, reason: String) {
		eprintln!("Rejecting agent action at step {step}: {reason}");

		run.record_error(AgentError {
			step: Some(step),
			kind: "action_guard".into(),
			message: reason.clone(),
			raw_response: None,
			recoverable: true,
			timestamp: chrono::Utc::now(),
		});

		run.ctx.history.push(AgentObservation::Current {
			message: format!(
				"ACTION REJECTED BEFORE EXECUTION\n\
			Reason: {reason}\n\n\
			No command was executed for this action. \
			Choose a different bounded action. Do not repeat \
			the rejected action unchanged."
			),
		});
	}
	fn handle_current(
		&self,
		task: &AgentTask,
		event_tx: &UnboundedSender<RuntimeEvent>,
		run: &mut AgentRun,
		message: String,
	) {
		let now = chrono::Local::now().format("%Y-%m-%d").to_string();
		let message = format!("Context update: The current date is {now}. {message}");

		let event = RuntimeEvent::Agent(AgentEvent::Working {
			task: task.clone(),
			message: message.clone(),
		});

		if let Err(error) = crate::agent::log::append_agent_event(&event) {
			eprintln!("Failed to write agent event log: {error:#}");
		}

		if let Err(error) = event_tx.send(event) {
			eprintln!("Failed to emit current event: {error}");
		}

		run.ctx.history.push(AgentObservation::Current { message });
	}
	fn handle_context(
		&self,
		task: &AgentTask,
		event_tx: &UnboundedSender<RuntimeEvent>,
		run: &mut AgentRun,
		path: Option<String>,
	) {
		let message = match path {
			Some(path) => format!(
				"Context requested for path: {path}. \
			Inspect it using a supported command, respecting the task's scope."
			),
			None => format!(
				"Context requested. Current history entries: {}. \
			Use supported commands to inspect only what the task authorizes.",
				run.ctx.history.len(),
			),
		};

		let event = RuntimeEvent::Agent(AgentEvent::Working {
			task: task.clone(),
			message: message.clone(),
		});

		if let Err(error) = crate::agent::log::append_agent_event(&event) {
			eprintln!("Failed to write agent event log: {error:#}");
		}

		if let Err(error) = event_tx.send(event) {
			eprintln!("Failed to emit context event: {error}");
		}

		run.ctx.history.push(AgentObservation::Current { message });
	}
	fn try_finish(
		&self,
		task_id: uuid::Uuid,
		run: &mut AgentRun,
		baseline: &WSSnapshot,
		step: usize,
		message: String,
	) -> Result<Option<TaskResult>> {
		let workspace_final = WSSnapshot::capture(&self.workspace.cwd)?;

		let final_changed_paths = reconcile_agent_artifacts(&mut run.ctx, baseline, &workspace_final)?;

		let validation = validate_agent_completion(
			&self.workspace.cwd,
			&run.ctx.artifacts,
			&run.commands,
			&final_changed_paths,
		)?;

		if !validation.is_valid() {
			let feedback = format!(
				"COMPLETION REJECTED: runtime validation failed.\n\
				You claimed the task was complete, but these checks failed:\n{}\n\n\
				Correct the actual files, then inspect and verify them. \
				Do not simply repeat the completion claim.",
				validation.errors().join("\n"),
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

			return Ok(None);
		}

		let final_message = format!(
			"{message}\n\nRuntime-observed changed files:\n{}",
			format_changed_files(&final_changed_paths),
		);

		Ok(Some(TaskResult::completed_with_summary(
			task_id,
			std::mem::replace(
				&mut run.ctx,
				AgentCtx::with_workspace(String::new(), (*self.workspace).clone()),
			),
			final_message,
		)))
	}
	fn emit_finished(&self, event_tx: &UnboundedSender<RuntimeEvent>, result: TaskResult) {
		let event = RuntimeEvent::Agent(AgentEvent::Finished { result });

		if let Err(error) = crate::agent::log::append_agent_event(&event) {
			eprintln!("Failed to write agent event log: {error:#}");
		}

		if let Err(error) = event_tx.send(event) {
			eprintln!("Failed to emit finished event: {error}");
		}
	}
	fn fail_step_limit(
		&self,
		task_id: uuid::Uuid,
		mut run: AgentRun,
		max_steps: usize,
	) -> TaskResult {
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

		TaskResult::failed(task_id, run.ctx, "Step limit exceeded", Some(message))
	}

	pub fn with_cwd(cwd: impl Into<PathBuf>) -> Self {
		Self::with_workspace(CtxWorkspace::from_cwd(cwd))
	}
	pub fn with_ctx(ctx: AgentCtx, _session: &AiSession) -> Result<Self> {
		Ok(Self {
			files_seen: 0,
			tokens_seen: 0,
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(ctx.workspace),
		})
	}
	pub fn with_workspace(workspace: CtxWorkspace) -> Self {
		Self {
			files_seen: 0,
			tokens_seen: 0,
			id: uuid::Uuid::new_v4().to_string(),
			tools: AgentTools::default(),
			workspace: Arc::new(workspace),
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
	async fn select_action(&self, run: &mut AgentRun, step: usize) -> Result<Option<AgentAction>> {
		const MAX_SELECTION_ERRORS: usize = 3;

		match self.pick_action(&run.ctx).await {
			Ok(action) => {
				// A valid response breaks the consecutive failure streak.
				Ok(Some(action))
			}
			Err(error) => {
				let message = format!("Action selection failed: {error:#}");
				eprintln!("{message}");

				let previous_failures = run
					.errors
					.iter()
					.rev()
					.take_while(|error| error.kind == "action_selection")
					.count();

				let consecutive_failures = previous_failures + 1;

				run.record_error(AgentError {
					step: Some(step),
					kind: "action_selection".into(),
					message: message.clone(),
					raw_response: None,
					recoverable: consecutive_failures < MAX_SELECTION_ERRORS,
					timestamp: chrono::Utc::now(),
				});

				if consecutive_failures >= MAX_SELECTION_ERRORS {
					return Ok(None);
				}

				run.ctx.history.push(AgentObservation::Current {
					message: format!(
						"INVALID ACTION RESPONSE\n\
					Reason: {message}\n\n\
					No command from this response was executed.\n\
					Return one valid JSON action using the supported schema. \
					For shell execution, use `run_command` with a non-empty \
					`command` field. For completion, use `finish` with a \
					`message` field. Do not repeat an invalid response."
					),
				});

				Ok(Some(AgentAction::Current {
					message: "Action selection failed; corrective feedback was added \
					to the history. Choose a valid next action."
						.into(),
				}))
			}
		}
	}
	async fn execute_shell_action(
		&self,
		run: &mut AgentRun,
		step: usize,
		command: String,
	) -> Result<()> {
		println!("COMMAND: {command}");

		let workspace_before = WSSnapshot::capture(&self.workspace.cwd)?;

		let mut shell_command = ShellCommand::shell(command.clone());
		shell_command.cwd = Some(self.workspace.cwd.clone());

		let result = match self.tools.shell.run(shell_command).await {
			Ok(result) => result,
			Err(error) => {
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
						"{message}\n\
						The shell tool did not return a completed result. \
						Inspect the error and observed changes before retrying."
					),
				});

				return Ok(());
			}
		};

		section!("SHELL RESULT");

		println!("command: {command}");
		println!("exit: {:?}", result.exit_code);

		println!(
			"stdout ({} chars, {} lines):\n{}",
			result.stdout.len(),
			result.stdout.lines().count(),
			preview_lines(&result.stdout, SHELL_OUTPUT_PREVIEW_LINES),
		);

		println!(
			"stderr ({} chars, {} lines):\n{}",
			result.stderr.len(),
			result.stderr.lines().count(),
			preview_lines(&result.stderr, SHELL_OUTPUT_PREVIEW_LINES),
		);

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
				Some(format!("Command exited with status {:?}", result.exit_code,))
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

		Ok(())
	}
	async fn execute_program_action(
		&self,
		run: &mut AgentRun,
		step: usize,
		program: String,
		args: Vec<String>,
		cwd: Option<String>,
	) -> Result<()> {
		let display_command = std::iter::once(program.as_str())
			.chain(args.iter().map(String::as_str))
			.collect::<Vec<_>>()
			.join(" ");

		println!("PROGRAM: {display_command}");
		let workspace_before = WSSnapshot::capture(&self.workspace.cwd)?;
		let mut shell_command = ShellCommand::program(program.clone(), args.clone());
		// The workspace root is authoritative. Do not trust an arbitrary
		// working directory supplied by the model.
		let _requested_cwd = cwd;
		shell_command.cwd = Some(self.workspace.cwd.clone());

		let result = match self.tools.shell.run(shell_command).await {
			Ok(result) => result,
			Err(error) => {
				let workspace_after = WSSnapshot::capture(&self.workspace.cwd)?;

				let changed_paths =
					reconcile_agent_artifacts(&mut run.ctx, &workspace_before, &workspace_after)?;

				let message = format!(
					"Program execution failed for `{display_command}`: {error:#}\n\
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
						"{message}\n\
						The program did not return a completed result. Inspect \
						the error and observed changes before choosing the next action."
					),
				});

				return Ok(());
			}
		};

		section!("PROGRAM RESULT");

		println!("program: {program}");
		println!("args: {args:?}");
		println!("exit: {:?}", result.exit_code);

		println!(
			"stdout ({} chars, {} lines):\n{}",
			result.stdout.len(),
			result.stdout.lines().count(),
			preview_lines(&result.stdout, SHELL_OUTPUT_PREVIEW_LINES),
		);

		println!(
			"stderr ({} chars, {} lines):\n{}",
			result.stderr.len(),
			result.stderr.lines().count(),
			preview_lines(&result.stderr, SHELL_OUTPUT_PREVIEW_LINES),
		);

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
			command: display_command.clone(),
			exit_code: result.exit_code,
			stdout: result.stdout.clone(),
			stderr: result.stderr.clone(),
			success,
			error: if success {
				None
			} else {
				Some(format!("Program exited with status {:?}", result.exit_code,))
			},
			timestamp: chrono::Utc::now(),
		});
		if !success {
			run.record_error(AgentError {
				step: Some(step),
				kind: "command_exit".into(),
				message: format!(
					"Program `{display_command}` exited with status {:?}.\n\
					Stderr:\n{}",
					result.exit_code,
					preview(&result.stderr, 1000),
				),
				raw_response: None,
				recoverable: true,
				timestamp: chrono::Utc::now(),
			});
		}
		run.ctx.history.push(AgentObservation::Current {
			message: format!(
				"Runtime filesystem reconciliation after program `{display_command}`:\n\
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
		Ok(())
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
		let mut run = AgentRun::new(AgentCtx::with_workspace(
			task.prompt.clone(),
			(*self.workspace).clone(),
		));

		let prompt = run.ctx.prompt.as_deref().unwrap_or("");

		section!("AGENT run_agent_loop CONTEXT");
		log_run(
			&mut run.clone(),
			format!(
				"RUN START: prompt_chars={}, prompt_lines={}, history={}, commands={}, errors={}, logs={}, tokens={}, files={}",
				prompt.len(),
				prompt.lines().count(),
				run.ctx.history.len(),
				run.commands.len(),
				run.errors.len(),
				run.ctx.logs.len(),
				self.tokens_seen,
				self.files_seen,
			),
		);

		log_run(
			&mut run.clone(),
			format!(
				"PROMPT PREVIEW:\n{}",
				preview_lines(prompt, PROMPT_PREVIEW_LINES),
			),
		);

		let context_log = format!(
			"Initial context: prompt_chars={}, prompt_lines={}, workspace={}",
			prompt.len(),
			prompt.lines().count(),
			run.ctx.workspace,
		);

		run.record_log(context_log);

		self.emit_thinking(&task, &event_tx);

		log_run(&mut run, "Selecting agent mode");

		let mode = match self.pick_mode(&run.ctx).await {
			Ok(mode) => {
				log_run(&mut run, format!("Mode selected: {mode:?}"));
				mode
			}
			Err(error) => {
				log_run(&mut run, format!("MODE SELECTION FAILED: {error:#}"));
				return Err(error);
			}
		};

		if matches!(mode, AgentMode::Chat) {
			log_run(&mut run, "Entering chat path");

			let response = match prompt_chat(&run.ctx).await {
				Ok(response) => response,
				Err(error) => {
					log_run(&mut run, format!("CHAT FAILED: {error:#}"));
					return Err(error);
				}
			};

			log_run(
				&mut run,
				format!("Chat completed: response_chars={}", response.len()),
			);

			return Ok(TaskResult::completed_chat(task.id, run.ctx, response));
		}

		log_run(
			&mut run,
			format!(
				"Capturing baseline workspace snapshot: {}",
				self.workspace.cwd.display()
			),
		);

		let baseline = match WSSnapshot::capture(&self.workspace.cwd) {
			Ok(snapshot) => {
				log_run(&mut run, "Baseline workspace snapshot captured");
				snapshot
			}
			Err(error) => {
				log_run(&mut run, format!("BASELINE SNAPSHOT FAILED: {error:#}"));
				return Err(error.into());
			}
		};

		let mut guard = AgentGuard::default();
		log_run(
			&mut run.clone(),
			format!("Starting agent loop: max_steps={max_steps}"),
		);

		for step in 1..=max_steps {
			log_run(
				&mut run.clone(),
				format!(
					"STEP {step}/{max_steps}: history={}, commands={}, errors={}, logs={}, tokens={}, files={}",
					run.clone().ctx.history.len(),
					run.clone().commands.len(),
					run.clone().errors.len(),
					run.clone().ctx.logs.len(),
					self.tokens_seen,
					self.files_seen,
				),
			);
			let action = match self.select_action(&mut run, step).await {
				Ok(Some(action)) => {
					log_run(
						&mut run,
						format!("STEP {step}: selected action: {action:?}"),
					);
					action
				}
				Ok(None) => {
					log_run(
						&mut run,
						format!("STEP {step}: action selection exhausted retries"),
					);

					return Ok(TaskResult::failed(
						task.id,
						run.ctx,
						"Action selection failed",
						Some("Too many consecutive invalid actions.".into()),
					));
				}
				Err(error) => {
					log_run(
						&mut run,
						format!("STEP {step}: ACTION SELECTION ERROR: {error:#}"),
					);
					return Err(error);
				}
			};
			match self.guard_action(&action, &run, &mut guard) {
				Ok(GuardDecision::Allow) => {
					log_run(&mut run, format!("STEP {step}: guard allowed action"));
				}
				Ok(GuardDecision::Reject(reason)) => {
					log_run(
						&mut run,
						format!("STEP {step}: GUARD REJECTED ACTION: {reason}"),
					);

					self.reject_action(&mut run, step, reason);
					continue;
				}
				Err(error) => {
					log_run(&mut run, format!("STEP {step}: GUARD ERROR: {error:#}"));
					return Err(error);
				}
			}

			match action {
				AgentAction::Current { message } => {
					log_run(
						&mut run,
						format!("STEP {step}: CURRENT message_chars={}", message.len()),
					);

					self.handle_current(&task, &event_tx, &mut run, message);

					log_run(&mut run, format!("STEP {step}: CURRENT handled"));
				}

				AgentAction::Context { path } => {
					log_run(&mut run, format!("STEP {step}: CONTEXT path={path:?}"));

					self.handle_context(&task, &event_tx, &mut run, path);

					log_run(&mut run, format!("STEP {step}: CONTEXT handled"));
				}

				AgentAction::RunCommand { command } => {
					log_run(
						&mut run,
						format!("STEP {step}: SHELL START command={command:?}"),
					);

					match self.execute_shell_action(&mut run, step, command).await {
						Ok(()) => {
							log_run(&mut run, format!("STEP {step}: SHELL HANDLER COMPLETE"));
						}
						Err(error) => {
							log_run(
								&mut run,
								format!("STEP {step}: SHELL HANDLER FAILED: {error:#}"),
							);
							return Err(error);
						}
					}
				}

				AgentAction::RunProgram { program, args, cwd } => {
					log_run(
						&mut run,
						format!("STEP {step}: PROGRAM START program={program:?} args={args:?} cwd={cwd:?}"),
					);

					match self
						.execute_program_action(&mut run, step, program, args, cwd)
						.await
					{
						Ok(()) => {
							log_run(&mut run, format!("STEP {step}: PROGRAM HANDLER COMPLETE"));
						}
						Err(error) => {
							log_run(
								&mut run,
								format!("STEP {step}: PROGRAM HANDLER FAILED: {error:#}"),
							);
							return Err(error);
						}
					}
				}

				AgentAction::Finish { message } => {
					log_run(
						&mut run,
						format!("STEP {step}: FINISH requested message={message:?}"),
					);

					match self.try_finish(task.id, &mut run, &baseline, step, message) {
						Ok(Some(result)) => {
							log_run(&mut run, format!("STEP {step}: FINISH VALIDATED"));
							self.emit_finished(&event_tx, result.clone());
							return Ok(result);
						}
						Ok(None) => {
							log_run(
								&mut run,
								format!("STEP {step}: FINISH REJECTED; continuing loop"),
							);
						}
						Err(error) => {
							log_run(
								&mut run,
								format!("STEP {step}: FINISH VALIDATION ERROR: {error:#}"),
							);
							return Err(error);
						}
					}
				}
			}
		}
		log_run(
			&mut run,
			format!("STEP LIMIT REACHED: max_steps={max_steps}"),
		);
		Ok(self.fail_step_limit(task.id, run, max_steps))
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

impl AgentCompletionValidation {
	pub fn is_valid(&self) -> bool {
		self.errors.is_empty()
	}

	pub fn errors(&self) -> &[String] {
		&self.errors
	}
}

impl TryFrom<LlmAction> for AgentAction {
	type Error = anyhow::Error;

	fn try_from(raw: LlmAction) -> Result<Self> {
		match raw.action.trim() {
			"run_command" => match (raw.command, raw.args) {
				(Some(command), None) if !command.trim().is_empty() => Ok(Self::RunCommand { command }),

				(None, Some(args)) if !args.is_empty() => {
					let mut args = args.into_iter();
					let program = args
						.next()
						.filter(|program| !program.trim().is_empty())
						.ok_or_else(|| {
							anyhow::anyhow!("run_command args must start with a non-empty program")
						})?;

					Ok(Self::RunProgram {
						program,
						args: args.collect(),
						cwd: raw.cwd,
					})
				}

				(Some(_), Some(_)) => {
					anyhow::bail!("ambiguous run_command: provide command or args, not both")
				}

				(Some(_), None) => {
					anyhow::bail!("run_command requires a non-empty command")
				}

				(None, Some(_)) => {
					anyhow::bail!("run_command args must not be empty")
				}

				(None, None) => {
					anyhow::bail!("run_command requires either command or args")
				}
			},

			"finish" => {
				let message = raw
					.message
					.filter(|message| !message.trim().is_empty())
					.ok_or_else(|| anyhow::anyhow!("finish requires a non-empty message"))?;

				Ok(Self::Finish { message })
			}

			action => {
				anyhow::bail!("unsupported action: {action:?}")
			}
		}
	}
}
#[derive(Debug, Clone)]
pub struct Agent {
	pub id: String,
	pub tools: AgentTools,
	pub workspace: Arc<CtxWorkspace>,
	pub tokens_seen: u128,
	pub files_seen: u128,
	// pub tokens_seen: u128,
}
#[derive(Clone, Debug)]
pub struct AgentBus {
	pub tx: UnboundedSender<AgentEvent>,
	pub event_tx: UnboundedSender<RuntimeEvent>,
}
pub struct AgentCompletionValidation {
	pub errors: Vec<String>,
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
#[derive(Debug, Default)]
pub struct AgentGuard {
	consecutive_no_progress: usize,
	recent_commands: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
	pub ctx: AgentCtx,
	pub commands: Vec<CommandRecord>,
	pub errors: Vec<AgentError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AgentTask {
	pub id: Uuid,
	pub prompt: String,
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
pub struct LlmAction {
	pub action: String,
	#[serde(default)]
	pub command: Option<String>,
	#[serde(default)]
	pub message: Option<String>,
	#[serde(default)]
	pub args: Option<Vec<String>>,
	#[serde(default)]
	pub cwd: Option<String>,
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

fn log_run(run: &mut AgentRun, message: impl Into<String>) {
	let message = message.into();
	let line = format!("[AGENT] {message}");

	println!("{line}");
	run.record_log(line);
}
