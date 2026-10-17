use super::*;

impl Pipeline {
	fn from_session(kontex: Kontex, session: AiSession) -> anyhow::Result<Self> {
		let qa = Evaluator {
			session: session.clone(),
			jev: TypeSafeClient::from_env()?,
		};
		let mut system = AgentSystem::new();
		system.cwd(session.workspace.clone());
		let generator = Box::new(LocalGenerator::new(system.runtime.clone(), DEFAULT_MODEL));
		Ok(Self {
			qa,

			system,
			kontex,
			generator,
			session,
			stage_attempt: 0,
		})
	}
	fn init_session_dir(kontex: &Kontex, title: &str) -> Result<PathBuf> {
		let sessions_dir = kontex.path(FW::Session)?;
		let dir = sessions_dir;
		FS::ensure_dir(&dir)?;
		Ok(dir)
	}
	fn init_templates(kontex: &Kontex, dir: &Path) -> Result<()> {
		let template_dir = kontex.path(FW::AiTemplates)?;
		for file in [
			SrcArtifact::Intent,
			SrcArtifact::Spec,
			SrcArtifact::Plan,
			SrcArtifact::Progress,
		] {
			let source = template_dir.join(file.name());
			let destination = file.path(dir);
			let contents = if FS::exists(&source) {
				FS::read(source)?
			} else {
				format!("# {}\n\n", file.name())
			};
			FS::write(destination, contents)?;
		}
		Ok(())
	}
	fn session_save(session: &AiSession) -> Result<()> {
		tracing::warn!("session_save");
		FS::save(SpecialFile::WriteCurrent.path()?, session)?;
		Ok(())
	}
	fn write(path: PathBuf, contents: &str) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}
	fn emit(&self, event: SdlcEvent) {
		// let _ = self.event_tx.send(event.clone());
		// let path = std::path::Path::new(SDLC_LOG_DIR).join(SDLC_LOG_FILE_NAME);
		// if let Ok(mut file) = std::fs::OpenOptions::new()
		// 	.create(true)
		// 	.append(true)
		// 	.open(path)
		// {
		// 	let _ = serde_json::to_writer(&mut file, &event);
		// 	let _ = writeln!(file);
		// }
	}
	fn evaluate_checks(&self, checks: &[CheckResult]) -> Vec<Metric> {
		checks
			.iter()
			.map(|check| Metric {
				name: check.name.clone(),
				passed: check.passed,
				score: if check.passed { 1.0 } else { 0.0 },
				confidence: 1.0,
				explanation: match &check.output {
					Some(output) if !output.is_empty() => output.clone(),
					_ => {
						if check.passed {
							"Check passed.".into()
						} else {
							"Check failed.".into()
						}
					}
				},
			})
			.collect()
	}
	fn is_passing(&self, checks: &[CheckResult], evaluations: &[Metric]) -> bool {
		checks.iter().all(|check| check.passed)
			&& evaluations.iter().all(|evaluation| evaluation.passed)
	}
	fn next_attempt(&mut self, stage: Stage) -> Result<Attempt> {
		let session = self.session()?;
		let max = DEFAULT_LOOP_TRIES;
		let number = session
			.stages
			.iter()
			.filter(|record| record.stage == stage)
			.map(|record| record.attempt.number)
			.max()
			.map_or(1, |number| number + 1);
		Ok(Attempt { stage, number, max })
	}

	fn persist(&self) -> Result<()> {
		self.kontex.session_save(&self.session)
	}
	fn persist_evaluation(&mut self, evaluation: &QACheck, attempt: Attempt) -> Result<()> {
		let session = self.session()?;
		persist_evaluation(session, evaluation, attempt)?;
		self.persist()
	}
	fn persist_outcome(&mut self, outcome: &AttemptOutcome) -> Result<()> {
		let (stage, attempt, status, actor, time_started, time_completed, description, evaluation) =
			match outcome {
				AttemptOutcome::Complete {
					execution,
					evaluation,
				} => (
					execution.stage,
					execution.attempt,
					Status::Completed,
					StageActor::Runner,
					execution.time_started,
					Some(execution.time_completed),
					format!("Stage Completed: {}", execution.stage),
					Some(evaluation.clone()),
				),
				AttemptOutcome::NeedsRevision {
					execution,
					evaluation,
				} => (
					execution.stage,
					execution.attempt,
					Status::NeedsRevision,
					StageActor::Evaluator,
					execution.time_started,
					Some(execution.time_completed),
					format!("Needs Revision: {}", execution.stage),
					Some(evaluation.clone()),
				),
				AttemptOutcome::ExecutionFailed {
					stage,
					attempt,
					error,
				} => (
					*stage,
					*attempt,
					Status::Failed,
					StageActor::Runner,
					Utc::now(),
					Some(Utc::now()),
					format!("{error:#}"),
					None,
				),
				AttemptOutcome::EvaluationFailed { execution, error } => (
					execution.stage,
					execution.attempt,
					Status::EvaluationFailed,
					StageActor::Runner,
					execution.time_started,
					Some(execution.time_completed),
					error.to_string(),
					None,
				),
			};
		let record = StageRunRecord {
			stage,
			attempt,
			status,
			actor,
			time_started,
			time_completed,
			description: Some(description),
			evaluation,
		};
		self.push_stage_record(record)
	}
	fn persist_progress(&mut self, message: &str) -> Result<()> {
		let session = self.session()?;
		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
		let entry = format!("\n## {timestamp}\n\n{message}\n");
		SrcArtifact::Progress.append(&session.dir, entry)?;
		Ok(())
	}
	fn persist_session(&mut self) -> Result<()> {
		tracing::warn!("persist_session");
		let session = self.session()?;
		Self::session_save(session)
	}

	fn push_stage_record(&mut self, record: StageRunRecord) -> Result<()> {
		let session = self.session()?;
		session.stages.push(record);
		session.time_updated = Utc::now();
		self.persist()
	}
	fn session(&mut self) -> Result<&mut AiSession> {
		Ok(&mut self.session)
	}
	fn session_read(&mut self, name: &str) -> Result<String> {
		read_from_session(name, self.session()?)
	}
	fn session_record(&mut self) -> Result<()> {
		let session = self.session()?;
		let index_path = SpecialFile::SessionsIndex.path()?;
		let mut sessions: Vec<AiSession> = FS::load(&index_path)?.unwrap_or_default();
		sessions.retain(|existing| existing.id != session.id);
		sessions.push(session.clone());
		FS::save(index_path, &sessions)?;
		Ok(())
	}
	fn retry(&mut self, _stage: Stage) -> Result<()> {
		Ok(())
	}
	fn stage_attempt(&self) -> u32 {
		self.stage_attempt
	}
	fn transition(&mut self, next: Stage) -> Result<()> {
		let session = self.session()?;
		let valid = matches!(
			(&session.stage, &next),
			(Stage::Intent, Stage::Spec)
				| (Stage::Spec, Stage::Plan)
				| (Stage::Plan, Stage::Build)
				| (Stage::Build, Stage::QA)
				| (Stage::QA, Stage::Build)
				| (Stage::QA, Stage::Complete)
		);

		if !valid {
			return Err(anyhow::anyhow!(
				"invalid SDLC transition: {:?} -> {:?}",
				session.stage,
				next
			));
		}

		session.stage = next;
		session.time_updated = Utc::now();

		self.persist()?;
		self.session_record()?;

		Ok(())
	}

	pub fn stage(&self) -> Stage {
		self.session.stage
	}
	pub fn stage_set(&mut self, stage: Stage) -> Result<()> {
		self.session()?.stage = stage;
		Ok(())
	}

	pub fn stage_attempt_increment(&mut self) -> u32 {
		self.stage_attempt += 1;
		self.stage_attempt
	}
	pub fn stage_attempt_reset(&mut self) {
		self.stage_attempt = 1;
	}

	pub async fn evaluate(&self, execution: &Execution) -> Result<QACheck> {
		self.qa.evaluate(execution).await
	}
	pub async fn resume(&mut self) -> Result<()> {
		self.persist_progress("SDLC session resumed")?;
		self.persist()?;
		Ok(())
	}
	pub async fn resume_from(&mut self, stage: Stage) -> Result<()> {
		tracing::info!("resume_from sprintpi pipeline");
		self.stage_set(stage)?;
		self.persist_progress(&format!("SDLC session resumed from {stage}"))?;
		self.persist()?;
		Ok(())
	}
	pub async fn run_task(&mut self, prompt: &str) -> Result<String> {
		self
			.system
			.runtime
			.run_agent(AgentTask::new(prompt.into()))
			.await
			.map(|result| {
				result
					.chat
					.or(result.summary)
					.unwrap_or_else(|| "Agent completed".to_string())
			})
	}
	pub async fn run_checks(&self, stage: Stage) -> Result<Vec<CheckResult>> {
		if !matches!(stage, Stage::Build | Stage::QA) {
			return Ok(Vec::new());
		}
		let plan_path = self.session.dir.join("plan.md");
		let plan = std::fs::read_to_string(&plan_path)
			.with_context(|| format!("reading build plan at {}", plan_path.display()))?;
		let expected_outputs = p::expected_outputs_from_plan(&plan)?;
		let mut results = Vec::with_capacity(expected_outputs.len());
		let workspace = self.session.workspace.canonicalize().with_context(|| {
			format!(
				"canonicalizing project workspace {}",
				self.session.workspace.display()
			)
		})?;
		for required_file in &expected_outputs {
			let candidate = workspace.join(required_file);
			let (passed, output) = match candidate.canonicalize() {
				Ok(path) if path.starts_with(&workspace) && path.is_file() => {
					match std::fs::metadata(&path) {
						Ok(metadata) if metadata.len() > 0 => (true, "File exists and is non-empty".into()),
						Ok(_) => (false, "File is empty".into()),
						Err(error) => (false, error.to_string()),
					}
				}
				Ok(_) => (
					false,
					"Path is not a file inside the project workspace".into(),
				),
				Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
					(false, "Required file is missing".into())
				}
				Err(error) => (false, error.to_string()),
			};
			results.push(CheckResult {
				name: format!("required file {}", required_file.display()),
				passed,
				output: Some(output),
			});
		}
		Ok(results)
	}
	pub async fn summarize_title(intent: &str) -> Result<String> {
		Ok(String::from("create-sdlc-pipeline"))
	}
	pub async fn wait_for_intervention(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self.emit(SdlcEvent::InterventionRequired {
			stage,
			attempt,
			reason,
		});
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::AwaitingHuman,
		});
		let input = input_rx
			.recv()
			.await
			.ok_or_else(|| anyhow!("SDLC input channel closed"))?;
		match input {
			SdlcInput::Human(string) => {
				self.emit(SdlcEvent::HumanInput {
					stage,
					input: string.clone(),
				});
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "human input provided".into(),
				});
				Ok(Intervention::Human(string))
			}
			SdlcInput::Retry => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "retry".into(),
				});
				Ok(Intervention::Retry)
			}
			SdlcInput::ProvideContext(context) => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "context provided".into(),
				});
				Ok(Intervention::ProvideContext(context))
			}
			SdlcInput::Reviewed => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "reviewed".into(),
				});
				Ok(Intervention::Reviewed)
			}
			SdlcInput::Abort => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "abort".into(),
				});
				Ok(Intervention::Abort)
			}
			SdlcInput::Revision { evaluation } => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "revision".into(),
				});
				Ok(Intervention::Revision { evaluation })
			}
		}
	}
	pub async fn init(&mut self, intent: impl Into<String>) -> Result<()> {
		let kontex = Kontex::new(special::Appp::Estate)?;
		let intent = intent.into();
		let title = Self::summarize_title(&intent).await?;
		let dir = Self::init_session_dir(&kontex, &title)?;
		Self::init_templates(&kontex, &dir)?;
		let session = AiSession::new(title, intent, dir)?;
		self.session = session;
		self.stage_attempt = 1;
		self.persist_session()?;
		Ok(())
	}
	pub async fn new(intent: impl Into<String>, _event_tx: EventBus) -> anyhow::Result<Self> {
		dotenvy::dotenv().ok();
		let kontex = Kontex::new(special::Appp::Estate)?;
		let intent = intent.into();
		let title = Self::summarize_title(&intent).await?;
		let dir = Self::init_session_dir(&kontex, &title)?;
		Self::init_templates(&kontex, &dir)?;
		let session = AiSession::new(title, intent, dir)?;
		kontex.session_save(&session)?;
		Self::from_session(kontex, session)
	}
	pub async fn resume_saved(_event_tx: EventBus) -> anyhow::Result<Self> {
		dotenvy::dotenv().ok();
		let kontex = Kontex::new(special::Appp::Estate)?;
		let mut session = kontex
			.session_load::<AiSession>()?
			.ok_or_else(|| anyhow::anyhow!("No saved SDLC session is available to resume"))?;
		if session.workspace == session.dir || !session.workspace.is_dir() {
			session.workspace = session.cwd.clone();
			kontex.session_save(&session)?;
		}
		Self::from_session(kontex, session)
	}
}

#[async_trait::async_trait]
impl<'a, Context: Ctx> sdlc_trait::RunnerTrait for PipelineRunner<'a, Context> {
	type Context = UnboundedReceiver<SdlcInput>;
	type Output = ();
	/// "What happens next?"
	/// Orchestrates the SDLC state machine:
	/// 1. run/execute stage -> Given the current stage, do business logic
	/// 2. persist outcome -> update the execution results, logs, state, etc
	/// 3. handle_outcome -> given the outcome decide what to do next.
	/// 4. apply -> given an outcome of execution handle business logic of that decision with run control.
	/// 5. control -> continue, retry, or exit. there is no other possibility.
	async fn run(&mut self, input_rx: &mut Self::Context) -> Result<Self::Output> {
		tracing::info!("sdlc_trait::RunnerTrait for PipelineRunner");
		let mut pending_input = None;
		let mut stage = self.runtime.pipeline.stage();
		self.emit(SdlcEvent::RunStarted);
		loop {
			let attempt = self.next_attempt(stage)?;
			section_info!(
				"run",
				metrics: [
				"stage" => stage,
				"attempt" => attempt,
				],
			);
			// 1. Execute the current stage.
			let outcome = self
				.execute(stage, attempt, &mut pending_input)
				.await
				.with_context(|| format!("run_stage({stage:?})"))?;
			// 2. Persist the execution and evaluation result.
			self.runtime.pipeline.persist_outcome(&outcome)?;

			// 3. Decide what should happen next.
			let decision = self.handle_outcome(&outcome).await?;

			// 4. Apply the decision.
			// apply() transitions to the next stage.
			let control = self
				.apply(outcome, decision, input_rx, &mut pending_input)
				.await?;

			match control {
				RunControl::Continue => {
					stage = self.runtime.pipeline.stage();
					// Every new stage starts at attempt 1.
					self.runtime.pipeline.stage_attempt_reset();
					tracing::info!(
						stage = ?stage,
						attempt = self.runtime.pipeline.stage_attempt(),
						">>> Runner continuing."
					);
				}
				RunControl::RetryStage => {
					continue;
				}
				RunControl::Exit(status) => {
					tracing::info!(?status, "RunnerTrait exiting");
					return Ok(());
				}
			}
		}
	}
}
impl<'a, Context: Ctx> PipelineRunner<'a, Context> {
	/// Run the entire pipeline from its current state.
	pub async fn run(
		&mut self,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<PipelineOutcome> {
		tracing::info!("Context: Ctx> PipelineRunner");
		let mut pending_input = None;
		let mut stage = self.runtime.pipeline.stage();
		let mut stage_outcomes: Vec<StageOutcome> = Vec::new();
		self.emit(SdlcEvent::RunStarted);
		loop {
			let attempt = self.next_attempt(stage)?;
			section_info!(
				"run",
				metrics: [
				"stage" => stage,
				"attempt" => attempt,
				],
			);
			// 1. Execute the current stage.
			tracing::info!(?stage, ?attempt, ">>> execute ENTER");

			let outcome = self
				.execute(stage, attempt, &mut pending_input)
				.await
				.with_context(|| format!("run_stage({stage:?})"))?;

			tracing::info!(?stage, "<<< execute EXIT");
			// 2. Persist the execution and evaluation result.
			self.runtime.pipeline.persist_outcome(&outcome)?;
			tracing::info!("persist_outcome");

			// 3. Decide what should happen next.
			let decision = self.handle_outcome(&outcome).await?;
			tracing::info!("handle_outcome");

			// 4. Apply the decision.
			// apply() transitions to the next stage.
			let control = self
				.apply(outcome, decision, input_rx, &mut pending_input)
				.await?;

			tracing::info!("apply");

			match control {
				RunControl::Continue => {
					tracing::info!("continue");
					stage = self.runtime.pipeline.stage();
					// Every new stage starts at attempt 1.
					self.runtime.pipeline.stage_attempt_reset();
					tracing::info!(
						stage = ?stage,
						attempt = self.runtime.pipeline.stage_attempt(),
						">>> Runner continuing."
					);
				}
				RunControl::RetryStage => {
					tracing::info!("RetryStage");
					continue;
				}
				RunControl::Exit(status) => {
					tracing::info!(?status, "Pipeline exiting");

					return Ok(PipelineOutcome {
						status,
						stages: stage_outcomes,
					});
				}
			}
		}
	}

	/// Resume execution from the saved state.
	pub async fn resume(&mut self) -> Result<PipelineOutcome> {
		todo!("run")
	}

	/// Run a specific stage, respecting its prerequisites.
	pub async fn run_stage(&mut self, stage: Stage) -> Result<StageOutcome> {
		todo!("run")
	}

	/// Run a stage while explicitly allowing existing upstream artifacts.
	pub async fn run_stage_with(
		&mut self,
		stage: Stage,
		options: StageRunOptions,
	) -> Result<Outcome> {
		todo!("run")
	}

	/// Evaluate an existing artifact without rerunning the stage.
	pub async fn evaluate_stage(&mut self, stage: Stage) -> Result<Evaluation> {
		todo!("run")
	}

	/// Prepare a stage without executing it.
	pub async fn prepare_stage(&mut self, stage: Stage) -> Result<()> {
		todo!("run")
	}

	/// Retry the selected stage using the existing retry policy.
	pub async fn retry_stage(&mut self, stage: Stage) -> Result<Outcome> {
		todo!("run")
	}

	/// Save the current pipeline state and stage checkpoint.
	pub async fn checkpoint(&mut self) -> Result<()> {
		todo!("run")
	}

	/// Execute exactly one attempt.
	async fn execute_attempt_with_input(
		&mut self,
		stage: Stage,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<AttemptOutcome> {
		let attempt = self.next_attempt(stage)?;

		section_info!(
				"execute_attempt",
				metrics: [
						"stage" => stage,
						"attempt" => attempt,
				],
		);

		let outcome = self
			.execute(stage, attempt, pending_input)
			.await
			.with_context(|| format!("execute_attempt({stage:?})"))?;

		self.runtime.pipeline.persist_outcome(&outcome)?;

		Ok(outcome)
	}
	/// Convenience API for executing one attempt.
	pub async fn execute_attempt(&mut self, stage: Stage) -> Result<AttemptOutcome> {
		// pub async fn execute_attempt(&mut self, stage: Stage) -> Result<AttemptOutcome> {
		// 1. Prepare the stage.
		// 2. Increment/create the attempt.
		// 3. Execute the stage.
		// 4. Evaluate the execution.
		// 5. Return the attempt outcome.
		let mut pending_input = None;

		self
			.execute_attempt_with_input(stage, &mut pending_input)
			.await
	}
}
impl<'a, Context: Ctx> PipelineRunner<'a, Context> {
	fn ensure_agent_completed(result: TaskResult, phase: &str) -> Result<TaskResult> {
		if matches!(&result.status, crate::TaskStatus::Completed) {
			return Ok(result);
		}
		let errors = result
			.errors
			.iter()
			.map(|error| format!("{}: {}", error.kind, error.message))
			.collect::<Vec<_>>()
			.join("\n");
		Err(anyhow!(
			"Build {phase} agent did not complete ({:?})\n{errors}",
			result.status
		))
	}
	fn next_attempt(&mut self, stage: Stage) -> Result<Attempt> {
		self.runtime.next_attempt(stage)
	}
	fn emit(&self, event: SdlcEvent) {
		self.runtime.emit(event);
	}
	fn evaluate_checks(&self, checks: Vec<CheckResult>) -> Result<Vec<CheckResult>> {
		todo!("evaluate_checks")
	}
	fn load_state(&mut self) -> Result<Stage> {
		let path = SpecialFile::WriteCurrent.path()?;
		tracing::info!(">>> load_state path = {:?}", path);
		tracing::info!(">>> exists = {}", path.exists());
		let json = std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
		tracing::info!(">>> loaded state = {}", json);
		#[derive(serde::Deserialize)]
		struct PersistedStage {
			stage: Stage,
		}
		let state: PersistedStage = serde_json::from_str(&json)?;
		let stage = match state.stage {
			e::Stage::QA => Stage::Build,
			stage => stage,
		};
		self.runtime.pipeline.session.stage = stage;
		Ok(stage)
	}
	fn persist(&mut self, msg: &str) -> Result<()> {
		self.runtime.pipeline.persist_progress(msg)?;
		Ok(())
	}
	fn retry(&mut self, stage: Stage) -> Result<()> {
		self.runtime.pipeline.retry(stage)
	}
	fn session(&mut self) -> &AiSession {
		&self.runtime.pipeline.session
	}
	fn stage_dir(&mut self) -> (Stage, PathBuf) {
		let session = &self.session();
		(session.stage, session.dir.clone())
	}
	fn write_dir(&mut self) -> PathBuf {
		let session = &self.session();
		session.dir.clone()
	}
	fn transition(&mut self, next: Stage) -> Result<()> {
		self.runtime.pipeline.transition(next)
	}

	/// "Given this decision, what actions/state changes must happen?"
	///
	/// Applies the decision to the pipeline/session and returns control
	/// to the outer run loop.

	async fn apply(
		&mut self,
		outcome: AttemptOutcome,
		decision: Decision,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		let stage = outcome.stage();
		let attempt = outcome.attempt();

		match decision {
			Decision::Continue => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;
				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});

				Ok(RunControl::Continue)
			}

			Decision::Retry => {
				if attempt.number >= attempt.max {
					tracing::warn!(
						stage = ?stage,
						attempt = attempt.number,
						max = attempt.max,
						"maximum stage attempts reached"
					);

					self.emit(SdlcEvent::Failed {
						stage: Some(stage),
						error: format!(
							"Stage {stage:?} exhausted maximum attempts ({})",
							attempt.max
						),
					});

					self.runtime.pipeline.persist_progress(&format!(
						"{} exhausted maximum attempts ({})",
						stage, attempt.max
					))?;

					return Ok(RunControl::Exit(PipelineStatus::Failed));
				}

				let next_attempt = attempt.number + 1;

				self.runtime.pipeline.persist_progress(&format!(
					"Retrying stage {} (attempt {}/{})",
					stage, next_attempt, attempt.max,
				))?;

				self.runtime.pipeline.stage_attempt_increment();
				self.handle_retry(stage, attempt).await?;

				Ok(RunControl::RetryStage)
			}

			Decision::Revise => {
				self
					.runtime
					.pipeline
					.persist_progress(&format!("Revising stage {}", stage))?;

				// TODO: Restore revision handling here if this branch is
				// intended to perform the revision directly.
				Ok(RunControl::RetryStage)
			}

			Decision::AwaitHuman => {
				self.emit(SdlcEvent::PhaseChanged {
					stage,
					phase: Phase::AwaitingHuman,
				});

				let intervention = self
					.wait_for_intervention(
						stage,
						attempt,
						format!("Stage {stage:?} requires human intervention"),
						input_rx,
					)
					.await?;

				match intervention {
					Intervention::Retry => {
						if attempt.number >= attempt.max {
							return Ok(RunControl::Exit(PipelineStatus::Failed));
						}

						self.runtime.pipeline.stage_attempt_increment();
						self.handle_retry(stage, attempt).await?;

						Ok(RunControl::RetryStage)
					}

					Intervention::Revision { evaluation } => {
						*pending_input = Some(SdlcInput::Revision { evaluation });
						self.runtime.pipeline.retry(stage)?;

						Ok(RunControl::RetryStage)
					}

					Intervention::Revise => {
						self
							.handle_revision(stage, attempt, outcome, input_rx, pending_input)
							.await?;

						Ok(RunControl::RetryStage)
					}

					Intervention::Reviewed => Ok(RunControl::RetryStage),

					Intervention::ProvideContext(context) => {
						*pending_input = None;

						self
							.runtime
							.pipeline
							.persist_progress(&format!("Human provided context: {context}"))?;

						Ok(RunControl::RetryStage)
					}

					Intervention::Abort => {
						self.emit(SdlcEvent::Failed {
							stage: Some(stage),
							error: "aborted by user".into(),
						});

						Ok(RunControl::Exit(PipelineStatus::Interrupted))
					}

					Intervention::Human(_) => Ok(RunControl::Exit(PipelineStatus::Interrupted)),
				}
			}

			Decision::Fail => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: format!("Stage {stage:?} failed on attempt {}", attempt.number),
				});

				self
					.runtime
					.pipeline
					.persist_progress(&format!("{} failed", stage))?;

				Ok(RunControl::Exit(PipelineStatus::Failed))
			}

			Decision::Exit => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: format!("Stage {stage:?} exited on attempt {}", attempt.number),
				});

				Ok(RunControl::Exit(PipelineStatus::Interrupted))
			}
			Decision::Complete => {
				self.emit(SdlcEvent::PhaseChanged {
					stage,
					phase: Phase::Completed,
				});
				self
					.runtime
					.pipeline
					.persist_progress(&format!("{} completed", stage))?;
				self.emit(SdlcEvent::Completed);
				Ok(RunControl::Exit(PipelineStatus::Complete))
			}
		}
	}
	async fn await_human(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::AwaitingHuman,
		});
		self.emit(SdlcEvent::Activity {
			stage,
			attempt,
			message: "Waiting for human input".into(),
		});
		let input = input_rx.recv().await.context("SDLC input channel closed")?;
		match input {
			SdlcInput::Retry => {
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			SdlcInput::Reviewed => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;
				self.transition(next)?;
				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
				Ok(RunControl::Continue)
			}

			SdlcInput::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input));
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			SdlcInput::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			SdlcInput::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			SdlcInput::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit(PipelineStatus::Interrupted))
			}
		}
	}
	async fn evaluate(&self, execution: &Execution) -> Result<QACheck> {
		tracing::info!("evaluate pipeline runner {:?}", execution);
		let stage = execution.stage;
		let checks = self.runtime.pipeline.run_checks(stage).await?;
		let structural = self.evaluate_checks(checks)?;
		let semantic = self.runtime.pipeline.qa.evaluate(&execution).await?;
		Ok(semantic.with_evaluations(structural))
	}
	async fn execute(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<AttemptOutcome> {
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::Executing,
		});
		let input = match pending_input.take() {
			Some(SdlcInput::Revision { evaluation }) => StageInput::Revision { evaluation },
			Some(input) => {
				*pending_input = Some(input);
				StageInput::Initial
			}
			None => StageInput::Initial,
		};
		tracing::info!(?stage, ?attempt, ">>> route_execution ENTER");

		let execution = match self.route_execution(stage, attempt, input).await {
			Ok(execution) => execution,
			Err(error) => {
				return Ok(AttemptOutcome::ExecutionFailed {
					stage,
					attempt,
					error,
				});
			}
		};
		self.handle_evaluation(execution, attempt).await
	}

	async fn handle_outcome(&self, outcome: &AttemptOutcome) -> Result<Decision> {
		if std::env::var_os("SDLC_FORCE_EXIT").is_some() {
			section_info!(
				"decision",
				metrics: [
					"override" => "SDLC_FORCE_EXIT",
					"decision" => "Exit",
				],
				"" => "Forced exit",
			);
			return Ok(Decision::Exit);
		}

		if std::env::var_os("SDLC_FORCE_CONTINUE").is_some() {
			section_info!(
				"decision",
				metrics: [
					"override" => "SDLC_FORCE_CONTINUE",
					"decision" => "Continue",
				],
				"" => "Forced continue",
			);
			return Ok(Decision::Continue);
		}

		let (decision, stage, attempt, max_attempts, outcome_name, details) = match outcome {
			AttemptOutcome::EvaluationFailed { execution, .. } => {
				let attempt = &execution.attempt;
				let decision = if attempt.number < attempt.max {
					Decision::Retry
				} else {
					Decision::AwaitHuman
				};

				(
					decision,
					attempt.stage,
					attempt.number,
					attempt.max,
					"EvaluationFailed",
					"Evaluation failed".to_string(),
				)
			}

			AttemptOutcome::Complete { execution, .. } => {
				let decision = match &execution.result {
					RunResult::Verification(verification) if verification.passed => Decision::Continue,
					RunResult::Verification(_) => Decision::Retry,
					_ if execution.stage == Stage::Complete => Decision::Complete,
					_ => Decision::Continue,
				};

				let details = match &execution.result {
					RunResult::Verification(verification) => {
						format!("Verification passed={}", verification.passed)
					}
					_ => "Execution completed".to_string(),
				};

				(
					decision,
					execution.stage,
					execution.attempt.number,
					execution.attempt.max,
					"Complete",
					details,
				)
			}

			AttemptOutcome::NeedsRevision {
				execution,
				evaluation,
			} => {
				let attempt = &execution.attempt;
				let decision = if attempt.number < attempt.max {
					Decision::Revise
				} else {
					Decision::AwaitHuman
				};

				(
					decision,
					execution.stage,
					attempt.number,
					attempt.max,
					"NeedsRevision",
					format!(
						"Revision required\nscore => {:.2}\nconfidence => {:.2}",
						evaluation.score, evaluation.confidence,
					),
				)
			}

			AttemptOutcome::ExecutionFailed { attempt, error, .. } => {
				let decision = if attempt.number < attempt.max {
					Decision::Retry
				} else {
					Decision::AwaitHuman
				};

				(
					decision,
					attempt.stage,
					attempt.number,
					attempt.max,
					"ExecutionFailed",
					format!("Execution error => {error}\nError debug => {error:?}"),
				)
			}
		};

		section_info!(
			"SDLC decision",
			metrics: [
				"outcome" => outcome_name,
				"stage" => format!("{stage:?}"),
				"attempt" => format!("{attempt}/{max_attempts}"),
				"decision" => format!("{decision:?}"),
			],
			"details" => details,
		);

		Ok(decision)
	}
	async fn handle_retry(&mut self, stage: Stage, attempt: Attempt) -> Result<()> {
		let next_attempt = Attempt {
			stage,
			number: attempt.number + 1,
			max: attempt.max,
		};
		self.emit(SdlcEvent::StageRetrying {
			stage,
			number: next_attempt.number,
		});
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::Retrying,
		});
		self.runtime.pipeline.retry(stage)?;
		Ok(())
	}
	async fn handle_evaluation(
		&mut self,
		execution: Execution,
		attempt: Attempt,
	) -> Result<AttemptOutcome> {
		let stage = execution.stage;

		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::Evaluating,
		});

		self.emit(SdlcEvent::EvaluationStarted { stage });

		tracing::info!("handle_evaluation {:?}", execution);

		match self.runtime.pipeline.evaluate(&execution).await {
			Ok(evaluation) => {
				self.emit(SdlcEvent::Evaluated {
					stage,
					score: evaluation.score,
					confidence: evaluation.confidence,
					passed: evaluation.passed,
				});

				self
					.runtime
					.pipeline
					.persist_evaluation(&evaluation, attempt);

				if evaluation.passed {
					Ok(AttemptOutcome::Complete {
						execution,
						evaluation,
					})
				} else {
					Ok(AttemptOutcome::NeedsRevision {
						execution,
						evaluation,
					})
				}
			}

			Err(error) => Ok(AttemptOutcome::EvaluationFailed { execution, error }),
		}
	}
	async fn handle_revision(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		_outcome: AttemptOutcome,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<()> {
		match self
			.wait_for_intervention(
				stage,
				attempt,
				String::from("Stage needs revision"),
				input_rx,
			)
			.await?
		{
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input));
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::Retry => {
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::Reviewed => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;
				self.transition(next)?;
				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
			}
			Intervention::Revise => {
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});
			}
			Intervention::Reviewed => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;
				self.transition(next)?;
				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
			}
			Intervention::Revise => {
				self.runtime.pipeline.retry(stage)?;
			}
			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});
			}
		}
		Ok(())
	}
	async fn handle_failure_execution(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::ExecutionFailed {
			stage,
			attempt,
			error: error.to_string(),
		});
		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});
			self.emit(SdlcEvent::PhaseChanged {
				stage,
				phase: Phase::Retrying,
			});
			self.runtime.pipeline.retry(stage)?;
			return Ok(RunControl::Continue);
		}
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::AwaitingHuman,
		});
		match self
			.wait_for_intervention(stage, attempt, String::from("Execution Failure"), input_rx)
			.await?
		{
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.as_str().to_owned()));
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: error.to_string(),
				});

				Ok(RunControl::Exit(PipelineStatus::Interrupted))
			}
			Intervention::Revise => {
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
		}
	}
	async fn handle_failure_evaluation(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::EvaluationFailed {
			stage,
			error: error.to_string(),
		});
		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				stage,
				phase: Phase::Retrying,
			});

			self.runtime.pipeline.retry(stage)?;
			return Ok(RunControl::Continue);
		}
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::AwaitingHuman,
		});
		match self
			.wait_for_intervention(stage, attempt, String::from("Evaluation Failure"), input_rx)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Retry => {
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::ProvideContext(context) => {
				// If ProvideContext eventually becomes stage input,
				// this is where it should be stored.
				let _ = context;
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Reviewed => {
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Abort => {
				self.emit(SdlcEvent::PhaseChanged {
					stage,
					phase: Phase::Failed,
				});

				self
					.runtime
					.pipeline
					.persist_progress("Pipeline interrupted: aborted by user")?;

				Ok(RunControl::Exit(PipelineStatus::Interrupted))
			}
			Intervention::Revise => {
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.runtime.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
		}
	}
	async fn handle_failure_of_quality(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				stage,
				phase: Phase::Retrying,
			});

			self.runtime.pipeline.retry(stage)?;

			return Ok(RunControl::Continue);
		}
		self.emit(SdlcEvent::PhaseChanged {
			stage,
			phase: Phase::AwaitingHuman,
		});
		match self
			.wait_for_intervention(
				stage,
				attempt,
				String::from("Quality Error (Needs Revision)"),
				input_rx,
			)
			.await?
		{
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });

				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				// "Reviewed" means the human explicitly accepts
				// the current artifact despite JEV not passing it.
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});

				Ok(RunControl::Continue)
			}
			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user: quality requirements not met".into(),
				});

				Ok(RunControl::Exit(PipelineStatus::NeedsRevision))
			}
			Intervention::Revise => {
				self.runtime.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
		}
	}

	async fn on_intent(&mut self, input: StageInput) -> Result<RunResult> {
		let (stage, session_dir) = self.stage_dir();
		let goal = self.session().prompt.clone();

		tracing::info!(
			stage = ?stage,
			session_dir = %session_dir.display(),
			goal_len = goal.len(),
			"Intent: start"
		);

		if stage != Stage::Intent {
			return Err(anyhow!("cannot execute Intent stage while at {stage:?}"));
		}

		let prompt = p::gen_intent(&goal)?;
		tracing::info!(prompt_len = prompt.len(), "Intent: prompt generated");

		std::fs::write(sdlc_log_file("prompt.intent.md"), &prompt)
			.context("Intent: writing prompt debug file")?;
		tracing::info!("Intent: prompt debug file written");

		let generated = self
			.runtime
			.pipeline
			.system
			.runtime
			.generate(&prompt)
			.await
			.context("Intent: generating artifact")?;
		tracing::info!(
			generated_len = generated.len(),
			"Intent: generation returned"
		);

		if generated.trim().is_empty() {
			return Err(anyhow!("generated Intent artifact is empty"));
		}

		self
			.write(&generated)
			.context("Intent: writing generated artifact")?;
		tracing::info!("Intent: artifact written");

		self
			.runtime
			.pipeline
			.persist_progress("Intent stage completed")
			.context("Intent: persisting progress")?;
		tracing::info!("Intent: progress persisted");

		Ok(RunResult::Intent)
	}
	async fn on_spec(&mut self, input: StageInput) -> Result<RunResult> {
		let (stage, _session_dir) = self.stage_dir();
		if stage != Stage::Spec {
			return Err(anyhow!("cannot execute Spec stage while at {stage:?}"));
		}
		let intent = self.runtime.pipeline.session_read("intent.md")?;
		let is_revision = matches!(input, StageInput::Revision { .. });
		let prompt = match input {
			StageInput::Initial => p::gen_spec(&intent)?,
			StageInput::Revision { evaluation } => {
				let spec = self.runtime.pipeline.session_read("spec.md")?;
				if spec.trim().is_empty() {
					return Err(anyhow!("Cannot revise Spec because spec.md is empty"));
				}
				p::revise_spec(&intent, &spec, &evaluation)?
			}
		};
		if prompt.trim().is_empty() {
			return Err(anyhow!("generated Spec prompt is empty"));
		}
		std::fs::write(sdlc_log_file("prompt.spec.md"), &prompt)
			.context("writing Spec prompt debug file")?;
		let generated = self
			.runtime
			.pipeline
			.system
			.runtime
			.generate(&prompt)
			.await
			.context("generating Spec artifact from Intent and prompt")?;
		if generated.trim().is_empty() {
			return Err(anyhow!("generated Spec artifact is empty"));
		}
		self.write(&generated)?;
		self.runtime.pipeline.persist_progress(if is_revision {
			"Spec revision completed"
		} else {
			"Spec stage completed"
		})?;
		Ok(RunResult::Spec)
	}
	async fn on_plan(&mut self, input: StageInput) -> Result<RunResult> {
		tracing::info!("on_plan: ENTER");

		let (stage, session_dir) = self.stage_dir();
		tracing::info!(?stage, ?session_dir, "on_plan: stage_dir");

		if stage != Stage::Plan {
			return Err(anyhow!("cannot execute Plan stage while at {stage:?}"));
		}

		let intent = self.runtime.pipeline.session_read("intent.md")?;
		let spec = self.runtime.pipeline.session_read("spec.md")?;
		tracing::info!(
			intent_len = intent.len(),
			spec_len = spec.len(),
			"on_plan: inputs loaded"
		);
		let prompt = p::gen_plan(&intent, &spec)?;
		tracing::info!(prompt_len = prompt.len(), "on_plan: prompt generated");
		if prompt.trim().is_empty() {
			return Err(anyhow!("Generated Plan prompt is empty"));
		}
		std::fs::write(sdlc_log_file("prompt.plan.md"), &prompt)
			.context("writing plan prompt debug file")?;
		tracing::info!("on_plan: runtime.generate ENTER");
		let generated = self
			.runtime
			.pipeline
			.system
			.runtime
			.generate(&prompt)
			.await
			.context("generating Plan artifact from Intent and Spec")?;

		tracing::info!(
			generated_len = generated.len(),
			"on_plan: runtime.generate EXIT"
		);
		// Keep the rest of your existing on_plan implementation.
		tracing::info!(
			generated_len = prompt.len(),
			has_inventory = prompt.contains("Machine-Readable Output Inventory"),
			has_build_contract = prompt.contains("Machine-Readable Build Contract"),
			generated_start = %prompt.chars().take(300).collect::<String>(),
			"Plan: inspect prompt artifact"
		);
		let generated = self
			.runtime
			.pipeline
			.system
			.runtime
			.generate(&prompt)
			.await
			.context("generating Plan artifact from Intent and Spec")?;
		if generated.trim().is_empty() {
			return Err(anyhow!("generated Plan artifact is empty"));
		}
		tracing::info!(
			generated_len = generated.len(),
				has_inventory = generated.contains("Machine-Readable Output Inventory"),
				has_build_contract = generated.contains("Machine-Readable Build Contract"),
				generated_start = %generated.chars().take(300).collect::<String>(),
				"Plan: inspect generated artifact"
		);
		std::fs::write(sdlc_log_file("plan.md"), &generated)
			.context("writing generated Plan debug file")?;
		let expected_outputs = p::expected_outputs_from_plan(&generated)
			.context("generated Plan has no valid expected-output inventory")?;
		if expected_outputs.is_empty() {
			return Err(anyhow!("generated Plan has no expected output files"));
		}
		self
			.write(&generated)
			.context("writing generated Plan to the SDLC session")?;
		self
			.runtime
			.pipeline
			.persist_progress("Plan stage completed")?;
		Ok(RunResult::Plan)
	}
	// SDLC_FORCE_CONTINUE=1 SDLC_PLAIN=1 cargo run --bin sdlc --features sdlc
	// SDLC_RESUME_BUILD=1 SDLC_PLAIN=1 cargo run --bin sdlc --features sdlc
	async fn on_build(&mut self, _input: StageInput) -> Result<RunResult> {
		let (stage, session_dir) = self.stage_dir();
		if stage != Stage::Build {
			return Err(anyhow::anyhow!(
				"cannot execute Build stage while at {:?}",
				stage
			));
		}

		self.runtime.pipeline.persist_progress("Build started")?;

		let workspace = self.runtime.pipeline.session.workspace_owned();

		// Load and validate the approved SDLC artifacts once.
		let intent_path = session_dir.join("intent.md");
		let spec_path = session_dir.join("spec.md");
		let plan_path = session_dir.join("plan.md");

		let intent = tokio::fs::read_to_string(&intent_path)
			.await
			.context("reading intent.md")?;
		let spec = tokio::fs::read_to_string(&spec_path)
			.await
			.context("reading spec.md")?;
		let plan = tokio::fs::read_to_string(&plan_path)
			.await
			.context("reading plan.md")?;

		if intent.trim().is_empty() {
			return Err(anyhow::anyhow!("Intent artifact is empty"));
		}
		if spec.trim().is_empty() {
			return Err(anyhow::anyhow!("Spec artifact is empty"));
		}
		if plan.trim().is_empty() {
			return Err(anyhow::anyhow!("Plan artifact is empty"));
		}

		self.runtime.pipeline.system.add_file(intent_path);
		self.runtime.pipeline.system.add_file(spec_path);
		self.runtime.pipeline.system.add_file(plan_path);

		// Capture the baseline before any Build phase modifies the workspace.
		let workspace_before = WSSnapshot::capture(&workspace)?;
		let git_status = workspace_before.git_status.trim();

		let mut values = Vec::new();
		if !git_status.is_empty() {
			for line in git_status.lines() {
				values.push(("changed", line.to_string()));
			}
		}

		section_info!(
			"workspace",
			metrics: [
				"files" => self.runtime.pipeline.system.runtime.workspace.files.len(),
				"git" => if git_status.is_empty() { "clean" } else { "dirty" },
			],
			"" => values.iter()
				.map(|(_, value)| value.as_str())
				.collect::<Vec<_>>()
				.join("\n"),
		);

		// Phase 1: Orient.
		let orient_report = self
			.on_build_orient(&session_dir, &intent, &spec, &plan)
			.await?;

		// Phase 2: Generate.
		let execute_report = self
			.on_build_generate(
				&session_dir,
				&workspace,
				&workspace_before,
				&intent,
				&spec,
				&plan,
				&orient_report,
			)
			.await?;

		// Phase 3: Verify.
		let _verify_report = self
			.on_build_verify(
				&session_dir,
				&workspace,
				&workspace_before,
				&intent,
				&spec,
				&plan,
				&orient_report,
				&execute_report,
			)
			.await?;

		// Capture final workspace state after all three phases.
		section!("BUILD SNAPSHOT START");

		let workspace_after =
			WSSnapshot::capture(&workspace).context("capturing workspace after Build")?;
		let changes = workspace_before.diff(&workspace_after);

		section!(&format!(
			"BUILD SNAPSHOT COMPLETE: files_changed={}",
			changes.file_count()
		));

		self.write(&changes.to_markdown("Build completed"))?;

		self.runtime.pipeline.persist_progress(&format!(
			"Build phases completed; {} file(s) changed",
			changes.file_count()
		))?;

		section!("BUILD STAGE COMPLETE");

		Ok(RunResult::Build)
	}

	async fn on_build_orient(
		&mut self,
		session_dir: &std::path::Path,
		intent: &str,
		spec: &str,
		plan: &str,
	) -> Result<String> {
		let orient_task = AgentTask::new(format!(
			"{PROMPT_BUILD_ORIENT}\n\n\
		 # Supplied SDLC Artifacts\n\n\
		 The following sections contain the artifact contents already loaded \
		 by the orchestrator. Analyze these contents directly; do not reread \
		 the corresponding files from disk.\n\n\
		 ## Intent (intent.md)\n\n{intent}\n\n\
		 ## Specification (spec.md)\n\n{spec}\n\n\
		 ## Plan (plan.md)\n\n{plan}"
		));

		let phase_started = std::time::Instant::now();
		section!("BUILD PHASE START: Orient");

		let result = match self
			.runtime
			.pipeline
			.system
			.runtime
			.run_agent(orient_task)
			.await
		{
			Ok(result) => Self::ensure_agent_completed(result, "Orient")?,
			Err(error) => {
				section_info!(
					"BUILD PHASE FAILED",
					metrics: [
						"phase" => "Orient",
						"elapsed" => format!("{:?}", phase_started.elapsed()),
					],
					"error" => format!("{error:#}"),
				);

				return Err(error).context("Build Orient phase failed");
			}
		};

		section!(&format!(
			"BUILD PHASE COMPLETE: Orient elapsed={:?}",
			phase_started.elapsed()
		));

		let report = format!("{result:#?}");

		tokio::fs::write(session_dir.join("build-orient.md"), &report)
			.await
			.context("writing Build Orient report")?;

		Ok(report)
	}

	async fn on_build_generate(
		&mut self,
		session_dir: &std::path::Path,
		workspace: &std::path::Path,
		workspace_before: &WSSnapshot,
		intent: &str,
		spec: &str,
		plan: &str,
		orient_report: &str,
	) -> Result<String> {
		let expected_outputs = p::expected_outputs_from_plan(plan)?;

		let execute_task = AgentTask::new(format!(
			"{PROMPT_BUILD_EXECUTE}\n\n\
		 ## Intent\n\n{intent}\n\n\
		 ## Specification\n\n{spec}\n\n\
		 ## Plan\n\n{plan}\n\n\
		 ## Orientation Report\n\n{orient_report}"
		))
		.with_expected_outputs(expected_outputs)
		.with_recovery_prompt(PROMPT_PHASE_RECOVER);

		let phase_started = std::time::Instant::now();
		section!("BUILD PHASE START: Generate");

		let result = match self
			.runtime
			.pipeline
			.system
			.runtime
			.run_agent(execute_task)
			.await
		{
			Ok(result) => Self::ensure_agent_completed(result, "Generate")?,
			Err(error) => {
				section_info!(
					"BUILD PHASE FAILED",
					metrics: [
						"phase" => "Generate",
						"elapsed" => format!("{:?}", phase_started.elapsed()),
					],
					"error" => format!("{error:#}"),
				);

				// Preserve partial work even when generation fails.
				match WSSnapshot::capture(workspace) {
					Ok(snapshot) => {
						let changes = workspace_before.diff(&snapshot);
						section_info!(
							"BUILD PARTIAL WORK",
							metrics: [
								"phase" => "Generate",
								"files changed" => changes.file_count(),
							],
						);

						let _ = self.write(&changes.to_markdown("Build failed during Generate"));
					}
					Err(snapshot_error) => {
						section_info!(
							"BUILD SNAPSHOT FAILED",
							metrics: [
								"phase" => "Generate",
							],
							"error" => format!("{snapshot_error:#}"),
						);
					}
				}

				return Err(error).context("Build Generate phase failed");
			}
		};

		section!(&format!(
			"BUILD PHASE COMPLETE: Generate elapsed={:?}",
			phase_started.elapsed()
		));

		let report = format!("{result:#?}");

		tokio::fs::write(session_dir.join("build-execute.md"), &report)
			.await
			.context("writing Build Generate report")?;

		Ok(report)
	}

	async fn on_build_verify(
		&mut self,
		session_dir: &std::path::Path,
		workspace: &std::path::Path,
		workspace_before: &WSSnapshot,
		intent: &str,
		spec: &str,
		plan: &str,
		orient_report: &str,
		execute_report: &str,
	) -> Result<String> {
		let verify_task = AgentTask::new(format!(
			"{PROMPT_BUILD_VERIFY}\n\n\
		 ## Intent\n\n{intent}\n\n\
		 ## Specification\n\n{spec}\n\n\
		 ## Plan\n\n{plan}\n\n\
		 ## Orientation Report\n\n{orient_report}\n\n\
		 ## Implementation Report\n\n{execute_report}"
		));

		let phase_started = std::time::Instant::now();
		section!("BUILD PHASE START: Verify");

		let result = match self
			.runtime
			.pipeline
			.system
			.runtime
			.run_agent(verify_task)
			.await
		{
			Ok(result) => Self::ensure_agent_completed(result, "Verify")?,
			Err(error) => {
				section_info!(
					"BUILD PHASE FAILED",
					metrics: [
						"phase" => "Verify",
						"elapsed" => format!("{:?}", phase_started.elapsed()),
					],
					"error" => format!("{error:#}"),
				);

				// Generation may have modified files before verification failed.
				match WSSnapshot::capture(workspace) {
					Ok(snapshot) => {
						let changes = workspace_before.diff(&snapshot);

						section_info!(
							"BUILD PARTIAL WORK",
							metrics: [
								"phase" => "Verify",
								"files changed" => changes.file_count(),
							],
						);

						let _ = self.write(&changes.to_markdown("Build failed during Verify"));
					}
					Err(snapshot_error) => {
						section_info!(
							"BUILD SNAPSHOT FAILED",
							metrics: [
								"phase" => "Verify",
							],
							"error" => format!("{snapshot_error:#}"),
						);
					}
				}

				return Err(error).context("Build Verify phase failed");
			}
		};

		section!(&format!(
			"BUILD PHASE COMPLETE: Verify elapsed={:?}",
			phase_started.elapsed()
		));

		let report = format!("{result:#?}");

		tokio::fs::write(session_dir.join("build-verify.md"), &report)
			.await
			.context("writing Build Verify report")?;

		Ok(report)
	}

	async fn on_qa(&mut self, input: StageInput) -> Result<RunResult> {
		let verification = self.run_qa_checks(self.runtime.pipeline.stage()).await?;
		self.runtime.pipeline.persist_progress(&format!(
			"Verification completed: passed={}",
			verification.passed
		))?;
		Ok(RunResult::Verification(verification))
	}

	async fn route_execution(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		input: StageInput,
	) -> Result<Execution> {
		let time_started = Utc::now();
		tracing::info!(?stage, "route_execution: dispatching");

		let result = match stage {
			Stage::Intent => self.on_intent(input).await?,
			Stage::Spec => self.on_spec(input).await?,
			Stage::Plan => self.on_plan(input).await?,
			Stage::Build => match self.on_build(input).await {
				Ok(result) => result,
				Err(error) => {
					eprintln!("Build stage failed: {error:#}");
					return Err(error);
				}
			},
			Stage::QA => self.on_qa(input).await?,
			Stage::Complete => RunResult::Complete,
			Stage::Finalize => RunResult::Finalize,
			Stage::Deploy | Stage::Maintain => {
				return Err(anyhow!("stage {stage:?} not implemented"));
			}
			_ => {
				todo!("run_stage")
			}
		};

		Ok(Execution {
			stage,
			attempt,
			time_started,
			time_completed: Utc::now(),
			result,
		})
	}
	async fn run_qa_checks(&mut self, stage: Stage) -> Result<Verification> {
		tracing::info!("run_qa_checks");
		let checks = self.runtime.pipeline.run_checks(stage).await?;
		let evaluations = self.runtime.pipeline.evaluate_checks(&checks);
		let passed = self.runtime.pipeline.is_passing(&checks, &evaluations);
		Ok(Verification {
			passed,
			checks,
			evaluations,
		})
	}

	async fn stage_deploy(&mut self) -> Result<()> {
		todo!("sdlc deploy")
	}
	async fn stage_maintain(&mut self) -> Result<()> {
		todo!("sdlc maintain")
	}
	async fn stage_complete(&mut self) -> Result<RunControl> {
		todo!("WOW DONE!")
	}

	async fn wait_for_intervention(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self
			.runtime
			.pipeline
			.wait_for_intervention(stage, attempt, reason, input_rx)
			.await
	}
	fn write(&mut self, content: &str) -> Result<()> {
		let (stage, session_dir) = self.stage_dir();
		let filename = format!("{stage:?}").to_lowercase() + ".md";
		let output_path = session_dir.join(&filename);

		tracing::info!(
				stage = ?stage,
				session_dir = %session_dir.display(),
				filename = %filename,
				output_path = %output_path.display(),
				content_len = content.len(),
				"SDLC artifact write"
		);

		Pipeline::write(output_path, content).context("writing generated SDLC artifact")?;

		Ok(())
	}
}

impl<C: Ctx> PipelineRuntime<C> {
	fn retry(&mut self, _stage: Stage) -> Result<()> {
		Ok(())
	}
	fn next_attempt(&mut self, stage: Stage) -> Result<Attempt> {
		self.pipeline.next_attempt(stage)
	}
	fn emit(&self, event: SdlcEvent) {
		self.pipeline.emit(event)
	}
	pub fn new(pipeline: Pipeline, runtime: Arc<NativeRuntime<C>>) -> Self {
		let event_bus = EventBus::new();
		let stage = pipeline.stage().clone();
		Self {
			stage,
			pipeline,
			runtime,
			event_bus,
			activity: vec![],
			attempt: 0,
			confidence: None,
			history: Vec::new(),
			message: None,
			passed: None,
			phase: Phase::Starting,
			score: None,
			stage_time_started: Instant::now(),
			time_started: Instant::now(),
			total_agent_calls: 0,
			total_tokens: 0,
			error: None,
			event_history: vec![],
		}
	}
	pub fn view(&self) -> SSPipelineRuntime {
		SSPipelineRuntime {
			activity: self.activity.clone(),
			attempt: self.attempt,
			stage: self.stage,
			time_started: self.time_started,
			stage_time_started: self.stage_time_started,
			phase: self.phase,
			score: self.score,
			confidence: self.confidence,
			passed: self.passed,
			message: self.message.clone(),
			error: self.error.clone(),
			total_tokens: self.total_tokens,
			total_agent_calls: self.total_agent_calls,
			history: self.history.clone(),
			events: self.event_history.clone(),
		}
	}
	pub async fn run(
		&mut self,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<PipelineOutcome> {
		tracing::info!("PipelineRuntime<C>");
		let mut runner = PipelineRunner { runtime: self };
		runner.run(input_rx).await.context("PipelineRunner::run")
	}
	pub async fn run_simulated(
		&mut self,
		_input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<()> {
		self.pipeline.emit(SdlcEvent::RunStarted);
		for (index, step) in Step::ALL.iter().enumerate() {
			if *step == Step::Complete {
				break;
			}
			let Some(stage) = step.stage() else {
				continue;
			};
			let attempt = self.pipeline.next_attempt(stage)?;
			self
				.pipeline
				.emit(SdlcEvent::StageStarted { stage, attempt });
			self.pipeline.emit(SdlcEvent::Activity {
				stage,
				attempt,
				message: format!("Dry Run · step {}/{}", index + 1, Step::ALL.len(),),
			});
			self.pipeline.emit(SdlcEvent::PhaseChanged {
				stage,
				phase: Phase::Executing,
			});
			sleep(DEMO_EXECUTION_TIME).await;
			self.pipeline.emit(SdlcEvent::ExecutionComplete { stage });
			self.pipeline.emit(SdlcEvent::PhaseChanged {
				stage,
				phase: Phase::Evaluating,
			});
			self.pipeline.emit(SdlcEvent::EvaluationStarted { stage });
			sleep(DEMO_EVALUATION_TIME).await;
			self.pipeline.emit(SdlcEvent::Evaluated {
				stage,
				score: 0.91,
				confidence: 0.94,
				passed: true,
			});
			let Some(next) = stage.next() else {
				break;
			};
			self.pipeline.transition(next)?;
			log_step_transition(stage, next)?;
			self.pipeline.emit(SdlcEvent::StageTransitioned {
				from: stage,
				to: next,
			});
		}
		self.pipeline.emit(SdlcEvent::Completed);
		sleep(DEMO_COMPLETE_DELAY).await;
		Ok(())
	}
	pub async fn resume_from(
		&mut self,
		stage: Stage,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<PipelineOutcome> {
		tracing::info!("resume_from pipeline runtime");
		self.pipeline.resume_from(stage).await?;
		let mut runner = PipelineRunner { runtime: self };
		runner.run(input_rx).await.context("PipelineRunner::run")
	}
	pub async fn run_stage(&mut self, stage: Stage) -> Result<StageOutcome> {
		let mut runner = PipelineRunner { runtime: self };
		runner
			.run_stage(stage)
			.await
			.context("PipelineRuntime::run_stage")
	}
	pub async fn execute_attempt(&mut self, stage: Stage) -> Result<AttemptOutcome> {
		let mut runner: PipelineRunner<'_, C> = PipelineRunner { runtime: self };
		runner
			.execute_attempt(stage)
			.await
			.context("PipelineRuntime::execute_attempt")
	}
}

#[derive(Debug)]
pub struct Pipeline {
	pub kontex: Kontex,
	pub qa: Evaluator,
	pub system: AgentSystem,
	pub generator: Box<dyn GeneratorTrait>,
	pub session: AiSession,
	pub stage_attempt: u32,
}
pub struct PipelineRunner<'a, Context: Ctx> {
	pub runtime: &'a mut PipelineRuntime<Context>,
}
#[derive(Debug)]
pub struct PipelineRuntime<Context: Ctx> {
	pub activity: Vec<String>,
	pub attempt: u32,
	pub confidence: Option<f64>,
	pub error: Option<String>,
	pub event_history: Vec<SdlcEvent>,
	pub event_bus: EventBus,
	pub history: Vec<SdlcEvent>,
	pub message: Option<String>,
	pub passed: Option<bool>,
	pub phase: Phase,
	pub pipeline: Pipeline,
	pub runtime: Arc<NativeRuntime<Context>>,
	pub score: Option<f64>,
	pub stage: Stage,
	pub stage_time_started: Instant,
	pub time_started: Instant,
	pub total_agent_calls: u32,
	pub total_tokens: u64,
}

pub struct PipelineState<S> {
	stage: S,
	attempt: u32,
}

pub struct StageRunOptions {
	pub artifact_policy: ArtifactPolicy,
	pub evaluate_after_run: bool,
	pub checkpoint_after_run: bool,
}

pub enum ArtifactPolicy {
	/// Require valid upstream artifacts before executing.
	RequireValid,

	/// Explicitly allow existing artifacts, even if QA previously rejected them.
	AcceptExisting,

	/// Regenerate missing or invalid upstream artifacts as needed.
	Regenerate,
}

impl Default for StageRunOptions {
	fn default() -> Self {
		Self {
			artifact_policy: ArtifactPolicy::RequireValid,
			evaluate_after_run: true,
			checkpoint_after_run: true,
		}
	}
}
