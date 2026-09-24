use crate::{
	model::{
		AgentTask,
		agent::{Agent, AgentContext},
		resolver::*,
		task::TaskResult,
	},
	prelude::*,
};
use anyhow::{Context, anyhow};
use crossterm::{
	cursor,
	event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use egui_plot::Corner;
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};

use std::{io::Stdout, process::Command};
use tokio::time::{Duration, sleep};

// cmd+alt+f
// - Search in all files overlay
// cmd+shift+f
// - Search in all files tab
// cmd+f
// - Search in file

// fn from_session(session: &SdlcSession) -> String {
// 	session.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?
// }
fn format_elapsed(duration: Duration) -> String {
	let total_seconds = duration.as_secs();
	let hours = total_seconds / 3600;
	let minutes = (total_seconds % 3600) / 60;
	let seconds = total_seconds % 60;

	if hours > 0 {
		format!("{hours}h {minutes}m {seconds}s")
	} else if minutes > 0 {
		format!("{minutes}m {seconds}s")
	} else {
		format!("{seconds}s")
	}
}
async fn monitor<F, T>(
	stage: Stage,
	attempt: u32,
	run_started: Instant,
	phase: &'static str,
	future: F,
) -> T
where
	F: Future<Output = T>,
{
	let started = Instant::now();
	tokio::pin!(future);
	let mut ticker = tokio::time::interval(STATUS_INTERVAL);
	loop {
		tokio::select! {
			result = &mut future => {
				return result;
			}
			_ = ticker.tick() => {
			}
		}
	}
}

const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(3);
const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(2);
const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(5);
const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
const MAX_STAGE_ATTEMPTS: u32 = 3;
const STATUS_INTERVAL: Duration = Duration::from_secs(30);

#[async_trait::async_trait]
trait ArtifactGenerator: Send + Sync {
	async fn generate(&self, prompt: &str) -> Result<String>;
}
trait TextModel {
	async fn generate(&self, prompt: &str) -> Result<String>;
}

#[async_trait]
impl ArtifactGenerator for LocalGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		let task = AgentTask::new(prompt.to_string());
		let (event_tx, _) = tokio::sync::mpsc::unbounded_channel();

		let result = self.agent.run_agent_loop(task, event_tx).await?;

		println!("=== GENERATOR RESULT ===");
		println!("status: {:?}", result.status);
		println!("chat: {:?}", result.chat);
		println!("summary: {:?}", result.summary);
		println!("artifacts: {:?}", result.artifacts);
		println!("logs: {:?}", result.logs);
		println!("========================");
		Ok(
			result
				.chat
				.or(result.summary)
				.unwrap_or_else(|| "Agent completed".to_string()),
		)
	}
}
#[async_trait]
impl ArtifactGenerator for ApiGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		todo!("apigenerator generate")
	}
}
impl Evaluator {
	async fn evaluate_intent(&self, intent: &str) -> Result<StageEvaluation> {
		let started_at = Utc::now();
		let response = self
			.jev
			.system_one(
				intent,
				[
					(
						"quality",
						Question::from(Score::new(
							"How well does this intent define a concrete software task?",
							[
								"Unusable: the desired outcome is unclear or not actionable",
								"Weak: some intent is present, but major ambiguity remains",
								"Usable: the intended outcome is understandable with some ambiguity",
								"Strong: the desired outcome is concrete and actionable",
								"Excellent: the desired outcome is precise, bounded, and directly actionable",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does this intent provide a sufficiently clear and concrete \
							 desired outcome for an implementation agent to act on \
							 without inventing major requirements?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no intent quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no intent decision"))?;

		Ok(StageEvaluation {
			stage: Stage::Intent,
			actor: StageActor::Human,
			started_at,
			score: quality.score,
			confidence: quality.confidence,
			passed: meets_bar.noul >= 0.80 && quality.confidence >= 0.70,
		})
	}
	async fn evaluate_spec(&self, intent: &str, spec: &str) -> Result<StageEvaluation> {
		let started_at = Utc::now();

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
			 ## Specification\n\n{spec}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How faithfully does the specification translate the intent \
							 into concrete, testable requirements?",
							[
								"Unusable: requirements are missing, contradictory, or unrelated",
								"Weak: substantial requirements are missing or invented",
								"Usable: the main intent is represented but some details are weak",
								"Strong: requirements are concrete, relevant, and testable",
								"Excellent: requirements comprehensively and precisely capture the intent \
								 without inventing unnecessary scope",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the specification faithfully represent the user's intent \
							 and provide sufficiently concrete requirements for planning \
							 and verification?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no spec quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no spec decision"))?;

		Ok(StageEvaluation {
			stage: Stage::Spec,
			actor: StageActor::Sdlc,
			started_at,
			score: quality.score,
			confidence: quality.confidence,
			passed: meets_bar.noul >= 0.80 && quality.confidence >= 0.70,
		})
	}
	async fn evaluate_plan(
		&self,
		intent: &str,
		spec: &str,
		plan: &str,
		tests: &str,
	) -> Result<StageEvaluation> {
		let started_at = Utc::now();
		let state = format!(
			"## User Intent\n\n{intent}\n\n\
			 ## Specification\n\n{spec}\n\n\
			 ## Implementation Plan\n\n{plan}\n\n\
			 ## Test Plan\n\n{tests}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How well does the implementation and test plan cover \
							 the specification?",
							[
								"Unusable: the plan does not provide a viable path to implementation",
								"Weak: major requirements or verification steps are uncovered",
								"Usable: the main implementation and verification work is covered",
								"Strong: requirements map clearly to implementation and verification steps",
								"Excellent: the plan is complete, ordered, dependency-aware, and provides \
								 explicit verification coverage for every requirement",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the implementation plan provide a concrete path from the \
							 specification to implementation, while ensuring that every \
							 requirement has corresponding verification coverage?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no plan quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no plan decision"))?;

		Ok(StageEvaluation {
			stage: Stage::Plan,
			actor: StageActor::Agent,
			started_at,
			score: quality.score,
			confidence: quality.confidence,
			passed: meets_bar.noul >= 0.80 && quality.confidence >= 0.70,
		})
	}
	async fn evaluate_build(
		&self,
		session: &Path,
		intent: &str,
		spec: &str,
		plan: &str,
		tests: &str,
	) -> Result<StageEvaluation> {
		let started_at = Utc::now();

		let implementation = std::fs::read_dir(session)?
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| {
				let path = entry.path();
				let name = path.file_name()?.to_string_lossy().into_owned();

				Some(if path.is_dir() {
					format!("[directory] {name}")
				} else {
					format!("[file] {name}")
				})
			})
			.collect::<Vec<_>>()
			.join("\n");

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
		 ## Specification\n\n{spec}\n\n\
		 ## Implementation Plan\n\n{plan}\n\n\
		 ## Test Plan\n\n{tests}\n\n\
		 ## Repository Artifacts\n\n{implementation}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How faithfully does the implemented work satisfy the \
						 specification and implementation plan?",
							[
								"Unusable: the implementation does not meaningfully address the task",
								"Weak: substantial requirements are missing or the implementation \
							 diverges from the plan",
								"Usable: the primary requirements appear implemented but some \
							 gaps or deviations remain",
								"Strong: the implementation closely follows the specification \
							 and provides the planned functionality",
								"Excellent: the implementation comprehensively satisfies the \
							 specification and plan with no significant unexplained gaps",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the implementation appear to satisfy the specified requirements \
						 and provide the functionality described by the implementation plan?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no build quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no build decision"))?;

		Ok(StageEvaluation {
			stage: Stage::Build,
			actor: StageActor::Agent,
			started_at,
			score: quality.score,
			confidence: quality.confidence,
			passed: meets_bar.noul >= 0.80 && quality.confidence >= 0.70,
		})
	}
	async fn evaluate_verification(&self, session: &Path) -> Result<StageEvaluation> {
		let started_at = Utc::now();
		let intent = SessionFile::Intent.read(&session)?;
		let spec = SessionFile::Spec.read(&session)?;
		let plan = SessionFile::Plan.read(&session)?;
		let tests = SessionFile::Tests.read(&session)?;
		let tests = SessionFile::Tests.read(&session)?;
		let evidence = SessionFile::Verification
			.read(&session)
			.unwrap_or_else(|_| String::from("No verification evidence was recorded."));

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
				## Specification\n\n{spec}\n\n\
				## Test Plan\n\n{tests}\n\n\
				## Verification Evidence\n\n{evidence}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How strong is the verification evidence for establishing \
						 that the implementation satisfies the specification?",
							[
								"Unusable: there is no meaningful verification evidence",
								"Weak: some checks exist but important requirements are unverified",
								"Usable: the primary requirements have verification evidence but \
							 some gaps remain",
								"Strong: deterministic and behavioral evidence covers the \
							 requirements with only minor gaps",
								"Excellent: verification provides comprehensive, concrete evidence \
							 for every requirement and clearly establishes the intended behavior",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the verification evidence provide sufficient evidence that \
						 every requirement in the specification has been satisfied?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no verification quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no verification decision"))?;

		Ok(StageEvaluation {
			stage: Stage::Verify,
			actor: StageActor::Sdlc,
			started_at,
			score: quality.score,
			confidence: quality.confidence,
			passed: meets_bar.noul >= 0.80 && quality.confidence >= 0.70,
		})
	}
}
impl Sdlc {
	// Lifecycle methods:
	// This needs to be focused because its easy to break... cause infintite loops
	//
	async fn complete_stage(&mut self) -> Result<RunControl> {
		todo!("complete_stage")
	}
	async fn execute_stage(
		&mut self,
		stage: Stage,
		pending_input: &mut Option<String>,
	) -> Result<()> {
		match stage {
			Stage::Intent => self.stage_intent().await,
			Stage::Spec => self.stage_spec().await,
			Stage::Plan => {
				let input = pending_input.take();
				self.stage_plan(input.as_deref()).await
			}
			Stage::Build => self.stage_build().await,
			Stage::Verify => self.verify().await.map(|_| ()),
			Stage::Complete => Ok(()),
			Stage::Deploy | Stage::Maintain => Err(anyhow!("stage {stage:?} not implemented")),
		}
	}
	async fn evaluate_stage_outcome(&mut self, stage: Stage) -> Result<StageOutcome> {
		todo!("evaluate_stage_outcome");
	}
	fn exit_no_stage(&mut self) -> Result<()> {
		todo!("exit_no_stage")
	}
	async fn handle_failure_execution(
		&mut self,
		stage: Stage,
		attempt: u32,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<String>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::ExecutionFailed {
			stage,
			attempt,
			error: error.to_string(),
		});

		if attempt < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				attempt: attempt + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});

			self.retry(stage)?;
			return Ok(RunControl::Continue);
		}

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(stage, attempt, String::from("Execution Failure"), input_rx)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(input);
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: error.to_string(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_failure_evaluation(
		&mut self,
		stage: Stage,
		attempt: u32,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<String>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::EvaluationFailed {
			stage,
			error: error.to_string(),
		});

		if attempt < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				attempt: attempt + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});

			self.retry(stage)?;
			return Ok(RunControl::Continue);
		}

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(stage, attempt, String::from("Evaluation Failure"), input_rx)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(input);
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				// If ProvideContext eventually becomes stage input,
				// this is where it should be stored.
				let _ = context;

				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_failure_of_quality(
		&mut self,
		stage: Stage,
		attempt: u32,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<String>,
	) -> Result<RunControl> {
		if attempt < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				attempt: attempt + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});

			self.retry(stage)?;

			return Ok(RunControl::Continue);
		}

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
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
			Intervention::Human(input) => {
				*pending_input = Some(input);
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.retry(stage)?;

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
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}

	pub fn init() -> Result<Option<Self>> {
		dotenvy::dotenv().ok();
		let jev = TypeSafeClient::from_env().context("creating TypeSafe client")?;
		let evaluator = Evaluator { jev };
		let (event_tx, _) = broadcast::channel(256);
		let session = SpecialFile::SdlcCurrent.load::<SdlcSession>()?;
		Ok(Some(Self {
			evaluator,
			event_tx,
			generator: Box::new(LocalGenerator {
				agent: Agent::new(),
			}),
			stage_attempt: 0,
			state_path: SpecialFile::SdlcCurrent.path()?,
			session,
		}))
	}

	pub async fn run(
		&mut self,
		mut input_rx: tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
	) -> Result<()> {
		let mut pending_input = None;
		let mut last_stage = None;
		let mut attempt = 0;
		self.emit(SdlcEvent::RunStarted);
		loop {
			let stage = match self.stage() {
				Some(stage) => stage,
				None => return self.exit_no_stage(),
			};
			attempt = self.next_attempt(stage, &mut last_stage, &mut attempt);

			self.emit(SdlcEvent::StageStarted { stage, attempt });
			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Executing,
			});
			// ------------------------------------------------------------
			// EXECUTION
			// Run the current stage.
			// ------------------------------------------------------------
			let execution = self.execute_stage(stage, &mut pending_input).await;
			// ------------------------------------------------------------
			// OUTCOME
			// Evaluate successful execution and classify failures.
			// ------------------------------------------------------------
			let outcome = match execution {
				Ok(()) => {
					self.emit(SdlcEvent::ExecutionComplete { stage });
					self.evaluate_stage_outcome(stage).await?
				}
				Err(error) => {
					self.emit(SdlcEvent::ExecutionFailed {
						stage,
						attempt,
						error: error.to_string(),
					});

					StageOutcome::ExecutionFailed(error)
				}
			};
			self.record_stage_outcome(&outcome, attempt)?;
			// ------------------------------------------------------------
			// ORCHESTRATION
			// Decide what happens after the stage outcome.
			// ------------------------------------------------------------
			let control = match outcome {
				StageOutcome::Complete { result, evaluation } => {
					let next = stage
						.next()
						.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;
					self.transition(next)?;
					self.emit(SdlcEvent::StageTransitioned {
						from: stage,
						to: next,
					});
					RunControl::Continue
				}
				StageOutcome::NeedsRevision { result, evaluation } => {
					self
						.handle_failure_of_quality(stage, attempt, &mut input_rx, &mut pending_input)
						.await?
				}
				StageOutcome::ExecutionFailed(error) => {
					self
						.handle_failure_execution(stage, attempt, error, &mut input_rx, &mut pending_input)
						.await?
				}
				StageOutcome::EvaluationFailed(error) => {
					self
						.handle_failure_evaluation(stage, attempt, error, &mut input_rx, &mut pending_input)
						.await?
				}
			};
			// ------------------------------------------------------------
			// Follow up
			// Given stage outcome what comes next?
			// ------------------------------------------------------------
			match control {
				RunControl::Continue => continue,
				RunControl::Exit => return Ok(()),
			}
		}
	}

	pub async fn run_simulated(
		&mut self,
		_input_rx: tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
	) -> Result<()> {
		loop {
			self.emit(SdlcEvent::RunStarted);
			for stage in [
				Stage::Intent,
				Stage::Spec,
				Stage::Plan,
				Stage::Build,
				Stage::Verify,
			] {
				let attempt = if stage == Stage::Plan { 2 } else { 1 };
				self.emit(SdlcEvent::StageStarted { stage, attempt: 1 });
				self.emit(SdlcEvent::PhaseChanged {
					phase: SdlcPhase::Executing,
				});
				sleep(DEMO_EXECUTION_TIME).await;
				if stage == Stage::Plan {
					self.emit(SdlcEvent::ExecutionFailed {
						stage,
						attempt: 1,
						error: "Simulated execution failure".into(),
					});
					self.emit(SdlcEvent::StageRetrying { stage, attempt });
					self.emit(SdlcEvent::PhaseChanged {
						phase: SdlcPhase::Retrying,
					});
					sleep(DEMO_RETRY_DELAY).await;
					self.emit(SdlcEvent::StageStarted { stage, attempt });
					self.emit(SdlcEvent::PhaseChanged {
						phase: SdlcPhase::Executing,
					});
					sleep(DEMO_EXECUTION_TIME).await;
				}
				self.emit(SdlcEvent::ExecutionComplete { stage });
				self.emit(SdlcEvent::PhaseChanged {
					phase: SdlcPhase::Evaluating,
				});
				self.emit(SdlcEvent::EvaluationStarted { stage });
				sleep(DEMO_EVALUATION_TIME).await;
				self.emit(SdlcEvent::Evaluated {
					stage,
					score: 0.91,
					confidence: 0.94,
					passed: true,
				});
				let next = stage.next().ok_or_else(|| anyhow!("no next stage"))?;
				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
			}
			self.emit(SdlcEvent::StageStarted {
				stage: Stage::Complete,
				attempt: 1,
			});
			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Completed,
			});
			self.emit(SdlcEvent::Completed);
			sleep(DEMO_COMPLETE_DELAY).await;
		}
	}
	pub async fn start(&mut self, intent: String) -> Result<()> {
		if self.session.is_some() {
			return Err(anyhow::anyhow!("an SDLC session is already active"));
		}
		let id = uuid::Uuid::new_v4().to_string();
		let title = intent
			.split_whitespace()
			.take(8)
			.collect::<Vec<_>>()
			.join("-")
			.to_lowercase();
		let dir = self.create_dir(&title)?;
		let now = Utc::now();
		let session = SdlcSession {
			id,
			title,
			stage: Stage::Intent,
			stages: Vec::new(),
			dir,
			created_at: now,
			updated_at: now,
		};
		self.session = Some(session);
		self.init_templates(
			self
				.session()?
				.ok_or_else(|| anyhow::anyhow!("failed to create session"))?
				.dir
				.as_path(),
		)?;
		let intent_path = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active session"))?
			.dir
			.join("intent.md");
		Self::write(intent_path, format!("# Intent\n\n{}\n", intent));
		self.update_progress("SDLC session started")?;
		self.persist()?;
		Ok(())
	}

	async fn wait_for_intervention(
		&mut self,
		stage: Stage,
		attempt: u32,
		reason: String,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self.emit(SdlcEvent::InterventionRequired {
			stage,
			attempt,
			reason,
		});
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
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
		}
	}
}
impl Sdlc {
	fn commit(&mut self) -> Result<()> {
		Ok(())
	}
	fn create_dir(&self, title: &str) -> Result<PathBuf> {
		let sessions_dir = FS::ensure_dir(SpecialFile::SessionsDir.path()?)?;
		let date = Local::now().format("%Y-%m-%d");
		let dir = sessions_dir.join(format!("{date}.{title}"));
		FS::ensure_dir(&dir)?;
		Ok(dir)
	}
	fn dir(&self) -> Result<&Path> {
		self
			.session
			.as_ref()
			.map(|session| session.dir.as_path())
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))
	}
	fn init_templates(&self, dir: &Path) -> Result<()> {
		let template_dir = SpecialFile::AiTemplateDir.path()?;
		for file in [
			SessionFile::Intent,
			SessionFile::Spec,
			SessionFile::Plan,
			SessionFile::Progress,
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
	fn is_passing(&self, checks: &[CheckResult], evaluations: &[EvaluationResult]) -> bool {
		checks.iter().all(|check| check.passed)
			&& evaluations.iter().all(|evaluation| evaluation.passed)
	}
	fn emit(&self, event: SdlcEvent) {
		let _ = self.event_tx.send(event);
	}
	async fn generate_plan(&self, intent: &str, spec: &str) -> Result<String> {
		let prompt = prompt::plan_gen(intent, spec);
		self.generator.generate(&prompt).await
	}
	async fn generate_tests(&self, intent: &str, spec: &str, plan: &str) -> Result<String> {
		let prompt = prompt::tests_gen(intent, spec, plan);
		self.generator.generate(&prompt).await
	}

	async fn evaluate_stage(&self, stage: Stage) -> Result<StageEvaluation> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let started_at = Utc::now();
		let evaluation = match stage {
			Stage::Intent => {
				let intent = self.read_session("intent.md")?;
				self.evaluator.evaluate_intent(&intent).await?
			}
			Stage::Spec => {
				let intent = self.read_session("intent.md")?;
				let spec = self.read_session("spec.md")?;
				self.evaluator.evaluate_spec(&intent, &spec).await?
			}
			Stage::Plan => {
				let intent = self.read_session("intent.md")?;
				let spec = self.read_session("spec.md")?;
				let plan = self.read_session("plan.md")?;
				let tests = self.read_session("tests.md")?;
				self
					.evaluator
					.evaluate_plan(&intent, &spec, &plan, &tests)
					.await?
			}
			Stage::Build => {
				let intent = self.read_session("intent.md")?;
				let spec = self.read_session("spec.md")?;
				let plan = self.read_session("plan.md")?;
				let tests = self.read_session("tests.md")?;
				self
					.evaluator
					.evaluate_build(&session.dir, &intent, &spec, &plan, &tests)
					.await?
			}
			Stage::Verify => self.evaluator.evaluate_verification(&session.dir).await?,
			Stage::Deploy | Stage::Maintain | Stage::Complete => {
				return Err(anyhow::anyhow!(
					"stage {:?} does not have an evaluation defined",
					stage
				));
			}
		};
		Ok(StageEvaluation {
			stage,
			started_at: Utc::now(),
			actor: evaluation.actor,
			score: evaluation.score,
			confidence: evaluation.confidence,
			passed: evaluation.passed,
		})
	}
	async fn evaluate(&self, checks: &[CheckResult]) -> Result<Vec<EvaluationResult>> {
		let session = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let intent = self.read_session("intent.md")?;
		let spec = self.read_session("spec.md");
		let plan = self.read_session("plan.md");
		let progress = self.read_session("progress.md");
		Ok(vec![])
	}
	fn next_attempt(
		&mut self,
		stage: Stage,
		last_stage: &mut Option<Stage>,
		attempt: &mut u32,
	) -> u32 {
		if *last_stage == Some(stage) {
			*attempt += 1;
		} else {
			*last_stage = Some(stage);
			*attempt = 1;
		}
		*attempt
	}
	fn persist(&self) -> Result<()> {
		FS::save(&self.state_path, &self.session)
	}

	fn read_session(&self, name: &str) -> Result<String> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		read_from_session(name, session)
	}

	fn record_outcome(&mut self, stage: Stage, attempt: u32, outcome: &StageOutcome) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		let (status, evaluation) = match outcome {
			StageOutcome::Complete { evaluation, .. } => {
				(StageStatus::Completed, Some(evaluation.clone()))
			}

			StageOutcome::NeedsRevision { evaluation, .. } => {
				(StageStatus::NeedsRevision, Some(evaluation.clone()))
			}

			StageOutcome::ExecutionFailed(_) => (StageStatus::Failed, None),

			StageOutcome::EvaluationFailed(_) => (StageStatus::EvaluationFailed, None),
		};

		let record = StageRecord {
			stage,
			// attempt,
			status,
			actor: match outcome {
				StageOutcome::Complete { evaluation, .. }
				| StageOutcome::NeedsRevision { evaluation, .. } => evaluation.actor.clone(),

				StageOutcome::ExecutionFailed(_) | StageOutcome::EvaluationFailed(_) => StageActor::Sdlc,
			},
			started_at: Utc::now(),
			completed_at: Some(Utc::now()),
			evaluation,
		};

		session.stages.push(record);
		session.updated_at = Utc::now();

		self.persist()
	}
	fn record_stage_outcome(&mut self, outcome: &StageOutcome, attempt: u32) -> Result<()> {
		let stage = match outcome {
			StageOutcome::Complete { evaluation, .. }
			| StageOutcome::NeedsRevision { evaluation, .. } => evaluation.stage.clone(),

			StageOutcome::ExecutionFailed(_) | StageOutcome::EvaluationFailed(_) => self
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?
				.stage
				.clone(),
		};

		self.record_outcome(stage, attempt, outcome)
	}
	fn record_session(&self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		let index_path = SpecialFile::SessionsIndex.path()?;

		let mut sessions: Vec<SdlcSession> = FS::load(&index_path)?.unwrap_or_default();

		sessions.retain(|existing| existing.id != session.id);
		sessions.push(session.clone());

		FS::save(index_path, &sessions)?;

		Ok(())
	}
	fn record_evaluation(&mut self, evaluation: StageEvaluation) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let status = if evaluation.passed {
			StageStatus::Completed
		} else {
			StageStatus::NeedsRevision
		};
		let record = StageRecord {
			actor: evaluation.actor.clone(),
			// attempt: 0,
			completed_at: Some(Utc::now()),
			evaluation: Some(evaluation.clone()),
			stage: evaluation.stage.clone(),
			started_at: evaluation.started_at,
			status,
		};
		session.stages.push(record);
		session.updated_at = Utc::now();
		self.persist()
	}

	async fn resume(&mut self) -> Result<()> {
		if self.session.is_none() {
			return Err(anyhow::anyhow!("no active SDLC session to resume"));
		}

		self.update_progress("SDLC session resumed")?;
		self.persist()?;

		Ok(())
	}
	fn retry(&mut self, _stage: Stage) -> Result<()> {
		Ok(())
	}
	async fn run_checks(&self) -> Result<Vec<CheckResult>> {
		let checks = [
			("cargo check", vec!["cargo", "check"]),
			("cargo test", vec!["cargo", "test"]),
			(
				"cargo clippy",
				vec!["cargo", "clippy", "--", "-D", "warnings"],
			),
			("cargo fmt", vec!["cargo", "fmt", "--", "--check"]),
		];

		let mut results = Vec::with_capacity(checks.len());

		for (name, command) in checks {
			let result = Command::new(command[0])
				.args(&command[1..])
				.current_dir(env!("CARGO_MANIFEST_DIR"))
				.output()
				.map_err(|error| anyhow::anyhow!("failed to run verification check `{name}`: {error}"))?;

			let stdout = String::from_utf8_lossy(&result.stdout);
			let stderr = String::from_utf8_lossy(&result.stderr);

			let output = if stderr.is_empty() {
				stdout.into_owned()
			} else if stdout.is_empty() {
				stderr.into_owned()
			} else {
				format!("{stdout}\n{stderr}")
			};

			results.push(CheckResult {
				name: name.to_string(),
				passed: result.status.success(),
				output: Some(output),
			});

			if !result.status.success() {
				break;
			}
		}

		Ok(results)
	}

	pub fn session(&self) -> Result<Option<&SdlcSession>> {
		Ok(self.session.as_ref())
	}

	pub fn stage(&self) -> Option<Stage> {
		self.session.as_ref().map(|s| s.stage.clone())
	}
	async fn stage_build(&mut self) -> Result<()> {
		let session = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let context = AgentContext::from_session(session)?;
		self.update_progress("Build started")?;
		let task = context.task.clone();
		self.transition(Stage::Verify)?;
		Ok(())
	}
	async fn stage_deploy(&mut self) -> Result<()> {
		todo!("sdlc deploy")
	}
	async fn stage_maintain(&mut self) -> Result<()> {
		todo!("sdlc maintain")
	}
	async fn stage_intent(&mut self) -> Result<()> {
		let session = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		if session.stage != Stage::Intent {
			return Err(anyhow::anyhow!(
				"cannot execute Intent stage while at {:?}",
				session.stage
			));
		}
		let evaluation = self.evaluate_stage(Stage::Intent).await?;
		self.record_evaluation(evaluation)?;
		self.update_progress("Intent stage completed")?;
		self.transition(Stage::Spec)?;
		Ok(())
	}
	async fn stage_spec(&mut self) -> Result<()> {
		let (stage, session_dir) = {
			let session = self
				.session()?
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
			(session.stage.clone(), session.dir.clone())
		};

		if stage != Stage::Spec {
			return Err(anyhow::anyhow!(
				"cannot execute Spec stage while at {:?}",
				stage
			));
		}

		let intent = self.read_session("intent.md")?;

		let spec = format!(
			"# Specification\n\n\
		 ## Intent\n\n\
		 {}\n\n\
		 ## Requirements\n\n\
		 - The implementation must satisfy the intent above.\n\
		 - The implementation must be testable.\n\
		 - Verification must provide deterministic evidence.\n",
			intent.trim()
		);
		Self::write(session_dir.join("spec.md"), spec);
		let evaluation = self.evaluate_stage(Stage::Spec).await?;
		self.record_evaluation(evaluation)?;
		self.update_progress("Spec stage completed")?;
		self.transition(Stage::Plan)?;

		Ok(())
	}
	async fn stage_plan(&mut self, human_input: Option<&str>) -> Result<()> {
		let (stage, session_dir) = {
			let session = self
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
			(session.stage.clone(), session.dir.clone())
		};
		if stage != Stage::Plan {
			return Err(anyhow::anyhow!(
				"cannot execute Plan stage while at {:?}",
				stage
			));
		}
		let intent = self.read_session("intent.md")?;
		let spec = self.read_session("spec.md")?;
		let plan = self.generate_plan(&intent, &spec).await?;
		Self::write(session_dir.join("plan.md"), plan.clone())?;
		let tests = self.generate_tests(&intent, &spec, &plan).await?;
		Self::write(session_dir.join("tests.md"), tests)?;
		let evaluation = self.evaluate_stage(Stage::Plan).await?;
		self.record_evaluation(evaluation)?;
		self.update_progress("Plan and test plan generated")?;
		self.transition(Stage::Build)?;
		Ok(())
	}

	pub fn subscribe(&self) -> broadcast::Receiver<SdlcEvent> {
		self.event_tx.subscribe()
	}

	fn transition(&mut self, next: Stage) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let valid = matches!(
			(&session.stage, &next),
			(Stage::Intent, Stage::Spec)
				| (Stage::Spec, Stage::Plan)
				| (Stage::Plan, Stage::Build)
				| (Stage::Build, Stage::Verify)
				| (Stage::Verify, Stage::Build)
				| (Stage::Verify, Stage::Complete)
		);

		if !valid {
			return Err(anyhow::anyhow!(
				"invalid SDLC transition: {:?} -> {:?}",
				session.stage,
				next
			));
		}
		session.stage = next;
		session.updated_at = Utc::now();
		self.persist()?;
		self.record_session()?;
		Ok(())
	}
	async fn verify(&mut self) -> Result<Verification> {
		self.update_progress("Verification started")?;
		let checks = self.run_checks().await?;
		let evaluations = self.evaluate(&checks).await?;
		let verification = Verification {
			passed: self.is_passing(&checks, &evaluations),
			checks,
			evaluations,
		};
		self.update_progress(&format!(
			"Verification completed: passed={}",
			verification.passed
		))?;

		Ok(verification)
	}
	fn update_progress(&self, message: &str) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
		let entry = format!("\n## {timestamp}\n\n{message}\n");
		SessionFile::Progress.append(&session.dir, entry)?;
		Ok(())
	}

	fn write(path: PathBuf, contents: String) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}
}
impl SdlcRuntime {
	pub fn new(stage: Stage) -> Self {
		Self {
			activity: vec![],
			attempt: 0,
			confidence: None,
			history: Vec::new(),
			message: None,
			passed: None,
			phase: SdlcPhase::Starting,
			score: None,
			stage,
			stage_started_at: Instant::now(),
			started_at: Instant::now(),
			total_agent_calls: 0,
			total_tokens: 0,
		}
	}
}
impl SdlcSession {
	fn created_at_readable(&self) -> String {
		self.created_at.format(FMT_HUMAN_READABLE).to_string()
	}
	fn updated_at_readable(&self) -> String {
		self.updated_at.format(FMT_HUMAN_READABLE).to_string()
	}
	fn create_readable(&self) -> String {
		let current = Utc::now();
		current.format(FMT_HUMAN_READABLE).to_string()
	}
}
impl SdlcView {
	pub fn new(stage: Stage) -> Self {
		Self {
			input_active: false,
			input: String::new(),
			paused: false,
			show_logs: false,
			runtime: SdlcRuntime::new(stage),
		}
	}

	pub fn begin_input(&mut self) {
		self.input_active = true;
		self.input.clear();
	}
	pub fn end_input(&mut self) {
		self.input_active = false;
		self.input.clear();
	}

	pub fn is_input_active(&self) -> bool {
		self.input_active
	}

	pub fn toggle_pause(&mut self) {
		self.paused = !self.paused;
	}
	pub fn toggle_logs(&mut self) {
		self.show_logs = !self.show_logs;
	}

	pub fn handle_input_key(
		&mut self,
		key: KeyEvent,
		input_tx: &UnboundedSender<SdlcInput>,
	) -> anyhow::Result<()> {
		match key.code {
			KeyCode::Char(c) => {
				self.input.push(c);
			}

			KeyCode::Backspace => {
				self.input.pop();
			}
			KeyCode::Enter => {
				let input = std::mem::take(&mut self.input);
				input_tx.send(SdlcInput::Human(input))?;
				self.input_active = false;
			}

			KeyCode::Esc => {
				self.input_active = false;
				self.input.clear();
			}

			_ => {}
		}

		Ok(())
	}

	pub fn render(frame: &mut Frame<'_>, view: &SdlcView) {
		let area = frame.area();
		frame.render_widget(Clear, area);
		let chunks = RatatuiLayout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(2),
				Constraint::Length(3),
				Constraint::Min(8),
				Constraint::Length(3),
			])
			.split(area);

		ui::stepper(frame, view, chunks[1]);
		let body = ui::body(chunks[2]);
		ui::left_stage_panel(frame, view, body[0]);
		ui::right_activity_panel(frame, view, body[2]);
		ui::footer(frame, view, chunks[3]);
	}
	pub fn apply(&mut self, event: SdlcEvent) {
		self.runtime.history.push(event.clone());
		match event {
			SdlcEvent::HumanInput { stage, input } => {
				self.input_active = true;
				self.input.clear();
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::AwaitingHuman;
				self
					.runtime
					.activity
					.push(format!("Awaiting input: {input}"));
			}
			SdlcEvent::InterventionRequired {
				stage,
				attempt,
				reason,
			} => {
				self.runtime.stage = stage;
				self.runtime.attempt = attempt;
				self.runtime.phase = SdlcPhase::AwaitingHuman;
				self.runtime.message = Some(reason);

				self.begin_input();
			}

			SdlcEvent::InterventionResolved { stage, action } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Retrying;
				self.runtime.message = Some(format!("intervention resolved: {action}"));

				self.end_input();
			}
			SdlcEvent::RunStarted => {
				self.runtime.started_at = Instant::now();
				self.runtime.stage_started_at = Instant::now();
				self.runtime.attempt = 0;
				self.runtime.phase = SdlcPhase::Starting;
				self.runtime.score = None;
				self.runtime.confidence = None;
				self.runtime.passed = None;
				self.runtime.message = None;
				self.runtime.history.clear();
			}

			SdlcEvent::StageStarted { stage, attempt } => {
				self.runtime.stage = stage;
				self.runtime.attempt = attempt;
				self.runtime.stage_started_at = Instant::now();

				self.runtime.phase = SdlcPhase::Starting;
				self.runtime.score = None;
				self.runtime.confidence = None;
				self.runtime.passed = None;
				self.runtime.message = None;
			}

			SdlcEvent::PhaseChanged { phase } => {
				self.runtime.phase = phase;
			}

			SdlcEvent::ExecutionComplete { stage } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Evaluating;
			}

			SdlcEvent::ExecutionFailed { stage, error, .. } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Failed;
				self.runtime.message = Some(error);
			}

			SdlcEvent::EvaluationStarted { stage } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Evaluating;
			}

			SdlcEvent::Evaluated {
				stage,
				score,
				confidence,
				passed,
			} => {
				self.runtime.stage = stage;
				self.runtime.score = Some(score);
				self.runtime.confidence = Some(confidence);
				self.runtime.passed = Some(passed);
			}

			SdlcEvent::EvaluationFailed { stage, error } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Failed;
				self.runtime.message = Some(error);
			}

			SdlcEvent::StageRetrying { stage, attempt } => {
				self.runtime.stage = stage;
				self.runtime.attempt = attempt;
				self.runtime.phase = SdlcPhase::Retrying;
			}

			SdlcEvent::StageTransitioned { to, .. } => {
				self.runtime.stage = to;
			}

			SdlcEvent::Completed => {
				self.runtime.phase = SdlcPhase::Completed;
			}

			SdlcEvent::Exited { reason } => {
				self.runtime.message = Some(reason);
			}

			SdlcEvent::Failed { error, .. } => {
				self.runtime.phase = SdlcPhase::Failed;
				self.runtime.message = Some(error);
			}
			SdlcEvent::RunStarted {} => {}
		}
	}
}
impl Stage {
	pub fn next(&self) -> Option<Self> {
		match self {
			Self::Intent => Some(Self::Spec),
			Self::Spec => Some(Self::Plan),
			Self::Plan => Some(Self::Build),
			Self::Build => Some(Self::Verify),
			Self::Verify => Some(Self::Complete),
			Self::Deploy => Some(Self::Maintain),
			Self::Maintain => Some(Self::Complete),
			Self::Complete => None,
		}
	}
	pub fn is_before(self, other: Stage) -> bool {
		let rank = |stage: Stage| match stage {
			Stage::Intent => 0,
			Stage::Spec => 1,
			Stage::Plan => 2,
			Stage::Build => 3,
			Stage::Verify => 4,
			Stage::Deploy => 5,
			Stage::Maintain => 6,
			Stage::Complete => 7,
		};
		rank(self) < rank(other)
	}
}

pub struct ApiGenerator {
	// whatever API client you decide to use
}
pub struct Check {
	pub name: String,
	pub command: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
	pub name: String,
	pub passed: bool,
	pub output: Option<String>,
}
pub struct CommandResult {
	status: std::process::ExitStatus,
	output: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
	pub name: String,
	pub passed: bool,
	pub score: f64,
	pub confidence: f64,
	pub explanation: String,
}
pub struct Evaluator {
	jev: TypeSafeClient,
}
pub struct LocalGenerator {
	agent: Agent,
}
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Metric {
// 	pub name: String,
// 	pub score: f64,
// 	pub confidence: f64,
// }
pub struct Sdlc {
	state_path: PathBuf,
	session: Option<SdlcSession>,
	// jev: TypeSafeClient,
	evaluator: Evaluator,
	stage_attempt: u32,
	generator: Box<dyn ArtifactGenerator>,
	event_tx: broadcast::Sender<SdlcEvent>,
}
#[derive(Debug, Clone)]
pub struct SdlcRuntime {
	pub activity: Vec<String>,
	pub attempt: u32,
	pub stage: Stage,

	pub started_at: Instant,
	pub stage_started_at: Instant,

	pub phase: SdlcPhase,

	pub score: Option<f32>,
	pub confidence: Option<f32>,
	pub passed: Option<bool>,

	pub message: Option<String>,

	pub total_tokens: u64,
	pub total_agent_calls: u32,

	pub history: Vec<SdlcEvent>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlcSession {
	pub id: String,
	pub title: String,
	pub stage: Stage,
	pub stages: Vec<StageRecord>,
	pub dir: PathBuf,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}
pub struct SdlcView {
	pub runtime: SdlcRuntime,
	pub paused: bool,
	pub show_logs: bool,
	pub input: String,
	pub input_active: bool,
}

pub struct StageExecution {
	pub stage: Stage,
	pub attempt: u32,
	pub result: TaskResult,
	pub started_at: DateTime<Utc>,
	pub completed_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
	// pub attempt: u32,
	pub stage: Stage,
	pub status: StageStatus,

	/// What actually performed the work.
	pub actor: StageActor,

	pub started_at: DateTime<Utc>,
	pub completed_at: Option<DateTime<Utc>>,

	/// Semantic evaluation of the resulting artifact/work.
	pub evaluation: Option<StageEvaluation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEvaluation {
	pub stage: Stage,
	pub actor: StageActor,
	pub started_at: DateTime<Utc>,
	pub score: f64,
	pub confidence: f64,
	pub passed: bool,
	// pub metrics: Vec<EvaluationMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	// pub score: u32,
	pub checks: Vec<CheckResult>,
	pub evaluations: Vec<EvaluationResult>,
}

pub use enums::*;
pub use prompt as agent_prompts;
use prompt::*;

pub use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	// style::{Color, Modifier, Style},
	// text::{Line, Span},
	widgets::Clear,
};

mod ui {
	use crate::sdlc::{SdlcEvent, SdlcPhase, SdlcView, Stage, format_elapsed};
	use ratatui::{
		Frame,
		layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
		style::{Color, Modifier, Style},
		text::{Line, Span},
		widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
	};
	pub fn body(chunk: Rect) -> Vec<Rect> {
		RatatuiLayout::default()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Percentage(54),
				Constraint::Length(1),
				Constraint::Percentage(45),
			])
			.split(chunk)
			.to_vec()
	}
	pub fn header(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let elapsed = format_elapsed(view.runtime.started_at.elapsed());
		let line = Line::from(vec![
			Span::styled(
				format!("{:?}", view.runtime.stage),
				Style::default().add_modifier(Modifier::BOLD),
			),
			Span::raw(format!(
				" · attempt #{}/3   {}",
				view.runtime.attempt, elapsed
			)),
			Span::raw("    [q] quit  [p] pause  [l] logs"),
		]);

		frame.render_widget(Paragraph::new(line), area);
	}
	pub fn stepper(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let stages = [
			Stage::Intent,
			Stage::Spec,
			Stage::Plan,
			Stage::Build,
			Stage::Verify,
			Stage::Complete,
		];

		let current = view.runtime.stage;

		let current_style = match view.runtime.phase {
			SdlcPhase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),

			SdlcPhase::Retrying => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::AwaitingHuman => Style::default()
				.fg(Color::Magenta)
				.add_modifier(Modifier::BOLD),

			_ => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),
		};

		let spinner_text = spinner(view.runtime.stage_started_at.elapsed());

		let mut spans = Vec::new();

		for (index, stage) in stages.iter().copied().enumerate() {
			let (symbol, style) = if stage == current {
				(format!("{spinner_text} "), current_style)
			} else if stage.is_before(current) {
				(
					"✓ ".to_string(),
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD),
				)
			} else {
				(
					"○ ".to_string(),
					Style::default()
						.fg(Color::DarkGray)
						.add_modifier(Modifier::DIM),
				)
			};

			spans.push(Span::styled(symbol, style));

			spans.push(Span::styled(format!("{stage:?}"), style));

			if index + 1 < stages.len() {
				spans.push(Span::styled("  →  ", Style::default().fg(Color::DarkGray)));
			}
		}

		frame.render_widget(Paragraph::new(Line::from(spans)), area);
	}
	pub fn left_stage_panel(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let runtime = &view.runtime;

		let phase_style = phase_style(runtime.phase);

		let score = runtime
			.score
			.map(|value| format!("{value:.2}"))
			.unwrap_or_else(|| "—".into());

		let confidence = runtime
			.confidence
			.map(|value| format!("{value:.2}"))
			.unwrap_or_else(|| "—".into());

		let passed = match runtime.passed {
			Some(true) => Span::styled(
				"yes",
				Style::default()
					.fg(Color::Green)
					.add_modifier(Modifier::BOLD),
			),

			Some(false) => Span::styled(
				"no",
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			),

			None => Span::styled("—", Style::default().fg(Color::DarkGray)),
		};

		let status = match runtime.phase {
			SdlcPhase::Executing => {
				let frame = spinner(runtime.stage_started_at.elapsed());

				Line::from(vec![
					Span::styled(
						format!("{frame} "),
						Style::default()
							.fg(Color::Yellow)
							.add_modifier(Modifier::BOLD),
					),
					Span::styled(
						format!("{:?}", runtime.stage),
						Style::default()
							.fg(Color::White)
							.add_modifier(Modifier::BOLD),
					),
					Span::styled(
						format!("   attempt {}/3", runtime.attempt),
						Style::default().fg(Color::Gray),
					),
				])
			}
			SdlcPhase::Retrying => Line::from(vec![
				Span::styled(
					"↻ ",
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("   retrying · attempt {}/3", runtime.attempt),
					Style::default().fg(Color::Gray),
				),
			]),
			SdlcPhase::Evaluating => Line::from(vec![
				Span::styled(
					"◆ ",
					Style::default()
						.fg(Color::Cyan)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled("   evaluating", Style::default().fg(Color::Cyan)),
			]),
			SdlcPhase::AwaitingHuman => Line::from(vec![
				Span::styled(
					"⚠ ",
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled("  Input", Style::default().fg(Color::Gray)),
			]),
			SdlcPhase::Failed => Line::from(vec![
				Span::styled(
					"✗ ",
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled("   failed", Style::default().fg(Color::Gray)),
			]),
			SdlcPhase::Starting => Line::from(vec![
				Span::styled("· ", Style::default().fg(Color::Yellow)),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
			]),
			SdlcPhase::Completed => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Green)),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD),
				),
			]),
		};

		let lines = vec![
			status,
			Line::from(""),
			Line::from(vec![
				Span::styled("phase       ", Style::default().fg(Color::DarkGray)),
				Span::styled(format!("{:?}", runtime.phase), phase_style),
			]),
			Line::from(vec![
				Span::styled("stage time  ", Style::default().fg(Color::DarkGray)),
				Span::styled(
					format_elapsed(runtime.stage_started_at.elapsed()),
					Style::default().fg(Color::White),
				),
			]),
			Line::from(vec![
				Span::styled("JEV score   ", Style::default().fg(Color::DarkGray)),
				Span::styled(score, Style::default().fg(Color::Cyan)),
			]),
			Line::from(vec![
				Span::styled("confidence  ", Style::default().fg(Color::DarkGray)),
				Span::styled(confidence, Style::default().fg(Color::Cyan)),
			]),
			Line::from(vec![
				Span::styled("passed      ", Style::default().fg(Color::DarkGray)),
				passed,
			]),
		];
		frame.render_widget(
			Paragraph::new(lines).block(
				Block::default()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(Style::default().fg(Color::Gray))
					.style(Style::default().bg(Color::Black))
					.title(Span::styled(
						" Current Stage ",
						Style::default()
							.fg(Color::White)
							.add_modifier(Modifier::BOLD),
					)),
			),
			area,
		);
	}
	pub fn right_activity_panel(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let runtime = &view.runtime;

		let spinner = spinner(runtime.stage_started_at.elapsed());

		let stage_style = Style::default()
			.fg(Color::White)
			.add_modifier(Modifier::BOLD);

		let attempt_style = Style::default().fg(Color::DarkGray);
		let phase_style = phase_style(runtime.phase);

		let phase_label = match runtime.phase {
			SdlcPhase::Starting => "Starting",
			SdlcPhase::Executing => "Executing",
			SdlcPhase::Evaluating => "Evaluating",
			SdlcPhase::Retrying => "Retrying",
			SdlcPhase::AwaitingHuman => "Awaiting input",
			SdlcPhase::Completed => "Completed",
			SdlcPhase::Failed => "Failed",
		};

		let mut lines = Vec::new();

		// ------------------------------------------------------------
		// Current activity
		// ------------------------------------------------------------

		lines.push(Line::from(vec![
			Span::styled(
				format!("{spinner} "),
				Style::default()
					.fg(Color::Yellow)
					.add_modifier(Modifier::BOLD),
			),
			Span::styled(format!("{:?}", runtime.stage), stage_style),
			Span::styled(format!(" · #{}", runtime.attempt), attempt_style),
		]));

		lines.push(Line::from(vec![
			Span::raw("  ↳ "),
			Span::styled(phase_label, phase_style),
		]));

		// ------------------------------------------------------------
		// Current reason
		// ------------------------------------------------------------

		if let Some(message) = &runtime.message {
			lines.push(Line::from(""));
			lines.push(Line::from(Span::styled(
				"  Why",
				Style::default()
					.fg(Color::Gray)
					.add_modifier(Modifier::BOLD),
			)));

			for line in message.lines() {
				lines.push(Line::from(vec![
					Span::raw("     "),
					Span::styled(line, Style::default().fg(Color::DarkGray)),
				]));
			}
		}

		// ------------------------------------------------------------
		// Progression / history
		// ------------------------------------------------------------

		if !runtime.activity.is_empty() {
			lines.push(Line::from(""));
			lines.push(Line::from(Span::styled(
				"  Progress",
				Style::default()
					.fg(Color::Gray)
					.add_modifier(Modifier::BOLD),
			)));

			for activity in runtime.activity.iter().rev().take(8) {
				lines.push(Line::from(vec![
					Span::raw("     • "),
					Span::styled(activity.as_str(), Style::default().fg(Color::White)),
				]));
			}
		}

		frame.render_widget(
			Paragraph::new(lines).wrap(Wrap { trim: true }).block(
				Block::default()
					.borders(Borders::ALL)
					.border_type(BorderType::Plain)
					.border_style(Style::default().fg(Color::DarkGray))
					.title(Span::styled(" Activity ", Style::default().fg(Color::Gray))),
			),
			area,
		);
	}
	pub fn footer(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		if view.is_input_active() {
			let input = Paragraph::new(view.input.as_str())
				.block(
					Block::default()
						.borders(Borders::ALL)
						.title("Human input — Enter to send"),
				)
				.wrap(Wrap { trim: false });

			frame.render_widget(input, area);

			let x = area.x + 1 + view.input.chars().count() as u16;

			let y = area.y + 1;

			frame.set_cursor_position(Position::new(x, y));
		} else {
			let footer = Paragraph::new("p pause  l logs  Ctrl+C quit");

			frame.render_widget(footer, area);
		}
	}
	pub fn event_line(event: &SdlcEvent) -> Line<'static> {
		match event {
			SdlcEvent::HumanInput { stage, input } => Line::from(vec![
				Span::styled("↳ ", Style::default().fg(Color::Magenta)),
				Span::styled(
					format!("{stage:?} human input"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::raw(": "),
				Span::styled(input.clone(), Style::default().fg(Color::Gray)),
			]),
			SdlcEvent::InterventionRequired {
				stage,
				attempt,
				reason,
			} => Line::from(vec![
				Span::styled(
					"⚠ ",
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{stage:?} requires human intervention"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!(" · attempt #{attempt}"),
					Style::default().fg(Color::Gray),
				),
				Span::raw(format!(": {reason}")),
			]),

			SdlcEvent::InterventionResolved { stage, action } => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Magenta)),
				Span::styled(
					format!("{stage:?} intervention"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::raw(format!(" → {action}")),
			]),
			SdlcEvent::RunStarted => Line::from(Span::styled(
				"SDLC started",
				Style::default().fg(Color::Gray),
			)),

			SdlcEvent::StageStarted { stage, attempt } => Line::from(vec![
				Span::styled("● ", Style::default().fg(Color::White)),
				Span::styled(
					format!("{stage:?}"),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!(" · attempt #{attempt}"),
					Style::default().fg(Color::Gray),
				),
			]),

			SdlcEvent::PhaseChanged { phase } => Line::from(vec![
				Span::styled("  phase ", Style::default().fg(Color::DarkGray)),
				Span::styled(format!("→ {phase:?}"), phase_style(*phase)),
			]),

			SdlcEvent::ExecutionComplete { stage } => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Green)),
				Span::styled(
					format!("{stage:?} execution complete"),
					Style::default().fg(Color::Green),
				),
			]),

			SdlcEvent::ExecutionFailed {
				stage,
				attempt,
				error,
			} => Line::from(vec![
				Span::styled("✗ ", Style::default().fg(Color::Red)),
				Span::styled(
					format!("{stage:?} attempt #{attempt}"),
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled(format!(": {error}"), Style::default().fg(Color::Gray)),
			]),

			SdlcEvent::EvaluationStarted { stage } => Line::from(vec![
				Span::styled("◆ ", Style::default().fg(Color::Cyan)),
				Span::styled(
					format!("JEV evaluating {stage:?}"),
					Style::default().fg(Color::Cyan),
				),
			]),

			SdlcEvent::Evaluated {
				stage,
				score,
				confidence,
				passed,
			} => {
				let style = if *passed {
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD)
				} else {
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD)
				};

				Line::from(vec![
					Span::styled("◆ JEV ", Style::default().fg(Color::Cyan)),
					Span::styled(
						format!("{stage:?} · score={score:.2} confidence={confidence:.2}"),
						Style::default().fg(Color::Gray),
					),
					Span::raw(" · "),
					Span::styled(if *passed { "passed" } else { "rejected" }, style),
				])
			}

			SdlcEvent::EvaluationFailed { stage, error } => Line::from(vec![
				Span::styled("✗ JEV ", Style::default().fg(Color::Red)),
				Span::styled(
					format!("{stage:?}: {error}"),
					Style::default().fg(Color::Gray),
				),
			]),

			SdlcEvent::StageRetrying { stage, attempt } => Line::from(vec![
				Span::styled("↻ ", Style::default().fg(Color::Yellow)),
				Span::styled(
					format!("{stage:?} · retry #{attempt}"),
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
			]),

			SdlcEvent::StageTransitioned { from, to } => Line::from(vec![
				Span::styled("→ ", Style::default().fg(Color::Cyan)),
				Span::styled(format!("{from:?}"), Style::default().fg(Color::Gray)),
				Span::raw(" → "),
				Span::styled(
					format!("{to:?}"),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
			]),

			SdlcEvent::Completed => Line::from(Span::styled(
				"✓ SDLC complete",
				Style::default()
					.fg(Color::Green)
					.add_modifier(Modifier::BOLD),
			)),

			SdlcEvent::Exited { reason } => Line::from(vec![
				Span::styled("→ exited ", Style::default().fg(Color::DarkGray)),
				Span::styled(reason.clone(), Style::default().fg(Color::Gray)),
			]),

			SdlcEvent::Failed { stage, error } => {
				let message = match stage {
					Some(stage) => format!("{stage:?}: {error}"),
					None => error.clone(),
				};

				Line::from(vec![
					Span::styled("✗ ", Style::default().fg(Color::Red)),
					Span::styled(message, Style::default().fg(Color::Red)),
				])
			}
		}
	}
	pub fn stage_style(stage: Stage, current: Stage) -> Style {
		match stage {
			stage if stage == current => Style::default()
				.fg(Color::White)
				.add_modifier(Modifier::BOLD),

			stage if stage.is_before(current) => Style::default()
				.fg(Color::Green)
				.add_modifier(Modifier::BOLD),

			_ => Style::default()
				.fg(Color::DarkGray)
				.add_modifier(Modifier::DIM),
		}
	}
	pub fn phase_style(phase: SdlcPhase) -> Style {
		match phase {
			SdlcPhase::Starting => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Executing => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Evaluating => Style::default()
				.fg(Color::Cyan)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Retrying => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::AwaitingHuman => Style::default()
				.fg(Color::Magenta)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Completed => Style::default()
				.fg(Color::Green)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
		}
	}
	pub fn spinner(elapsed: std::time::Duration) -> &'static str {
		const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
		let index = (elapsed.as_millis() / 100) as usize % FRAMES.len();
		FRAMES[index]
	}
}
pub mod prompt {
	use super::*;
	pub fn for_intent() -> Result<String> {
		Ok(String::from(
			"Create a file named hello-world.js in the repository root. It should accept a command-line argument and write that value to hello-world.md. The user should be able to run node hello-world.js \"hi\". Add tests covering both the JavaScript logic and the CLI behavior.",
		))
	}
	pub fn tests_gen(intent: &str, spec: &str, plan: &str) -> String {
		format!(
			r#"
			You are designing the verification plan for an SDLC task.

			The user's intent is authoritative.

			## Intent

			{intent}

			## Specification

			{spec}

			## Implementation Plan

			{plan}

			## Instructions

			Create `tests.md`.

			Every requirement in the specification must have at least
			one corresponding verification test.

			Tests should distinguish between:

			1. deterministic checks
				- cargo test
				- cargo check
				- cargo clippy
				- cargo fmt
				- application-specific commands

			2. behavioral tests
				- unit tests
				- integration tests
				- end-to-end tests

			3. semantic verification
				- requirements that cannot be established purely through
					deterministic commands and should later be evaluated by JEV

			Each test must be concrete enough that another agent can implement
			or execute it.

			Use this format:

			# Tests

			## Requirement: <requirement>

			- [ ] <test>

			Do not mark any test as complete.

			Return only the contents of `tests.md`.
		"#,
		)
	}
	pub fn plan_gen(intent: &str, spec: &str) -> String {
		format!(
			r#"
				You are creating an implementation plan for an SDLC system.

				The user's intent is authoritative.

				## Intent

				{intent}

				## Specification

				{spec}

				## Instructions

				Create a concrete implementation plan.

				The plan must:
				- identify the implementation work required
				- break the work into ordered steps
				- identify files/components likely to change
				- identify dependencies between steps
				- identify how each requirement will be verified
				- avoid inventing requirements not present in the intent or specification

				Return only the contents of `plan.md`.
			"#,
		)
	}
}
mod enums {
	use super::*;
	#[derive(Debug, Clone)]
	pub enum Intervention {
		Human(String),
		Retry,
		ProvideContext(String),
		Reviewed,
		Abort,
	}
	#[derive(Debug, Clone, Copy)]
	pub enum GenerationProvider {
		Local,
		Api,
	}
	#[derive(Debug, Clone)]
	pub enum SdlcEvent {
		Completed,
		Failed {
			stage: Option<Stage>,
			error: String,
		},
		InterventionRequired {
			stage: Stage,
			attempt: u32,
			reason: String,
		},
		InterventionResolved {
			stage: Stage,
			action: String,
		},
		Evaluated {
			stage: Stage,
			score: f32,
			confidence: f32,
			passed: bool,
		},
		EvaluationStarted {
			stage: Stage,
		},
		EvaluationFailed {
			stage: Stage,
			error: String,
		},
		ExecutionComplete {
			stage: Stage,
		},
		ExecutionFailed {
			stage: Stage,
			attempt: u32,
			error: String,
		},
		Exited {
			reason: String,
		},
		PhaseChanged {
			phase: SdlcPhase,
		},
		RunStarted,
		StageRetrying {
			stage: Stage,
			attempt: u32,
		},
		StageStarted {
			stage: Stage,
			attempt: u32,
		},
		StageTransitioned {
			from: Stage,
			to: Stage,
		},
		HumanInput {
			stage: Stage,
			input: String,
		},
	}
	#[derive(Debug)]
	pub enum SdlcInput {
		Abort,
		ProvideContext(String),
		Retry,
		Reviewed,
		Human(String),
	}
	#[derive(Debug, Clone, Copy, PartialEq)]
	pub enum SdlcPhase {
		Starting,
		Executing,
		Evaluating,
		Retrying,
		AwaitingHuman,
		Completed,
		Failed,
	}
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
	pub enum Stage {
		Intent,
		Spec,
		Plan,
		Build,
		Verify,
		Deploy,
		Maintain,
		Complete,
	}
	#[derive(Debug, Clone, Copy)]
	pub enum StageAction {
		Continue,
		Retry,
		Intervene,
		Abort,
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageActor {
		Human,
		Sdlc,
		Agent,
	}
	pub enum StageOutcome {
		/// Stage executed and the resulting artifact passed evaluation.
		Complete {
			result: TaskResult,
			evaluation: StageEvaluation,
		},

		/// Stage executed successfully, but the artifact needs revision.
		NeedsRevision {
			result: TaskResult,
			evaluation: StageEvaluation,
		},

		/// The stage itself could not execute successfully.
		ExecutionFailed(anyhow::Error),

		/// The stage executed, but evaluation could not be completed.
		EvaluationFailed(anyhow::Error),
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageStatus {
		Running,
		Completed,
		Failed,
		NeedsRevision,
		EvaluationFailed,
	}
	pub enum InputMode {
		Normal,
		Human { buffer: String },
		AwaitingHuman { prompt: String },
	}
}

pub struct TerminalGuard;

impl Drop for TerminalGuard {
	fn drop(&mut self) {
		let _ = disable_raw_mode();
		let mut stdout = stdout();
		let _ = execute!(stdout, LeaveAlternateScreen);
		let _ = execute!(stdout, cursor::Show);
	}
}

enum RunControl {
	Continue,
	Exit,
}
