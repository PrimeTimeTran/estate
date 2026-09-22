fn _doc() {
	/// ## AI-Native SDLC playbook
	///
	/// A reimagined software development process that combines the control objectives
	/// of the traditional SDLC with automated enforcement and continuous feedback.
	///
	/// Instead of treating development as a linear sequence of handoffs, the
	/// AI-native SDLC treats it as a controlled loop. AI participates throughout
	/// the lifecycle, producing artifacts, executing work, evaluating evidence,
	/// and triggering the next stage when its exit criteria are satisfied.
	///
	/// The fundamental principle is:
	///
	/// ```text
	/// Human intent
	///      ↓
	///   intent.md     WHY
	///      ↓
	///    spec.md      WHAT
	///      ↓
	///    plan.md      HOW
	///      ↓
	/// Implementation
	///      ↓
	/// Tests / Evals / Checks
	///      ↓
	/// Verification
	///      │
	///      ├── fail ──→ Build → Verify
	///      │
	///      └── pass
	///           ↓
	///         Deploy
	///           ↓
	///        Maintain
	///           │
	///           └──── new problem / opportunity ────→ intent.md
	/// ```
	///
	/// The process therefore becomes a feedback system rather than a collection
	/// of disconnected development phases.
	///
	/// ### Artifacts
	///
	/// Each session produces a durable chain of artifacts connecting intent to
	/// implementation and evidence:
	///
	/// - `intent.md` — **why**
	///   - Human-defined goal, motivation, scope, constraints, and unknowns.
	/// - `spec.md` — **what**
	///   - Concrete requirements and behavioral/design specification derived from
	///     the intent.
	/// - `plan.md` — **how**
	///   - Ordered implementation strategy, affected files, dependencies, risks,
	///     alternatives, and verification strategy.
	/// - `code` — **implementation**
	///   - The source changes required to satisfy the specification.
	/// - `tests/evals` — **evidence**
	///   - Deterministic tests, builds, type checks, linting, browser checks,
	///     semantic evaluations, and other verification signals.
	/// - `progress.md` — **execution state**
	///   - What actually happened during the session, including deviations,
	///     failures, retries, discoveries, and verification results.
	///
	/// The artifacts form a chain of accountability:
	///
	/// ```text
	/// intent.md
	///    │
	///    ▼
	/// spec.md
	///    │
	///    ▼
	/// plan.md
	///    │
	///    ▼
	/// code
	///    │
	///    ▼
	/// evidence
	///    │
	///    ▼
	/// deployment
	/// ```
	///
	/// `progress.md` records the execution of that chain rather than replacing
	/// any of the artifacts above.
	///
	/// ### Session
	///
	/// A session is one complete attempt to move a unit of work from intent
	/// through deployment.
	///
	/// A session is stored under:
	///
	/// ```text
	/// ./log/sessions/26-09-21.intent-title/
	///     intent.md
	///     spec.md
	///     plan.md
	///     progress.md
	/// ```
	///
	/// The directory is the authoritative record of the session.
	///
	/// `./log/sessions.json` is an index of sessions and their outcomes. It should
	/// contain enough metadata to discover and summarize sessions without having
	/// to read every artifact.
	///
	/// A session must:
	///
	/// - Create a uniquely named directory using the format
	///   `YY-MM-DD.intent-title`.
	/// - Materialize the session templates from `./ai/template/`.
	/// - Adapt those templates to the specific session.
	/// - Maintain `progress.md` throughout execution.
	/// - Record the final outcome in `./log/sessions.json`.
	///
	/// The session index is an index, not the source of truth. The session
	/// directory contains the authoritative artifacts.
	///
	/// ### 1. Plan
	///
	/// Traditional development begins with requirements gathered by committees,
	/// workshops, and sign-offs and eventually written into a document.
	///
	/// The AI-native process begins with an explicit human intent that can be
	/// consumed by both humans and agents.
	///
	/// #### Outcome
	///
	/// `intent.md` aligns humans and AI on:
	///
	/// - Goal
	/// - Motivation
	/// - Scope
	/// - Out of scope
	/// - Constraints
	/// - Success criteria
	/// - Known unknowns
	///
	/// The stage must:
	///
	/// - Create a new directory in `./log/sessions/` using the session naming
	///   convention.
	/// - Create `intent.md` from `./ai/template/intent.md`.
	/// - Populate it with the intent specific to the session.
	/// - Establish the human-approved starting point for the session.
	///
	/// The agent may clarify or structure the intent, but the human remains the
	/// authority on what the system is intended to accomplish.
	///
	/// ### 2. Design
	///
	/// The accepted intent is transformed into an explicit specification.
	///
	/// #### Outcome
	///
	/// `spec.md` defines what must be true when the work is complete.
	///
	/// It should describe:
	///
	/// - Functional requirements
	/// - Behavioral requirements
	/// - Interfaces and constraints
	/// - Relevant architecture
	/// - Non-goals
	/// - Acceptance criteria
	///
	/// The stage must:
	///
	/// - Create `spec.md` from `./ai/template/spec.md`.
	/// - Derive the specification from `intent.md`.
	/// - Identify ambiguities before implementation begins.
	/// - Establish requirements that can later be verified.
	///
	/// The specification is the contract against which implementation is
	/// evaluated.
	///
	/// ### 3. Build
	///
	/// The accepted specification is converted into an implementation plan.
	///
	/// The agent then implements the plan while continuously validating local
	/// assumptions.
	///
	/// #### Outcome
	///
	/// - `plan.md` describing the implementation strategy.
	/// - Source changes implementing the plan.
	/// - Tests/evals covering the requirements.
	/// - Documentation updated where required.
	/// - `progress.md` recording actual execution.
	///
	/// `plan.md` should describe:
	///
	/// - Files and components expected to change.
	/// - Order of implementation.
	/// - Dependencies.
	/// - Testing strategy.
	/// - Risks.
	/// - Alternatives considered.
	/// - Expected verification signals.
	///
	/// Build is not complete merely because code exists. It is complete when
	/// there is a candidate implementation that can be submitted to verification.
	///
	/// The agent may discover information during implementation that invalidates
	/// the original plan. Such discoveries should be recorded in `progress.md`
	/// and may require the plan or specification to be revised before continuing.
	///
	/// ### 4. Verify
	///
	/// Verification is the control system of the SDLC.
	///
	/// The agent does not get to declare the session complete. The verification
	/// system determines whether the implementation satisfies the specification.
	///
	/// ```text
	///                 Build
	///                   │
	///                   ▼
	///             deterministic
	///                checks
	///                   │
	///                   ▼
	///             semantic evals
	///                   │
	///                   ▼
	///               evaluate
	///              ↙         ↘
	///           failure       pass
	///             │             │
	///             ▼             ▼
	///         return to       Deploy
	///           Build
	/// ```
	///
	/// Verification should combine multiple kinds of evidence.
	///
	/// #### Deterministic evidence
	///
	/// Evidence whose result can be established directly by software:
	///
	/// - Unit tests
	/// - Integration tests
	/// - Type checking
	/// - Linting
	/// - Formatting
	/// - Build success
	/// - Schema validation
	/// - Browser/E2E checks
	/// - Security checks
	/// - Artifact validation
	///
	/// These checks should remain deterministic wherever possible.
	///
	/// #### Semantic evidence
	///
	/// Some requirements cannot be reduced to simple assertions.
	///
	/// Examples include:
	///
	/// - Does the implementation actually satisfy the stated intent?
	/// - Does the behavior match the specification?
	/// - Is the resulting documentation understandable?
	/// - Does the implementation introduce an obvious requirement mismatch?
	///
	/// These can be evaluated using structured AI evaluation.
	///
	/// JEV-style evaluations belong here.
	///
	/// For example:
	///
	/// ```text
	/// intent.md
	///     │
	/// spec.md
	///     │
	/// implementation / diff
	///     │
	/// verification evidence
	///     │
	///     ▼
	///   JEV evaluation
	///     │
	///     ├── requirements satisfied?
	///     ├── intent aligned?
	///     ├── blocking issue?
	///     └── remaining risk?
	/// ```
	///
	/// JEV provides semantic evidence; it does not replace deterministic tests
	/// or human accountability.
	///
	/// #### Verification loop
	///
	/// When verification fails:
	///
	/// ```text
	/// Verify
	///   │
	///   ├── deterministic failure ──→ Build
	///   │
	///   ├── semantic failure ───────→ Build
	///   │
	///   └── human-required decision ─→ Human
	///                                  │
	///                                  ▼
	///                                Build
	/// ```
	///
	/// The loop continues until the required verification gates pass or a human
	/// explicitly intervenes.
	///
	/// #### Outcome
	///
	/// A successful verification produces an evidence record containing the
	/// results of the checks and evaluations.
	///
	/// For example:
	///
	/// ```text
	/// tests:             passed
	/// build:             passed
	/// typecheck:         passed
	/// lint:              passed
	/// intent_alignment:  evaluated
	/// requirements:      evaluated
	/// blocking_issue:    evaluated
	/// risk:              evaluated
	/// ```
	///
	/// Verification therefore answers:
	///
	/// > "What evidence do we have that the implementation satisfies the
	/// > specification?"
	///
	/// rather than:
	///
	/// > "Does the agent say that it is finished?"
	///
	/// ### 5. Deploy
	///
	/// Deployment moves a verified change from the development state into its
	/// intended environment.
	///
	/// Human review and governance remain explicit control points where required.
	/// The purpose of the automated lifecycle is not to remove human judgment,
	/// but to ensure that humans are reviewing a well-defined artifact with
	/// reproducible evidence.
	///
	/// #### Outcome
	///
	/// - Verified implementation is deployed.
	/// - Deployment metadata is recorded.
	/// - `./log/sessions.json` is updated with the session outcome.
	/// - A `./doc/.changeset` entry is created when the project requires one.
	/// - The deployed commit/version/artifact is associated with the session.
	///
	/// Administrative bookkeeping should be generated from the session rather
	/// than reconstructed manually after deployment.
	///
	/// ### 6. Maintain
	///
	/// Maintenance closes the lifecycle loop.
	///
	/// Production is observed for:
	///
	/// - Bugs
	/// - Regressions
	/// - Operational failures
	/// - Security issues
	/// - New requirements
	/// - User feedback
	/// - Opportunities for improvement
	///
	/// A production signal does not merely create a bug ticket. It can become
	/// the input to a new SDLC session.
	///
	/// ```text
	///              Deploy
	///                 │
	///                 ▼
	///              Maintain
	///                 │
	///        ┌────────┴────────┐
	///        │                 │
	///     healthy          new signal
	///        │                 │
	///        │                 ▼
	///        │             new intent
	///        │                 │
	///        └─────────────────┘
	///                  │
	///                  ▼
	///               Plan
	/// ```
	///
	/// #### Outcome
	///
	/// - Production state is observed.
	/// - Issues and opportunities are captured.
	/// - Significant new work produces a new `intent.md`.
	/// - The new intent begins another session.
	///
	/// Git commits are part of preserving and identifying the resulting state,
	/// but committing code is not the purpose of Maintain. Maintain exists to
	/// close the feedback loop between the deployed system and future intent.
	///
	/// ## Control model
	///
	/// The AI-native SDLC separates three different forms of authority:
	///
	/// ```text
	/// Human judgment
	///      │
	///      │ defines intent / approves decisions
	///      ▼
	/// Agent execution
	///      │
	///      │ implements and iterates
	///      ▼
	/// Verification system
	///      │
	///      │ produces evidence
	///      ▼
	/// Deployment gate
	/// ```
	///
	/// These should not be conflated.
	///
	/// - **Human** decides what should be built and where judgment is required.
	/// - **Agent** performs the implementation work.
	/// - **Deterministic tooling** establishes facts about the implementation.
	/// - **AI evaluation** provides structured semantic evidence where deterministic
	///   checks are insufficient.
	/// - **Verification** determines whether the required evidence has been
	///   satisfied.
	/// - **Deployment** occurs only after the required gates pass.
	///
	/// The central design principle is therefore:
	///
	/// > AI performs the work, artifacts preserve the reasoning, verification
	/// > establishes evidence, and humans retain authority over intent and
	/// > consequential decisions.
	///
	/// ## Reference
	///
	/// - https://claude.com/blog/the-ai-native-sdlc-playbook
	let foo = 1;
	//  ┌─────────────── SESSION ────────────────┐
	//  │                                        │
	//  │ intent → spec → plan → implementation  │
	//  │                         │               │
	//  │                         ▼               │
	//  │              deterministic checks      │
	//  │                         │               │
	//  │                         ▼               │
	//  │                    JEV eval            │
	//  │                         │               │
	//  │                 ┌───────┴───────┐      │
	//  │                 │               │      │
	//  │               FAIL            PASS     │
	//  │                 │               │      │
	//  │                 └──→ BUILD      │      │
	//  │                                 ▼      │
	//  │                              DEPLOY    │
	//  │                                 │      │
	//  └─────────────────────────────────┼──────┘
	//                                    │
	//                                    ▼
	//                                MAINTAIN
	//                                    │
	//                                    ▼
	//                                 INTENT
}

use crate::{
	model::{
		AgentTask,
		agent::{Agent, AgentContext},
		resolver::workspace_cargo_path,
	},
	prelude::*,
};
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum GenerationProvider {
	Local,
	Api,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageStatus {
	Running,
	Completed,
	Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageActor {
	Human,
	Sdlc,
	Agent,
}

pub fn prompt_for_intent() -> Result<String> {
	Ok(String::from(
		// "Finish SDLC Module which creates a loop for my SDLC. ",
		"Create a file named hello-world.js in the repository root. It should accept a command-line argument and write that value to hello-world.md. The user should be able to run node hello-world.js \"hi\". Add tests covering both the JavaScript logic and the CLI behavior.",
	))
}
fn output_status(_output: &str) -> std::process::ExitStatus {
	// placeholder
	unimplemented!()
}

#[async_trait::async_trait]
pub trait ArtifactGenerator: Send + Sync {
	async fn generate(&self, prompt: &str) -> Result<String>;
}
trait TextModel {
	async fn generate(&self, prompt: &str) -> Result<String>;
}

#[async_trait]
impl ArtifactGenerator for LocalGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		// println!("=== GENERATOR START ===");
		// println!("{prompt}");
		let task = AgentTask::new(prompt.to_string());
		let (event_tx, _) = tokio::sync::mpsc::unbounded_channel();
		let result = self.agent.run_agent_loop(task, event_tx).await?;
		// println!("=== AGENT RESULT ===");
		// println!("status: {:?}", result.status);
		// println!("chat: {:?}", result.chat);
		// println!("summary: {:?}", result.summary);
		// println!("artifacts: {:?}", result.artifacts);
		// println!("logs: {:?}", result.logs);
		// println!("=====================");
		Err(anyhow::anyhow!(
			"debug: Agent completed but no generation result was extracted"
		))
	}
}
#[async_trait]
impl ArtifactGenerator for ApiGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		// Call your API here.
		//
		// Return the generated markdown.
		todo!()
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

		let intent = std::fs::read_to_string(session.join("intent.md"))?;
		let spec = std::fs::read_to_string(session.join("spec.md"))?;
		let tests = std::fs::read_to_string(session.join("tests.md"))?;

		// This should eventually come from the actual deterministic verifier.
		// For now, `verification.md` can contain the commands that were run and
		// their results.
		let evidence = match std::fs::read_to_string(session.join("verification.md")) {
			Ok(evidence) => evidence,
			Err(_) => String::from("No verification evidence was recorded."),
		};

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

	// async fn evaluate_verification(&self, session: &PathBufrust) -> Result<StageEvaluation> {
	// 	todo!("evaluate_verification")
	// }
}
impl Sdlc {
	/// Path to the globally active SDLC session.
	///
	/// During development this lives inside the project's log directory
	/// so the current lifecycle state is easy to inspect.
	///
	/// Example:
	///
	/// crates/estate/log/tmp/sdlc.current.json
	///
	fn current_path() -> PathBuf {
		Self::root()
			.join("log")
			.join("tmp")
			.join("sdlc.current.json")
	}

	/// Load the current lifecycle, if one exists.
	pub fn load(jev: TypeSafeClient) -> Result<Option<Self>> {
		let state_path = Self::current_path();

		let evaluator = Evaluator { jev };

		if !state_path.exists() {
			return Ok(Some(Self {
				evaluator,
				generator: Box::new(LocalGenerator {
					agent: Agent::new(),
				}),
				state_path,
				session: None,
			}));
		}

		let contents = std::fs::read_to_string(&state_path)?;
		let session = serde_json::from_str(&contents)?;

		Ok(Some(Self {
			evaluator,
			generator: Box::new(LocalGenerator {
				agent: Agent::new(),
			}),
			state_path,
			session: Some(session),
		}))
	}

	/// Start a new lifecycle session from user-provided intent.
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

		self.initialize_templates(
			self
				.session()?
				.ok_or_else(|| anyhow::anyhow!("failed to create session"))?
				.dir
				.as_path(),
		)?;

		// The user's original intent is authoritative.
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

	/// Resume the currently active lifecycle.
	///
	/// State is already loaded from `sdlc.current.json`, so there is
	/// intentionally little to do here for now.
	async fn resume(&mut self) -> Result<()> {
		if self.session.is_none() {
			return Err(anyhow::anyhow!("no active SDLC session to resume"));
		}

		self.update_progress("SDLC session resumed")?;
		self.persist()?;

		Ok(())
	}

	/// Return the current lifecycle stage.
	pub fn stage(&self) -> Option<Stage> {
		self.session.as_ref().map(|s| s.stage.clone())
	}

	/// Return the active session.
	pub fn session(&self) -> Result<Option<&SdlcSession>> {
		Ok(self.session.as_ref())
	}

	// ─────────────────────────────────────────────────────────────────────
	// Stages
	// ─────────────────────────────────────────────────────────────────────
	pub async fn stage_intent(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
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
	pub async fn stage_spec(&mut self) -> Result<()> {
		let (stage, session_dir) = {
			let session = self
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

			(session.stage.clone(), session.dir.clone())
		};

		if stage != Stage::Spec {
			return Err(anyhow::anyhow!(
				"cannot execute Spec stage while at {:?}",
				stage
			));
		}

		let intent = self.read("intent.md")?;

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
	pub async fn stage_plan(&mut self) -> Result<()> {
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
		let intent = self.read("intent.md")?;
		let spec = self.read("spec.md")?;
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

	async fn generate_plan(&self, intent: &str, spec: &str) -> Result<String> {
		let prompt = format!(
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
		);

		self.generator.generate(&prompt).await
	}
	async fn generate_tests(&self, intent: &str, spec: &str, plan: &str) -> Result<String> {
		let prompt = format!(
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
		);

		self.generator.generate(&prompt).await
	}
	/// Execute the Build stage.
	///
	/// Produces:
	///
	///     source changes
	///     tests
	///     documentation
	///
	/// The agent performs implementation work here.
	pub async fn stage_build(&mut self) -> Result<()> {
		let session = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let context = AgentContext::from_session(session)?;
		self.update_progress("Build started")?;
		let task = context.task.clone();
		// Eventually:
		//
		// let result = self.agent_runtime.run_agent(task).await?;
		self.transition(Stage::Verify)?;
		Ok(())
	}
	/// Execute the Verify stage.
	///
	/// Runs deterministic checks and JEV evaluations.
	///
	/// Failure returns the lifecycle to Build.
	pub async fn verify(&mut self) -> Result<Verification> {
		self.update_progress("Verification started")?;
		let checks = self.run_checks().await?;
		let evaluations = self.evaluate(&checks).await?;
		let verification = Verification {
			passed: self.verification_passed(&checks, &evaluations),
			checks,
			evaluations,
		};
		// Persist the evidence somewhere.
		self.update_progress(&format!(
			"Verification completed: passed={}",
			verification.passed
		))?;

		Ok(verification)
	}

	/// Execute the Deploy stage.
	///
	/// Only allowed after successful verification.
	pub async fn stage_deploy(&mut self) -> Result<()> {
		todo!("sdlc deploy")
	}

	/// Execute the Maintain stage.
	///
	/// Records the deployed state and determines whether a new lifecycle
	/// session should be created.
	pub async fn stage_maintain(&mut self) -> Result<()> {
		todo!("sdlc maintain")
	}

	// ─────────────────────────────────────────────────────────────────────
	// State transitions
	// ─────────────────────────────────────────────────────────────────────

	/// Advance the lifecycle to the next stage.
	///
	/// This is the only method allowed to mutate the lifecycle stage.
	pub fn transition(&mut self, next: Stage) -> Result<()> {
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

	fn commit(&mut self) -> Result<()> {
		Ok(())
	}

	/// Persist the current lifecycle state.
	///
	/// Writes:
	///
	///     ./log/tmp/sdlc.current.json
	///
	/// This file is only the recovery cursor for the currently active session.
	/// The actual lifecycle artifacts live in the session directory.
	fn persist(&self) -> Result<()> {
		let parent = self
			.state_path
			.parent()
			.ok_or_else(|| anyhow::anyhow!("invalid SDLC state path"))?;
		std::fs::create_dir_all(parent)?;
		let contents = serde_json::to_string_pretty(&self.session)?;
		std::fs::write(&self.state_path, contents)?;

		Ok(())
	}

	/// Remove the global active-session marker.
	///
	/// Called after the lifecycle reaches a terminal state.
	pub fn clear_current(&self) -> Result<()> {
		if self.state_path.exists() {
			std::fs::remove_file(&self.state_path)?;
		}

		Ok(())
	}

	// ─────────────────────────────────────────────────────────────────────
	// Artifacts
	// ─────────────────────────────────────────────────────────────────────

	fn read(&self, name: &str) -> Result<String> {
		// self.session;
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		read_from_session(name, session)
	}
	fn write(path: PathBuf, contents: String) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}

	async fn evaluate_stage(&self, stage: Stage) -> Result<StageEvaluation> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let started_at = Utc::now();
		let evaluation = match stage {
			Stage::Intent => {
				let intent = self.read("intent.md")?;
				self.evaluator.evaluate_intent(&intent).await?
			}
			Stage::Spec => {
				let intent = self.read("intent.md")?;
				let spec = self.read("spec.md")?;
				self.evaluator.evaluate_spec(&intent, &spec).await?
			}
			Stage::Plan => {
				let intent = self.read("intent.md")?;
				let spec = self.read("spec.md")?;
				let plan = self.read("plan.md")?;
				let tests = self.read("tests.md")?;

				self
					.evaluator
					.evaluate_plan(&intent, &spec, &plan, &tests)
					.await?
			}
			Stage::Build => {
				let intent = self.read("intent.md")?;
				let spec = self.read("spec.md")?;
				let plan = self.read("plan.md")?;
				let tests = self.read("tests.md")?;

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
	fn record_evaluation(&mut self, evaluation: StageEvaluation) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let record = StageRecord {
			stage: evaluation.stage.clone(),
			status: StageStatus::Completed,
			actor: evaluation.actor.clone(),
			started_at: evaluation.started_at,
			completed_at: Some(Utc::now()),
			evaluation: Some(evaluation),
		};
		session.stages.push(record);
		session.updated_at = Utc::now();
		self.persist()
	}
	fn root() -> PathBuf {
		// PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		workspace_cargo_path()
	}
	/// Return the directory containing the current session's artifacts.
	pub fn dir(&self) -> Result<&Path> {
		self
			.session
			.as_ref()
			.map(|session| session.dir.as_path())
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))
	}

	/// Create a session directory.
	///
	/// Sessions live under:
	///
	///     crates/estate/log/session/
	///
	/// Templates live under:
	///
	///     crates/estate/ai/template/
	pub fn create_dir(&self, title: &str) -> Result<PathBuf> {
		let root = Self::root().join("log").join("session");
		std::fs::create_dir_all(&root)?;
		let date = Local::now().format("%Y-%m-%d");
		let dir = root.join(format!("{date}.{title}"));
		std::fs::create_dir_all(&dir)?;
		Ok(dir)
	}

	/// Materialize the template files for a new session.
	fn initialize_templates(&self, dir: &Path) -> Result<()> {
		let template_dir = Self::root().join("ai").join("template");

		for name in ["intent.md", "spec.md", "plan.md", "progress.md"] {
			let source = template_dir.join(name);
			let destination = dir.join(name);
			if source.exists() {
				std::fs::copy(source, destination)?;
			} else {
				Self::write(destination, format!("# {}\n\n", name));

				// std::fs::write(destination, format!("# {}\n\n", name))?;
			}
		}
		Ok(())
	}

	/// Update progress.md with the current lifecycle state.
	///
	/// The progress journal lives inside the active session directory:
	///
	///     crates/estate/log/session/<session>/progress.md
	fn update_progress(&self, message: &str) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		let path = session.dir.join("progress.md");

		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

		let entry = format!("\n## {}\n\n{}\n", timestamp, message,);

		let mut file = std::fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)?;

		file.write_all(entry.as_bytes())?;

		Ok(())
	}

	/// Record the completed session in the SDLC index.
	///
	/// The authoritative artifacts remain in:
	///
	///     crates/estate/log/session/<session>/
	///
	/// This file is only an index:
	///
	///     crates/estate/log/sessions.json
	fn record_session(&self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let log_dir = Self::root().join("log");
		std::fs::create_dir_all(&log_dir)?;
		let index_path = log_dir.join("sessions.json");
		let mut sessions: Vec<SdlcSession> = if index_path.exists() {
			let contents = std::fs::read_to_string(&index_path)?;

			if contents.trim().is_empty() {
				Vec::new()
			} else {
				serde_json::from_str(&contents)?
			}
		} else {
			Vec::new()
		};
		// Replace an existing entry for this session rather than
		// creating duplicate index entries.
		sessions.retain(|existing| existing.id != session.id);
		sessions.push(session.clone());
		let contents = serde_json::to_string_pretty(&sessions)?;
		Self::write(index_path, contents);
		Ok(())
	}

	pub async fn run(&mut self) -> Result<()> {
		loop {
			match self.stage() {
				// None => self.start(...).await?,
				Some(Stage::Intent) => self.stage_intent().await?,
				Some(Stage::Spec) => self.stage_spec().await?,
				Some(Stage::Plan) => self.stage_plan().await?,
				Some(Stage::Build) => self.stage_build().await?,

				Some(Stage::Verify) => {
					match self.verify().await? {
						verification if verification.passed => {
							self.commit()?;
							self.transition(Stage::Complete)?;
						}
						_ => {
							self.transition(Stage::Build)?;
						}
					}
					// let verification = self.verify().await?;

					// if verification.passed {
					// 	self.commit().await?;
					// 	self.transition(Stage::Complete)?;
					// } else {
					// 	self.transition(Stage::Build)?;
					// }
				}

				Some(Stage::Complete) => {
					self.clear_current()?;
					// break;
					return Ok(());
				}

				Some(Stage::Deploy | Stage::Maintain) => {
					todo!()
				}
				_ => return Ok(()),
			}
		}
	}

	// ─────────────────────────────────────────────────────────────────────
	// Verification
	// ─────────────────────────────────────────────────────────────────────
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
	async fn evaluate(&self, checks: &[CheckResult]) -> Result<Vec<EvaluationResult>> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		let intent = self.read("intent.md")?;
		let spec = self.read("spec.md");
		let plan = self.read("plan.md");
		let progress = self.read("progress.md");
		Ok(vec![])
	}

	/// Determine whether all verification gates have passed.
	fn verification_passed(&self, checks: &[CheckResult], evaluations: &[EvaluationResult]) -> bool {
		checks.iter().all(|check| check.passed)
			&& evaluations.iter().all(|evaluation| evaluation.passed)
	}
}
impl SdlcSession {
	fn created_at_readable(&self) -> String {
		self
			.created_at
			.format("%B %-d, %Y at %-I:%M:%S %p UTC")
			.to_string()
	}
	fn updated_at_readable(&self) -> String {
		self
			.updated_at
			.format("%B %-d, %Y at %-I:%M:%S %p UTC")
			.to_string()
	}
	fn create_readable(&self) -> String {
		let current = Utc::now();
		current.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()
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
struct CommandResult {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
	pub name: String,
	pub score: f64,
	pub confidence: f64,
}
/// Persistent state for the lifecycle runner.
///
/// The global location means there can be one active lifecycle at a time,
/// regardless of the current working directory.
pub struct Sdlc {
	state_path: PathBuf,
	session: Option<SdlcSession>,
	// jev: TypeSafeClient,
	evaluator: Evaluator,
	generator: Box<dyn ArtifactGenerator>,
}
/// The persistent state of an active SDLC session.
///
/// This file is intentionally small. It is the machine-readable state of the
/// lifecycle; the actual artifacts live in the session directory.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
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
/// Result of verification.
///
/// Deterministic checks and JEV evaluations should eventually be represented
/// separately here. A session should only advance when its required gates pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	pub checks: Vec<CheckResult>,
	pub evaluations: Vec<EvaluationResult>,
}
