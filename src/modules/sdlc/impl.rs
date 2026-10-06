pub use ratatui::Frame;

use super::{AiSession, *};

#[async_trait]
impl ArtifactGenerator for ApiGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		todo!("API generate")
	}
	async fn run_agent(&self, prompt: &str) -> Result<String> {
		todo!("API generate")
	}
	async fn with_session(&mut self, session: &AiSession, prompt: String) -> Result<TaskResult> {
		todo!("with_session")
	}
	fn clone_box(&self) -> Box<dyn ArtifactGenerator> {
		Box::new(self.clone())
	}
}
#[async_trait]
impl ArtifactGenerator for LocalGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		let request = serde_json::json!({
			"model": self.model,
			"prompt": prompt,
			"stream": false,
		});
		std::fs::write(
			"/tmp/estate-ollama-request.json",
			serde_json::to_string_pretty(&request)?,
		)?;
		let response = reqwest::Client::new()
			.post("http://localhost:11434/api/generate")
			.json(&request)
			.send()
			.await?
			.error_for_status()?;
		let body = response.text().await?;
		std::fs::write("/tmp/estate-ollama-response.json", &body)?;
		let response: OllamaResponse =
			serde_json::from_str(&body).context("invalid Ollama response")?;
		let artifact = response.response.trim();
		if artifact.is_empty() {
			return Err(anyhow!(
				"Ollama returned an empty artifact \
				 (model={}, done_reason={:?})",
				self.model,
				response.done_reason,
			));
		}
		Ok(artifact.to_string())
	}
	// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-intern
	// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-host-env
	async fn run_agent(&self, prompt: &str) -> Result<String> {
		let task = AgentTask::new(prompt.to_string());
		let result = self
			.runtime
			.run_agent(task)
			// .run_agent_with_sdlc(task, Path::new("/Users/future/kb/project/crates/estate/log/"))
			.await?;
		Ok(
			result
				.chat
				.or(result.summary)
				.unwrap_or_else(|| "Agent completed".to_string()),
		)
	}
	async fn with_session(&mut self, session: &AiSession, prompt: String) -> Result<TaskResult> {
		let task = AgentTask::new(prompt.to_string());
		let result = self.runtime.from_session(task, session).await?;
		Ok(result)
	}
	fn clone_box(&self) -> Box<dyn ArtifactGenerator> {
		Box::new(self.clone())
	}
}

impl Attempt {
	pub fn new() -> Self {
		Self {
			stage: Stage::Intent,
			number: 1,
			max: 3,
		}
	}
}
impl std::fmt::Display for Attempt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} attempt {}", self.stage, self.number)
	}
}
impl Evaluator {
	async fn evaluate(
		&self,
		session: &AiSession,
		execution: &StageExecution,
	) -> Result<StageEvaluation> {
		let ctx = EvaluationContext::load(session, execution.stage)?;
		let evaluation = match execution.stage {
			Stage::Intent => self.evaluate_intent(&ctx).await,
			Stage::Spec => self.evaluate_spec(&ctx).await,
			Stage::Plan => self.evaluate_plan(&ctx).await,
			Stage::Build => self.evaluate_build(&ctx).await,
			Stage::Verify => self.evaluate_verification(&ctx).await,
			stage => Err(anyhow::anyhow!(
				"stage {stage:?} does not support evaluation"
			)),
		};
		return evaluation;
	}
	async fn evaluate_goal(&self, goal: &str) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let state = format!(
			"## User Goal\n\n{goal}\n\n\
			## Evaluation Context\n\n\
			Evaluate this as a software-development goal. \
			The goal should be useful as the starting point for an \
			SDLC pipeline. Judge the goal itself, not how well an \
			implementation could compensate for missing information."
		);
		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"smart",
						Question::from(Score::new(
							"How well does this goal satisfy the SMART criteria \
							 for a software-development task?",
							[
								"Unusable: the goal is unclear, unbounded, or not actionable",
								"Weak: several SMART dimensions are substantially missing",
								"Usable: most SMART dimensions are present, but important ambiguity remains",
								"Strong: the goal is specific, measurable, attainable, realistic, and timely",
								"Excellent: the goal is precise, bounded, measurable, realistic, \
								 and provides a strong basis for implementation and verification",
							],
						)),
					),
					(
						"specific",
						Question::from(Score::new(
							"How specific is this software goal?",
							[
								"Unusable: no concrete software outcome is identifiable",
								"Weak: a general desire is expressed but the target outcome is ambiguous",
								"Usable: the primary software outcome is identifiable with some ambiguity",
								"Strong: the software outcome, scope, and relevant target are clearly defined",
								"Excellent: the desired software outcome and scope are precise and unambiguous",
							],
						)),
					),
					(
						"measurable",
						Question::from(Score::new(
							"How measurable is this software goal?",
							[
								"Unusable: there is no way to determine whether the goal was achieved",
								"Weak: success is mostly subjective or requires substantial invention",
								"Usable: some observable success criteria can be inferred",
								"Strong: success can be established through concrete observable criteria",
								"Excellent: the goal contains explicit, verifiable measures of success",
							],
						)),
					),
					(
						"attainable",
						Question::from(Score::new(
							"How attainable is this goal as a software-development task?",
							[
								"Unusable: the requested outcome is technically incoherent or impossible to act on",
								"Weak: substantial feasibility or dependency uncertainty exists",
								"Usable: the task appears feasible with some assumptions",
								"Strong: the requested outcome is plausibly achievable with ordinary software-development work",
								"Excellent: the goal is clearly achievable within the stated scope and constraints",
							],
						)),
					),
					(
						"realistic",
						Question::from(Score::new(
							"How realistic is this goal given the stated software context, \
							 constraints, dependencies, and expected scope?",
							[
								"Unusable: the goal conflicts with its context or contains unrealistic assumptions",
								"Weak: important assumptions or scope problems make the goal questionable",
								"Usable: the goal is broadly realistic but some assumptions need clarification",
								"Strong: the goal fits its context, constraints, and expected implementation scope",
								"Excellent: the goal is well-bounded and realistic with no major unstated assumptions",
							],
						)),
					),
					(
						"timely",
						Question::from(Score::new(
							"How well does this goal define a useful timeframe or completion boundary \
							 for the software task?",
							[
								"Unusable: there is no meaningful completion boundary",
								"Weak: completion timing or boundaries are substantially unclear",
								"Usable: the scope provides an implicit or approximate completion boundary",
								"Strong: the goal provides a clear completion boundary or useful timeframe",
								"Excellent: the goal contains a precise, verifiable timeframe, deadline, \
								 milestone, or explicit completion boundary",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Is this goal sufficiently clear, bounded, and software-actionable \
							 to begin an SDLC process without an implementation agent inventing \
							 major requirements?",
						)),
					),
				],
			)
			.await?;
		let smart = response
			.score("smart")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no SMART score"))?;
		let dimensions = [
			("specific", "specificity"),
			("measurable", "measurability"),
			("attainable", "attainability"),
			("realistic", "realism"),
			("timely", "timeliness"),
		];
		let mut evaluations = Vec::with_capacity(7);
		evaluations.push(EvaluationResult {
			name: "smart".into(),
			passed: smart.score >= 0.80,
			score: smart.score,
			confidence: smart.confidence,
			explanation: String::new(),
		});
		for (key, name) in dimensions {
			let result = response
				.score(key)
				.ok_or_else(|| anyhow::anyhow!("JEV returned no {name} score"))?;

			evaluations.push(EvaluationResult {
				name: name.into(),
				passed: result.score >= 0.80,
				score: result.score,
				confidence: result.confidence,
				explanation: String::new(),
			});
		}
		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no goal decision"))?;
		evaluations.push(EvaluationResult {
			name: "meets_bar".into(),
			passed: meets_bar.noul >= 0.80,
			score: meets_bar.noul,
			confidence: 1.0,
			explanation: String::new(),
		});
		Ok(StageEvaluation::new(
			Stage::Intent,
			StageActor::Evaluator,
			time_started,
			smart.score,
			smart.confidence,
			meets_bar.noul,
			evaluations,
		))
	}

	async fn evaluate_intent(&self, ctx: &EvaluationContext) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let intent = ctx.get(SessionFile::Intent)?;
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
		Ok(StageEvaluation::new(
			Stage::Intent,
			StageActor::Evaluator,
			time_started,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_spec(&self, ctx: &EvaluationContext) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let intent = ctx.get(SessionFile::Intent)?;
		let spec = ctx.get(SessionFile::Spec)?;
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
		Ok(StageEvaluation::new(
			Stage::Spec,
			StageActor::Sdlc,
			time_started,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_plan(&self, ctx: &EvaluationContext) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let intent = ctx.get(SessionFile::Intent)?;
		let spec = ctx.get(SessionFile::Spec)?;
		let plan = ctx.get(SessionFile::Plan)?;
		let state = format!(
			"## User Intent\n\n{intent}\n\n\
         ## Specification\n\n{spec}\n\n\
         ## Implementation Plan\n\n{plan}\n\n\
         "
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
		Ok(StageEvaluation::new(
			Stage::Plan,
			StageActor::Agent,
			time_started,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_build(&self, ctx: &EvaluationContext) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let intent = ctx.get(SessionFile::Intent)?;
		let spec = ctx.get(SessionFile::Spec)?;
		let plan = ctx.get(SessionFile::Plan)?;
		// let tests = ctx.get(SessionFile::Test)?;
		let implementation = std::fs::read_dir(&self.session.dir)?
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
		Ok(StageEvaluation::new(
			Stage::Build,
			StageActor::Sdlc,
			time_started,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_verification(&self, ctx: &EvaluationContext) -> Result<StageEvaluation> {
		let time_started = Utc::now();
		let intent = ctx.get(SessionFile::Intent)?;
		let spec = ctx.get(SessionFile::Spec)?;
		// let tests = ctx.get(SessionFile::Test)?;
		let evidence = ctx
			.get(SessionFile::Verification)
			.unwrap_or("No verification evidence was recorded.");
		let state = format!(
			"## User Intent\n\n{intent}\n\n\
		 ## Specification\n\n{spec}\n\n\
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
		Ok(StageEvaluation::new(
			Stage::Verify,
			StageActor::Sdlc,
			time_started,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
}
impl EvaluationContext {
	fn get(&self, file: SessionFile) -> Result<&str> {
		match file {
			SessionFile::Intent => self
				.intent
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("intent artifact not loaded")),
			SessionFile::Spec => self
				.spec
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("spec artifact not loaded")),

			SessionFile::Plan => self
				.plan
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("plan artifact not loaded")),
			SessionFile::Test => self
				.tests
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("tests artifact not loaded")),
			SessionFile::Build => self
				.tests
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("build artifact not loaded")),

			SessionFile::Progress => self
				.progress
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("progress artifact not loaded")),

			SessionFile::Verification => self
				.verification
				.as_deref()
				.ok_or_else(|| anyhow::anyhow!("verification artifact not loaded")),
		}
	}
	fn load(session: &AiSession, stage: Stage) -> Result<Self> {
		let read = |file: SessionFile| -> Option<String> { file.read(&session.dir).ok() };
		Ok(Self {
			stage,
			intent: match stage {
				Stage::Intent | Stage::Spec | Stage::Plan | Stage::Build | Stage::Verify => {
					read(SessionFile::Intent)
				}
				_ => None,
			},
			spec: match stage {
				Stage::Spec | Stage::Plan | Stage::Build | Stage::Verify => read(SessionFile::Spec),
				_ => None,
			},
			plan: match stage {
				Stage::Plan | Stage::Build | Stage::Verify => read(SessionFile::Plan),
				_ => None,
			},
			tests: match stage {
				Stage::Plan | Stage::Build | Stage::Verify => read(SessionFile::Test),
				_ => None,
			},
			progress: match stage {
				Stage::Build | Stage::Verify => read(SessionFile::Progress),
				_ => None,
			},
			verification: match stage {
				Stage::Verify => read(SessionFile::Verification),
				_ => None,
			},
		})
	}
}

impl LocalGenerator {
	pub fn new(runtime: AgentRuntime, model: impl Into<String>) -> Self {
		Self {
			runtime,
			model: model.into(),
		}
	}
}
impl PipelineRuntime {
	pub fn new(pipeline: SprintPipeline) -> Self {
		let stage = pipeline.stage().clone();
		Self {
			stage,
			pipeline,
			activity: vec![],
			attempt: 0,
			confidence: None,
			history: Vec::new(),
			message: None,
			passed: None,
			phase: SdlcPhase::Starting,
			score: None,
			stage_time_started: Instant::now(),
			time_started: Instant::now(),
			total_agent_calls: 0,
			total_tokens: 0,
			error: None,
			events: vec![],
		}
	}
	fn retry(&mut self, _stage: Stage) -> Result<()> {
		Ok(())
	}
	pub async fn run(&mut self, input_rx: &mut UnboundedReceiver<SdlcInput>) -> Result<()> {
		let mut runner = SprintRunner {
			pipeline: &mut self.pipeline,
		};
		runner.run(input_rx).await.context("SprintRunner::run")
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
				phase: SdlcPhase::Executing,
			});
			sleep(DEMO_EXECUTION_TIME).await;
			self.pipeline.emit(SdlcEvent::ExecutionComplete { stage });
			self.pipeline.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Evaluating,
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
		self.pipeline.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Completed,
		});
		self.pipeline.emit(SdlcEvent::Completed);
		sleep(DEMO_COMPLETE_DELAY).await;
		Ok(())
	}
	pub async fn resume_from(
		&mut self,
		stage: Stage,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<()> {
		tracing::info!("resume_from pipeline runtime");

		self.pipeline.resume_from(stage).await?;

		let generator = self.pipeline.generator.as_mut();

		generator
			.with_session(
				&self.pipeline.session.clone(),
				self.pipeline.session.goal.clone(),
			)
			.await?;

		let mut runner = SprintRunner {
			pipeline: &mut self.pipeline,
		};

		runner.run(input_rx).await.context("SprintRunner::run")
	}
	pub fn view(&self) -> PipelineRuntimeView {
		PipelineRuntimeView {
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
			events: self.events.clone(),
		}
	}
}

impl AiSession {
	fn create_readable(&self) -> String {
		let current = Utc::now();
		current.format(FMT_HUMAN_READABLE).to_string()
	}
	fn empty() -> Result<()> {
		Ok(())
	}
	pub fn new(title: impl Into<String>, dir: PathBuf) -> Result<Self> {
		let now = Utc::now();
		let goal = include_str!("../../../ai/template/user.goal.md").to_string();
		Ok(Self {
			workspace: dir.clone(),
			goal,
			id: uuid::Uuid::new_v4(),
			title: title.into(),
			stage: Stage::Intent,
			stages: Vec::new(),
			dir,
			time_created: now,
			time_updated: now,
		})
	}
	pub fn workspace(&self) -> &Path {
		&self.workspace
	}
}
impl crate::traits::DateableSession for AiSession {
	fn start(&self) -> Option<DateTime<Utc>> {
		Some(self.time_created)
	}
	fn end(&self) -> Option<DateTime<Utc>> {
		Some(self.time_updated)
	}
}
impl SdlcInput {
	pub fn text(&self) -> Option<&str> {
		match self {
			Self::Human(value) | Self::ProvideContext(value) => Some(value),
			Self::Abort | Self::Retry | Self::Reviewed | Self::Revision { .. } => None,
		}
	}
}

impl Stage {
	pub fn next(&self) -> Option<Self> {
		match self {
			Self::Intent => Some(Self::Spec),
			Self::Spec => Some(Self::Plan),
			// Self::Plan => Some(Self::Test),
			Self::Plan => Some(Self::Build),
			Self::Test => Some(Self::Build),
			Self::Build => Some(Self::Verify),
			Self::Verify => Some(Self::Complete),
			Self::Deploy => Some(Self::Maintain),
			Self::Maintain => Some(Self::Complete),
			Self::Complete => None,
			Self::Finalize => None,
		}
	}
	pub fn is_before(self, other: Stage) -> bool {
		let rank = |stage: Stage| match stage {
			Stage::Intent => 0,
			Stage::Spec => 1,
			Stage::Plan => 2,
			Stage::Test => 3,
			Stage::Build => 4,
			Stage::Verify => 5,
			Stage::Deploy => 6,
			Stage::Maintain => 7,
			Stage::Complete => 8,
			Stage::Finalize => 100,
		};
		rank(self) < rank(other)
	}
}
impl std::fmt::Display for Stage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = match self {
			Self::Intent => "Intent",
			Self::Spec => "Spec",
			Self::Plan => "Plan",
			Self::Test => "Test",
			Self::Build => "Build",
			Self::Verify => "Verify",
			Self::Deploy => "Deploy",
			Self::Maintain => "Maintain",
			Self::Complete => "Complete",
			Self::Finalize => "Finalize Sprint",
		};

		f.write_str(name)
	}
}
impl StageEvaluation {
	fn new(
		stage: Stage,
		actor: StageActor,
		time_started: DateTime<Utc>,
		score: f64,
		confidence: f64,
		meets_bar: f64,
		evaluations: Vec<EvaluationResult>,
	) -> Self {
		let time_completed = Utc::now();
		let time_total = time_completed - time_started;
		Self {
			stage,
			actor,
			time_started,
			time_completed,
			time_total,
			score,
			confidence,
			passed: meets_bar >= 0.80 && confidence >= 0.70,
			evaluations,
		}
	}
	pub fn time_total_readable(&self) -> String {
		duration_readable(self.time_total)
	}
	pub fn start_readable(&self) -> String {
		time_readable(self.time_started)
	}

	pub fn end_readable(&self) -> String {
		time_readable(self.time_completed)
	}
	pub fn with_evaluations(&self, checks: Vec<CheckResult>) -> Self {
		todo!("with_evaluations")
	}
}
impl StageOutcome {
	pub fn attempt(&self) -> Attempt {
		match self {
			Self::Complete { execution, .. }
			| Self::NeedsRevision { execution, .. }
			| Self::EvaluationFailed { execution, .. } => execution.attempt,

			Self::ExecutionFailed { attempt, .. } => *attempt,
		}
	}
	pub fn stage(&self) -> Stage {
		match self {
			Self::Complete { execution, .. }
			| Self::NeedsRevision { execution, .. }
			| Self::EvaluationFailed { execution, .. } => execution.stage,

			Self::ExecutionFailed { stage, .. } => *stage,
		}
	}
}

impl SprintPipeline {
	fn checks_for(stage: Stage) -> Vec<(&'static str, Vec<&'static str>)> {
		match stage {
			Stage::Intent | Stage::Spec | Stage::Plan => Vec::new(),
			Stage::Complete | Stage::Finalize => Vec::new(),
			Stage::Deploy | Stage::Maintain => Vec::new(),
			Stage::Build | Stage::Verify => vec![
				("cargo check", vec!["cargo", "check"]),
				("cargo test", vec!["cargo", "test"]),
				(
					"cargo clippy",
					vec!["cargo", "clippy", "--", "-D", "warnings"],
				),
				("cargo fmt", vec!["cargo", "fmt", "--", "--check"]),
			],
			_ => Vec::new(),
		}
	}

	fn emit(&self, event: SdlcEvent) {
		let _ = self.event_tx.send(event);
	}

	async fn evaluate(&self, execution: &StageExecution) -> Result<StageEvaluation> {
		self.evaluator.evaluate(&self.session, execution).await
	}
	fn evaluate_checks(&self, checks: &[CheckResult]) -> Vec<EvaluationResult> {
		checks
			.iter()
			.map(|check| EvaluationResult {
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
	pub async fn init(&mut self, intent: impl Into<String>) -> Result<()> {
		let intent = intent.into();
		let title = Self::summarize_title(&intent).await?;
		let dir = Self::init_session_dir(&title)?;
		Self::init_templates(&dir)?;
		let session = AiSession::new(title, dir)?;
		self.session = session;
		self.stage_attempt = 1;
		self.persist_session()?;
		Ok(())
	}
	fn init_session_dir(title: &str) -> Result<PathBuf> {
		let sessions_dir = FS::ensure_dir(SpecialFile::WriteDir.path()?)?;
		let date = Local::now().format("%Y-%m-%d");
		let dir = sessions_dir.join(format!("{date}.{title}"));
		FS::ensure_dir(&dir)?;
		Ok(dir)
	}
	fn init_templates(dir: &Path) -> Result<()> {
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
	// pub async fn new(runtime: AgentRuntime) -> anyhow::Result<Self> {
	// 	dotenvy::dotenv().ok();
	// 	let state_path = SpecialFile::LogFile.path()?;
	// 	let session = match SpecialFile::LogFile
	// 		.load::<AiSession>()
	// 		.context("loading current AiSession")?
	// 	{
	// 		Some(session) => session,
	// 		None => {
	// 			let intent = "Do the work required to build this CLI";
	// 			let title = Self::summarize_title(intent).await?;
	// 			let dir = Self::init_session_dir(&title)?;
	// 			Self::init_templates(&dir)?;
	// 			let session = AiSession::new(title, dir)?;
	// 			Self::session_save(&session)?;
	// 			session
	// 		}
	// 	};
	// 	if !state_path.exists() {
	// 		Self::session_save(&session)?;
	// 	}
	// 	println!("last stage: {:?}", session.stages.last());
	// 	let evaluator = Evaluator {
	// 		session: session.clone(),
	// 		jev: TypeSafeClient::from_env()?,
	// 	};
	// 	let generator = Box::new(LocalGenerator::new(runtime.clone(), "qwen3:8b"));
	// 	let (event_tx, _event_rx) = tokio::sync::broadcast::channel::<SdlcEvent>(256);
	// 	Ok(Self {
	// 		evaluator,
	// 		generator,
	// 		session: Some(session),
	// 		state_path,
	// 		event_tx,
	// 		stage_attempt: 1,
	// 	})
	// }
	pub async fn new(runtime: AgentRuntime, intent: impl Into<String>) -> anyhow::Result<Self> {
		dotenvy::dotenv().ok();
		let state_path = SpecialFile::WriteCurrent.path()?;
		let session = match SpecialFile::WriteCurrent
			.load::<AiSession>()
			.context("loading current AiSession")?
		{
			Some(session) => session,
			None => {
				let intent = intent.into();
				let title = Self::summarize_title(&intent).await?;
				let dir = Self::init_session_dir(&title)?;
				Self::init_templates(&dir)?;
				let session = AiSession::new(title, dir)?;
				Self::session_save(&session)?;
				session
			}
		};
		let evaluator = Evaluator {
			session: session.clone(),
			jev: TypeSafeClient::from_env()?,
		};
		let generator = Box::new(LocalGenerator::new(runtime.clone(), "qwen3:8b"));
		let (event_tx, _event_rx) = tokio::sync::broadcast::channel::<SdlcEvent>(256);
		Ok(Self {
			evaluator,
			generator,
			session,
			state_path,
			event_tx,
			stage_attempt: 0,
		})
	}
	fn next_attempt(&mut self, stage: Stage) -> Result<Attempt> {
		let session = self.session()?;
		let max = 3;
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
		FS::save(&self.state_path, &self.session)
		// FS::save(SpecialFile::WriteDir.path()?, &self.session)
	}
	fn persist_evaluation(&mut self, evaluation: &StageEvaluation, attempt: Attempt) -> Result<()> {
		let session = self.session()?;
		persist_evaluation(session, evaluation, attempt)?;
		self.persist()
	}
	fn persist_intervention(&mut self, attempt: Attempt, reason: impl Into<String>) -> Result<()> {
		let now = Utc::now();
		self.push_stage_record(StageRecord {
			stage: attempt.stage,
			attempt,
			status: StageStatus::InterventionNeeded,
			actor: StageActor::Sdlc,
			time_started: now,
			time_completed: Some(now),
			description: Some(reason.into()),
			evaluation: None,
		})
	}
	fn persist_outcome(&mut self, outcome: &StageOutcome) -> Result<()> {
		let record = match outcome {
			StageOutcome::Complete {
				execution,
				evaluation,
			} => StageRecord {
				stage: execution.stage,
				attempt: execution.attempt,
				status: StageStatus::Completed,
				actor: evaluation.actor.clone(),
				time_started: execution.time_started,
				time_completed: Some(execution.time_completed),
				description: Some(format!("Stage Completed: {}", execution.stage.clone())),
				evaluation: Some(evaluation.clone()),
			},

			StageOutcome::NeedsRevision {
				execution,
				evaluation,
			} => StageRecord {
				stage: execution.stage,
				attempt: execution.attempt,
				status: StageStatus::NeedsRevision,
				actor: evaluation.actor.clone(),
				time_started: execution.time_started,
				time_completed: Some(execution.time_completed),
				description: Some(format!("Needs Revision: {}", execution.stage.clone())),
				evaluation: Some(evaluation.clone()),
			},

			StageOutcome::ExecutionFailed {
				stage,
				attempt,
				error,
			} => StageRecord {
				stage: *stage,
				attempt: *attempt,
				status: StageStatus::Failed,
				actor: StageActor::Sdlc,
				time_started: Utc::now(),
				time_completed: Some(Utc::now()),
				description: Some(error.to_string()),
				evaluation: None,
			},

			StageOutcome::EvaluationFailed { execution, error } => StageRecord {
				stage: execution.stage,
				attempt: execution.attempt,
				status: StageStatus::EvaluationFailed,
				actor: StageActor::Sdlc,
				time_started: execution.time_started,
				time_completed: Some(execution.time_completed),
				description: Some(error.to_string()),
				evaluation: None,
			},
		};
		self.push_stage_record(record)
	}
	fn persist_progress(&mut self, message: &str) -> Result<()> {
		let session = self.session()?;
		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
		let entry = format!("\n## {timestamp}\n\n{message}\n");
		SessionFile::Progress.append(&session.dir, entry)?;
		Ok(())
	}
	fn persist_session(&mut self) -> Result<()> {
		let session = self.session()?;
		Self::session_save(session)
	}
	async fn resume(&mut self) -> Result<()> {
		self.persist_progress("SDLC session resumed")?;
		self.persist()?;
		Ok(())
	}
	async fn resume_from(&mut self, stage: Stage) -> Result<()> {
		tracing::info!("resume_from sprintpi pipeline");
		self.set_stage(stage)?;
		self.persist_progress(&format!("SDLC session resumed from {stage}"))?;
		self.persist()?;
		Ok(())
	}
	fn push_stage_record(&mut self, record: StageRecord) -> Result<()> {
		let session = self.session()?;
		session.stages.push(record);
		session.time_updated = Utc::now();
		self.persist()
	}
	async fn run_checks(&self, stage: Stage) -> Result<Vec<CheckResult>> {
		let checks = Self::checks_for(stage);
		let mut results = Vec::with_capacity(checks.len());
		for (name, command) in checks {
			let result = Command::new(command[0])
				.args(&command[1..])
				.current_dir(env!("CARGO_MANIFEST_DIR"))
				.output()
				.map_err(|error| anyhow::anyhow!("failed to run verification check `{name}`: {error}"))?;

			let stdout = String::from_utf8_lossy(&result.stdout);
			let stderr = String::from_utf8_lossy(&result.stderr);

			let output = match (stdout.is_empty(), stderr.is_empty()) {
				(false, false) => format!("{stdout}\n{stderr}"),
				(false, true) => stdout.into_owned(),
				(true, false) => stderr.into_owned(),
				(true, true) => String::new(),
			};

			let passed = result.status.success();

			results.push(CheckResult {
				name: name.to_string(),
				passed,
				output: Some(output),
			});

			if !passed {
				break;
			}
		}

		Ok(results)
	}
	fn retry(&mut self, _stage: Stage) -> Result<()> {
		Ok(())
	}
	pub fn set_stage(&mut self, stage: Stage) -> Result<()> {
		self.session()?.stage = stage;
		Ok(())
	}
	// fn session(&self) -> Result<&Session> {
	// 	Ok(&self.session)
	// }
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
	fn session_save(session: &AiSession) -> Result<()> {
		FS::save(SpecialFile::WriteCurrent.path()?, session)?;
		Ok(())
	}

	pub fn stage(&self) -> Stage {
		self.session.stage
	}
	fn stage_attempt(&self) -> u32 {
		self.stage_attempt
	}
	pub fn stage_attempt_increment(&mut self) -> u32 {
		self.stage_attempt += 1;
		self.stage_attempt
	}
	pub fn stage_attempt_reset(&mut self) {
		self.stage_attempt = 1;
	}

	pub fn subscribe(&self) -> broadcast::Receiver<SdlcEvent> {
		self.event_tx.subscribe()
	}
	async fn summarize_title(intent: &str) -> Result<String> {
		Ok(String::from("create-sdlc-pipeline"))
	}

	fn transition(&mut self, next: Stage) -> Result<()> {
		let session = self.session()?;
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
		session.time_updated = Utc::now();

		self.persist()?;
		self.session_record()?;

		Ok(())
	}

	async fn wait_for_intervention(
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
			SdlcInput::Revision { evaluation } => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "revision".into(),
				});
				Ok(Intervention::Revision { evaluation })
			}
		}
	}

	fn write(path: PathBuf, contents: String) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}
}
impl SdlcView {
	pub fn apply(&mut self, event: SdlcEvent) {
		match event {
			SdlcEvent::RunStarted => {
				self.runtime.phase = SdlcPhase::Starting;
				self.runtime.time_started = Instant::now();
				self.runtime.stage_time_started = Instant::now();
				self.runtime.message = Some(String::from("Run started"));
			}

			SdlcEvent::StageStarted { stage, attempt } => {
				self.runtime.stage = stage;
				self.runtime.attempt = attempt.number;
				self.runtime.phase = SdlcPhase::Starting;
				self.runtime.stage_time_started = Instant::now();
				self.runtime.score = None;
				self.runtime.confidence = None;
				self.runtime.error = None;
				self.runtime.message = Some(format!("{stage:?}"));
			}

			SdlcEvent::Activity {
				stage,
				attempt,
				message,
			} => {
				self.runtime.stage = stage;
				self.runtime.attempt = attempt.number;
				self.runtime.message = Some(message);
			}

			SdlcEvent::PhaseChanged { phase } => {
				self.runtime.phase = match phase {
					SdlcPhase::Executing => SdlcPhase::Executing,
					SdlcPhase::Evaluating => SdlcPhase::Evaluating,
					SdlcPhase::Completed => SdlcPhase::Completed,

					// Add the remaining mappings for your actual
					// SdlcPhase variants.
					_ => self.runtime.phase,
				};

				self.runtime.message = Some(format!("{phase:?}"));
			}

			SdlcEvent::ExecutionComplete { stage } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Evaluating;
				self.runtime.message = Some(String::from("Execution complete"));
			}

			SdlcEvent::EvaluationStarted { stage } => {
				self.runtime.stage = stage;
				self.runtime.phase = SdlcPhase::Evaluating;
				self.runtime.message = Some(String::from("Evaluating"));
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

				self.runtime.message = Some(format!(
					"Evaluation: {:.2} (confidence {:.2})",
					score, confidence
				));

				if !passed {
					self.runtime.phase = SdlcPhase::Failed;
				}
			}

			SdlcEvent::StageTransitioned { from: _, to } => {
				self.runtime.stage = to;
				self.runtime.stage_time_started = Instant::now();
				self.runtime.score = None;
				self.runtime.confidence = None;
				self.runtime.message = Some(format!("Starting {to:?}"));
			}

			SdlcEvent::Completed => {
				self.runtime.phase = SdlcPhase::Completed;
				self.runtime.message = Some(String::from("SDLC complete"));
			}

			SdlcEvent::Failed { stage, error } => {
				if let Some(stage) = stage {
					self.runtime.stage = stage;
				}

				self.runtime.phase = SdlcPhase::Failed;
				self.runtime.message = Some(error.clone());
				self.runtime.error = Some(error);
			}

			event => {
				self.runtime.events.push(event);
			}
		}
	}
	pub fn handle_input_key(
		&mut self,
		key: crossterm::event::KeyEvent,
		input_tx: &UnboundedSender<SdlcInput>,
	) -> anyhow::Result<()> {
		use crossterm::event::KeyCode;
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
	pub fn new(runtime: &PipelineRuntime) -> Self {
		Self {
			input_active: false,
			input: String::new(),
			paused: false,
			show_logs: false,
			events: vec![],
			runtime: runtime.view(),
		}
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
		stepper(frame, view, chunks[1]);
		let body = body(chunks[2]);
		left_stage_panel(frame, view, body[0]);
		right_activity_panel(frame, view, body[2]);
		footer(frame, view, chunks[3]);
	}
	pub fn toggle_pause(&mut self) {
		self.paused = !self.paused;
	}
	pub fn toggle_logs(&mut self) {
		self.show_logs = !self.show_logs;
	}
}
#[async_trait::async_trait]
impl sdlc_trait::Runner for SprintRunner<'_> {
	type Context = UnboundedReceiver<SdlcInput>;
	type Output = ();
	/// "What happens next?"
	///
	/// Orchestrates the SDLC state machine:
	/// run stage -> persist outcome -> decide -> apply -> follow control.
	async fn run(&mut self, input_rx: &mut Self::Context) -> Result<Self::Output> {
		let mut pending_input = None;
		let mut stage = self.pipeline.stage();
		self.emit(SdlcEvent::RunStarted);
		loop {
			let attempt = self.attempt_stage(stage)?;
			section!(&format!(" stage = {} , attempt = {}", stage, attempt));
			// 1. Execute the current stage.
			let outcome = self
				.run_stage(stage, attempt, &mut pending_input)
				.await
				.with_context(|| format!("run_stage({stage:?})"))?;

			tracing::info!(
				stage = ?stage,
				attempt = attempt.number,
				">>> RUN STAGE COMPLETE"
			);

			// 2. Persist the execution and evaluation result.
			self.pipeline.persist_outcome(&outcome)?;

			// 3. Decide what should happen next.
			let decision = self.decide(&outcome).await?;

			tracing::info!(
				stage = ?stage,
				attempt = attempt.number,
				decision = ?decision,
				">>> DECISION"
			);

			// 4. Apply the decision.
			let control = self
				.apply(outcome, decision, input_rx, &mut pending_input)
				.await?;

			match control {
				RunControl::Continue => {
					// apply() transitioned to the next stage.
					stage = self.pipeline.stage();

					// Every new stage starts at attempt 1.
					self.pipeline.stage_attempt_reset();

					tracing::info!(
						stage = ?stage,
						attempt = self.pipeline.stage_attempt(),
						">>> TRANSITION"
					);
				}
				RunControl::RetryStage => {
					tracing::info!(
						stage = ?stage,
						">>> RETRY"
					);
					continue;
				}

				RunControl::Exit => {
					tracing::info!(
						stage = ?stage,
						attempt = attempt.number,
						">>> EXIT"
					);

					return Ok(());
				}
			}
		}
	}
}
impl SprintRunner<'_> {
	/// "Given this decision, what actions/state changes must happen?"
	///
	/// Applies the decision to the pipeline/session and returns control
	/// to the outer run loop.
	async fn apply(
		&mut self,
		outcome: StageOutcome,
		decision: StageDecision,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		let stage = outcome.stage();
		let attempt = outcome.attempt();
		match decision {
			StageDecision::Continue => {
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
			StageDecision::Retry => {
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

					self.pipeline.persist_progress(&format!(
						"{} exhausted maximum attempts ({})",
						stage, attempt.max
					))?;

					return Ok(RunControl::Exit);
				}

				let next_attempt = attempt.number + 1;

				self.pipeline.persist_progress(&format!(
					"Retrying stage {} (attempt {}/{})",
					stage, next_attempt, attempt.max,
				))?;

				self.pipeline.stage_attempt_increment();

				self.handle_retry(stage, attempt).await?;

				Ok(RunControl::RetryStage)
			}
			StageDecision::Revise => {
				self
					.pipeline
					.persist_progress(&format!("Revising stage {}", stage))?;
				// self
				// 	.handle_revision(stage, attempt, outcome, input_rx, pending_input)
				// 	.await?;
				Ok(RunControl::RetryStage)
			}
			StageDecision::AwaitHuman => {
				self.emit(SdlcEvent::PhaseChanged {
					phase: SdlcPhase::AwaitingHuman,
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
						self.pipeline.stage_attempt_increment();
						self.handle_retry(stage, attempt).await?;
						Ok(RunControl::RetryStage)
					}
					Intervention::Revision { evaluation } => {
						*pending_input = Some(SdlcInput::Revision { evaluation });
						self.pipeline.retry(stage)?;
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
							.pipeline
							.persist_progress(&format!("Human provided context: {context}"))?;
						Ok(RunControl::RetryStage)
					}
					Intervention::Abort => Ok(RunControl::Exit),
					Intervention::Human(_) => Ok(RunControl::Exit),
				}
			}
			StageDecision::Fail => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: format!("Stage {stage:?} failed on attempt {}", attempt.number + 1),
				});
				self
					.pipeline
					.persist_progress(&format!("{} failed", stage))?;
				Ok(RunControl::Exit)
			}
			StageDecision::Exit => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: format!("Stage {stage:?} exited on attempt {}", attempt.number + 1),
				});
				Ok(RunControl::Exit)
			}
			StageDecision::Complete => {
				self.emit(SdlcEvent::PhaseChanged {
					phase: SdlcPhase::Completed,
				});
				self
					.pipeline
					.persist_progress(&format!("{} completed", stage))?;
				self.emit(SdlcEvent::Completed);
				Ok(RunControl::Exit)
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
			phase: SdlcPhase::AwaitingHuman,
		});

		self.emit(SdlcEvent::Activity {
			stage,
			attempt,
			message: "Waiting for human input".into(),
		});

		let input = input_rx.recv().await.context("SDLC input channel closed")?;

		match input {
			SdlcInput::Retry => {
				self.pipeline.retry(stage)?;
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
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			SdlcInput::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			SdlcInput::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			SdlcInput::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	fn attempt_stage(&mut self, stage: Stage) -> Result<Attempt> {
		self.pipeline.next_attempt(stage)
	}
	pub async fn decide(&self, outcome: &StageOutcome) -> Result<StageDecision> {
		tracing::info!(">>> DECIDE");
		// tracing::info!(">>> outcome = {outcome:#?}");
		if std::env::var_os("SDLC_FORCE_EXIT").is_some() {
			tracing::warn!(">>> SDLC_FORCE_EXIT=1 -> Exit");
			return Ok(StageDecision::Exit);
		}

		if std::env::var_os("SDLC_FORCE_CONTINUE").is_some() {
			tracing::warn!(">>> SDLC_FORCE_CONTINUE=1 -> Continue");
			return Ok(StageDecision::Continue);
		}
		let decision = match outcome {
			StageOutcome::EvaluationFailed { execution, .. } => {
				let attempt = execution.attempt;
				tracing::info!(
					">>> EvaluationFailed: stage={:?} attempt={}/{}",
					attempt.stage,
					attempt.number,
					attempt.max,
				);
				if attempt.number < attempt.max {
					tracing::info!(">>> attempt {}/{} -> Retry", attempt.number, attempt.max,);
					StageDecision::Retry
				} else {
					tracing::info!(
						">>> attempt {}/{} exhausted -> AwaitHuman",
						attempt.number,
						attempt.max,
					);
					StageDecision::AwaitHuman
				}
			}
			StageOutcome::Complete { execution, .. } => {
				tracing::info!(
					">>> Complete: stage={:?} attempt={}/{}",
					execution.stage,
					execution.attempt.number,
					execution.attempt.max,
				);
				match &execution.result {
					StageResult::Verification(verification) => {
						if verification.passed {
							tracing::info!(">>> verification passed -> Continue");
							StageDecision::Continue
						} else {
							tracing::info!(">>> verification failed -> Retry");
							StageDecision::Retry
						}
					}
					_ if execution.stage == Stage::Complete => {
						tracing::info!(">>> final stage -> Complete");
						StageDecision::Complete
					}
					_ => {
						tracing::info!(">>> stage complete -> Continue");
						StageDecision::Continue
					}
				}
			}
			StageOutcome::NeedsRevision {
				execution,
				evaluation,
			} => {
				let attempt = execution.attempt;
				tracing::info!(
					">>> NeedsRevision: stage={:?} attempt={}/{} score={:.2} confidence={:.2}",
					execution.stage,
					attempt.number,
					attempt.max,
					evaluation.score,
					evaluation.confidence,
				);
				if attempt.number < attempt.max {
					tracing::info!(">>> attempt {}/{} -> Revise", attempt.number, attempt.max,);
					StageDecision::Revise
				} else {
					tracing::info!(
						">>> attempt {}/{} exhausted -> AwaitHuman",
						attempt.number,
						attempt.max,
					);

					StageDecision::AwaitHuman
				}
			}
			StageOutcome::ExecutionFailed { attempt, .. } => {
				tracing::info!(
					">>> ExecutionFailed: stage={:?} attempt={}/{}",
					attempt.stage,
					attempt.number,
					attempt.max,
				);
				if attempt.number < attempt.max {
					tracing::info!(">>> attempt {}/{} -> Retry", attempt.number, attempt.max,);
					StageDecision::Retry
				} else {
					tracing::info!(
						">>> attempt {}/{} exhausted -> AwaitHuman",
						attempt.number,
						attempt.max,
					);
					StageDecision::AwaitHuman
				}
			}
		};
		tracing::info!(">>> FINAL DECISION = {:?}", decision);
		Ok(decision)
	}
	fn emit(&self, event: SdlcEvent) {
		let _ = self.pipeline.event_tx.send(event);
	}
	fn evaluate_checks(&self, checks: Vec<CheckResult>) -> Result<Vec<CheckResult>> {
		todo!("evaluate_checks")
	}
	async fn evaluate(&self, execution: &StageExecution) -> Result<StageEvaluation> {
		let stage = execution.stage;
		let checks = self.pipeline.run_checks(stage).await?;
		let structural = self.evaluate_checks(checks)?;
		let semantic = self
			.pipeline
			.evaluator
			.evaluate(&self.pipeline.session, &execution)
			.await?;
		Ok(semantic.with_evaluations(structural))
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
			phase: SdlcPhase::Retrying,
		});

		self.pipeline.retry(stage)?;
		Ok(())
	}
	async fn handle_evaluation(
		&mut self,
		execution: StageExecution,
		attempt: Attempt,
	) -> Result<StageOutcome> {
		let stage = execution.stage;
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Evaluating,
		});
		self.emit(SdlcEvent::EvaluationStarted { stage });
		match self.pipeline.evaluate(&execution).await {
			Ok(evaluation) => {
				self.emit(SdlcEvent::Evaluated {
					stage,
					score: evaluation.score,
					confidence: evaluation.confidence,
					passed: evaluation.passed,
				});
				self.pipeline.persist_evaluation(&evaluation, attempt);
				if evaluation.passed {
					Ok(StageOutcome::Complete {
						execution,
						evaluation,
					})
				} else {
					Ok(StageOutcome::NeedsRevision {
						execution,
						evaluation,
					})
				}
			}
			Err(error) => Ok(StageOutcome::EvaluationFailed { execution, error }),
		}
	}
	async fn handle_revision(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		_outcome: StageOutcome,
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
				self.pipeline.retry(stage)?;
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input));
				self.pipeline.retry(stage)?;
			}
			Intervention::Retry => {
				self.pipeline.retry(stage)?;
			}
			Intervention::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.pipeline.retry(stage)?;
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
				self.pipeline.retry(stage)?;
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
				self.pipeline.retry(stage)?;
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
				phase: SdlcPhase::Retrying,
			});
			self.pipeline.retry(stage)?;
			return Ok(RunControl::Continue);
		}
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(stage, attempt, String::from("Execution Failure"), input_rx)
			.await?
		{
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.as_str().to_owned()));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: error.to_string(),
				});

				Ok(RunControl::Exit)
			}
			Intervention::Revise => {
				self.pipeline.retry(stage)?;

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
				phase: SdlcPhase::Retrying,
			});

			self.pipeline.retry(stage)?;
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
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Retry => {
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::ProvideContext(context) => {
				// If ProvideContext eventually becomes stage input,
				// this is where it should be stored.
				let _ = context;
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Reviewed => {
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});
				Ok(RunControl::Exit)
			}
			Intervention::Revise => {
				self.pipeline.retry(stage)?;
				Ok(RunControl::Continue)
			}
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });
				self.pipeline.retry(stage)?;
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
				phase: SdlcPhase::Retrying,
			});

			self.pipeline.retry(stage)?;

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
			Intervention::Revision { evaluation } => {
				*pending_input = Some(SdlcInput::Revision { evaluation });

				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.pipeline.retry(stage)?;

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
			Intervention::Revise => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}
		}
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
			e::Stage::Verify => Stage::Build,
			stage => stage,
		};
		self.pipeline.session.stage = stage;
		Ok(stage)
	}

	fn persist(&mut self, msg: &str) -> Result<()> {
		self.pipeline.persist_progress(msg)?;
		Ok(())
	}
	fn retry(&mut self, stage: Stage) -> Result<()> {
		self.pipeline.retry(stage)
	}
	async fn run_stage(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<StageOutcome> {
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Executing,
		});

		let input = match pending_input.take() {
			Some(SdlcInput::Revision { evaluation }) => StageInput::Revision { evaluation },
			Some(input) => {
				*pending_input = Some(input);
				StageInput::Initial
			}
			None => StageInput::Initial,
		};

		let execution = match self.run_current_stage(stage, attempt, input).await {
			Ok(execution) => execution,
			Err(error) => {
				return Ok(StageOutcome::ExecutionFailed {
					stage,
					attempt,
					error,
				});
			}
		};

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Evaluating,
		});
		self.handle_evaluation(execution, attempt).await
	}
	async fn run_current_stage(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		input: StageInput,
	) -> Result<StageExecution> {
		let time_started = Utc::now();

		let result = match stage {
			Stage::Intent => self.stage_intent(input).await?,
			Stage::Spec => self.stage_spec(input).await?,
			Stage::Plan => self.stage_plan(input).await?,
			Stage::Build => self.stage_build(input).await?,
			Stage::Verify => self.stage_verify(input).await?,
			Stage::Complete => StageResult::Complete,
			Stage::Finalize => StageResult::Finalize,
			Stage::Deploy | Stage::Maintain => {
				return Err(anyhow!("stage {stage:?} not implemented"));
			}
			_ => {
				todo!("run_current_stage")
			}
		};

		Ok(StageExecution {
			stage,
			attempt,
			time_started,
			time_completed: Utc::now(),
			result,
		})
	}
	async fn stage_intent(&mut self, input: StageInput) -> Result<StageResult> {
		let (stage, session_dir, goal) = {
			let session = &self.pipeline.session;
			(session.stage, session.dir.clone(), session.goal.clone())
		};
		if stage != Stage::Intent {
			return Err(anyhow!("cannot execute Intent stage while at {stage:?}"));
		}
		if goal.trim().is_empty() {
			return Err(anyhow!("SDLC session goal is empty"));
		}

		let prompt = p::gen_intent(&goal)?;

		std::fs::write("/tmp/estate-intent-prompt.md", &prompt)
			.context("writing Intent prompt debug file")?;

		if prompt.trim().is_empty() {
			return Err(anyhow!("generated Intent prompt is empty"));
		}

		let generated = self.pipeline.generator.generate(&prompt).await?;

		if generated.trim().is_empty() {
			return Err(anyhow!("generated Intent artifact is empty"));
		}

		SprintPipeline::write(session_dir.join("intent.md"), generated)?;
		self.pipeline.persist_progress("Intent stage completed")?;

		Ok(StageResult::Intent)
	}
	async fn stage_spec(&mut self, input: StageInput) -> Result<StageResult> {
		let (stage, session_dir) = {
			let session = &self.pipeline.session;
			(session.stage, session.dir.clone())
		};

		if stage != Stage::Spec {
			return Err(anyhow!("cannot execute Spec stage while at {stage:?}"));
		}

		let intent = self.pipeline.session_read("intent.md")?;

		if intent.trim().is_empty() {
			return Err(anyhow!("Intent artifact is empty"));
		}

		let is_revision = matches!(input, StageInput::Revision { .. });

		let prompt = match input {
			StageInput::Initial => p::gen_spec(&intent)?,
			StageInput::Revision { evaluation } => {
				let spec = self.pipeline.session_read("spec.md")?;
				p::revise_spec(&intent, &spec, &evaluation)?
			}
		};

		if prompt.trim().is_empty() {
			return Err(anyhow!("generated Spec prompt is empty"));
		}

		std::fs::write("/tmp/estate-spec-prompt.md", &prompt)
			.context("writing Spec prompt debug file")?;

		let generated = self.pipeline.generator.generate(&prompt).await?;

		if generated.trim().is_empty() {
			return Err(anyhow!("generated Spec artifact is empty"));
		}

		SprintPipeline::write(session_dir.join("spec.md"), generated)?;

		self.pipeline.persist_progress(if is_revision {
			"Spec revision completed"
		} else {
			"Spec stage completed"
		})?;

		Ok(StageResult::Spec)
	}
	async fn stage_plan(&mut self, input: StageInput) -> Result<StageResult> {
		let (stage, session_dir) = {
			let session = &self.pipeline.session;
			(session.stage, session.dir.clone())
		};
		if stage != Stage::Plan {
			return Err(anyhow!("cannot execute Plan stage while at {stage:?}"));
		}
		let intent = self.pipeline.session_read("intent.md")?;
		let spec = self.pipeline.session_read("spec.md")?;
		if intent.trim().is_empty() {
			return Err(anyhow!("Intent artifact is empty"));
		}
		if spec.trim().is_empty() {
			return Err(anyhow!("Spec artifact is empty"));
		}
		let prompt = p::gen_plan(&intent, &spec)?;
		if prompt.trim().is_empty() {
			return Err(anyhow!("generated Plan prompt is empty"));
		}
		// std::fs::write("/tmp/estate-plan-prompt.md", &prompt)
		// 	.context("writing Plan prompt debug file")?;
		let generated = self.pipeline.generator.generate(&prompt).await?;
		if generated.trim().is_empty() {
			return Err(anyhow!("generated Plan artifact is empty"));
		}
		SprintPipeline::write(session_dir.join("plan.md"), generated)?;
		self.pipeline.persist_progress("Plan stage completed")?;
		Ok(StageResult::Plan)
	}
	async fn stage_build(&mut self, input: StageInput) -> Result<StageResult> {
		let (stage, session_dir, workspace) = {
			let session = &self.pipeline.session;
			(
				session.stage,
				session.dir.clone(),
				session.workspace.clone(),
			)
		};
		if stage != Stage::Build {
			return Err(anyhow::anyhow!(
				"cannot execute Build stage while at {:?}",
				stage
			));
		}
		self.pipeline.persist_progress("Build started")?;
		let plan = tokio::fs::read_to_string(session_dir.join("plan.md"))
			.await
			.context("reading plan.md")?;
		let steps = build_steps();
		let workspace_before = WorkspaceSnapshot::capture(&workspace)?;
		for (index, instruction) in steps.iter().enumerate() {
			let step = index + 1;

			self.persist(&format!(
				"Build step {}/{}: {}",
				step,
				steps.len(),
				instruction
			))?;

			// Rebuild context before EVERY agent call.
			//
			// This is important because the workspace has changed since
			// the previous call.
			let current_workspace = WorkspaceSnapshot::capture(&workspace)?;

			let workspace_context = format!(
				"CWD: {}\n\n{}",
				workspace.display(),
				current_workspace.to_markdown()
			);

			let prompt = build_step_prompt(instruction, step, steps.len(), &plan, &workspace_context);
			let task = AgentTask::new(prompt);
			let result = self.pipeline.generator.run_agent(&task.prompt).await?;

			// Persist what happened during this build step.
			let step_path = session_dir.join(format!("build-step-{step:02}.md"));

			SprintPipeline::write(
				step_path,
				format!(
					"# Build Step {step}/{total}\n\n\
                 ## Task\n\n\
                 {instruction}\n\n\
                 ## Result\n\n\
                 {result:?}\n",
					total = steps.len(),
				),
			)?;

			// Give JEV / the next iteration a fresh view of the workspace.
			//
			// Don't carry the original workspace snapshot forward.
			// The agent just changed it.
			let after_step = WorkspaceSnapshot::capture(&workspace)?;

			self.pipeline.persist_progress(&format!(
				"Build step {}/{} completed: {} file(s) changed",
				step,
				steps.len(),
				after_step.diff(&workspace_before).file_count(),
			))?;
		}
		let workspace_after = WorkspaceSnapshot::capture(&workspace)?;
		let changes = workspace_before.diff(&workspace_after);
		SprintPipeline::write(
			session_dir.join("build.md"),
			changes.to_markdown("Build completed"),
		)?;
		self.pipeline.persist_progress(&format!(
			"Build completed: {} file(s) changed",
			changes.file_count()
		))?;
		Ok(StageResult::Build)
	}
	async fn stage_verify(&mut self, input: StageInput) -> Result<StageResult> {
		let verification = self.verify_stage(self.pipeline.stage()).await?;
		self.pipeline.persist_progress(&format!(
			"Verification completed: passed={}",
			verification.passed
		))?;
		Ok(StageResult::Verification(verification))
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

	fn transition(&mut self, next: Stage) -> Result<()> {
		self.pipeline.transition(next)
	}
	async fn verify_stage(&mut self, stage: Stage) -> Result<Verification> {
		let checks = self.pipeline.run_checks(stage).await?;
		let evaluations = self.pipeline.evaluate_checks(&checks);
		let passed = self.pipeline.is_passing(&checks, &evaluations);
		Ok(Verification {
			passed,
			checks,
			evaluations,
		})
	}
	async fn wait_for_intervention(
		&mut self,
		stage: Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self
			.pipeline
			.wait_for_intervention(stage, attempt, reason, input_rx)
			.await
	}
}
impl Step {
	pub fn stage(self) -> Option<Stage> {
		match self {
			Self::Intent => Some(Stage::Intent),
			Self::Spec => Some(Stage::Spec),
			Self::Plan => Some(Stage::Plan),
			Self::Build => Some(Stage::Build),
			Self::Verify => Some(Stage::Verify),
			Self::Boot | Self::Init | Self::Deploy | Self::Maintain | Self::Complete => None,
		}
	}
	pub const ALL: &'static [Self] = &[
		Self::Boot,
		Self::Init,
		Self::Intent,
		Self::Spec,
		Self::Plan,
		Self::Build,
		Self::Verify,
		Self::Deploy,
		Self::Maintain,
		Self::Complete,
	];
}

impl WorkspaceChanges {
	pub fn file_count(&self) -> usize {
		self
			.git_status
			.lines()
			.filter(|line| !line.trim().is_empty())
			.count()
	}
	pub fn to_markdown(&self, agent_result: &str) -> String {
		let mut markdown = String::from("# Build\n\n");
		markdown.push_str("## Workspace Changes\n\n");
		if self.git_status.trim().is_empty() {
			markdown.push_str("No Git changes detected.\n");
		} else {
			markdown.push_str("```text\n");
			markdown.push_str(&self.git_status);
			markdown.push_str("```\n");
		}
		markdown.push_str("\n## Agent Result\n\n");
		markdown.push_str(agent_result);
		markdown.push('\n');
		markdown
	}
}
impl WorkspaceSnapshot {
	pub fn capture(workspace: impl AsRef<Path>) -> Result<Self> {
		let workspace = workspace.as_ref();
		let output = Command::new("git")
			.args(["status", "--short", "--porcelain=v1"])
			.current_dir(workspace)
			.output()
			.with_context(|| format!("failed to capture git status in {}", workspace.display()))?;
		if !output.status.success() {
			return Err(anyhow::anyhow!(
				"git status failed: {}",
				String::from_utf8_lossy(&output.stderr).trim()
			));
		}
		Ok(Self {
			git_status: String::from_utf8(output.stdout)
				.context("git status output was not valid UTF-8")?,
		})
	}
	pub fn diff(&self, after: &Self) -> WorkspaceChanges {
		WorkspaceChanges {
			git_status: after.git_status.clone(),
		}
	}
	pub fn to_markdown(&self) -> String {
		format!(
			"## Workspace State\n\n\
             **Git status:**\n\n\
             ```text\n\
             {}\n\
             ```\n",
			self.git_status
		)
	}
}
