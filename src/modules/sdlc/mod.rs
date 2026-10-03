use crate::{
	agent_event::RuntimeEvent,
	model::{
		AgentTask,
		agent::{Agent, AgentContext},
		resolver::*,
		task::TaskResult,
	},
	prelude::{structs as ext_structs, *},
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
pub use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	widgets::Clear,
};
use std::{io::Stdout, process::Command};
use tokio::time::{Duration, sleep};
use tracing::debug;
// cmd+alt+f
// - Search in all files overlay
// cmd+shift+f
// - Search in all files tab
// cmd+f
// - Search in file
fn git_status() -> Result<String> {
	let output = std::process::Command::new("git")
		.args(["status", "--short"])
		.output()?;
	if !output.status.success() {
		return Err(anyhow::anyhow!(
			"git status failed: {}",
			String::from_utf8_lossy(&output.stderr)
		));
	}
	Ok(String::from_utf8(output.stdout)?)
}
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
fn slugify(input: &str) -> String {
	let slug = input
		.to_lowercase()
		.chars()
		.map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
		.collect::<String>();
	let slug = slug
		.split('-')
		.filter(|s| !s.is_empty())
		.take(8)
		.collect::<Vec<_>>()
		.join("-");
	if slug.is_empty() {
		return "untitled".into();
	}
	// Windows reserved device names.
	let reserved = [
		"con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
		"com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
	];
	if reserved.contains(&slug.as_str()) {
		format!("session-{slug}")
	} else {
		slug
	}
}
fn persist_evaluation(session: &mut SdlcSession, evaluation: &StageEvaluation) -> Result<()> {
	let status = if evaluation.passed {
		StageStatus::Completed
	} else {
		StageStatus::NeedsRevision
	};

	let record = StageRecord {
		description: Some(String::from("Evaluation done")),
		actor: evaluation.actor.clone(),
		attempt: Attempt::new(),
		evaluation: Some(evaluation.clone()),
		stage: evaluation.stage,
		time_started: evaluation.time_started,
		time_completed: Some(Utc::now()),
		status,
	};
	let time_started = evaluation.time_started;
	let time_completed = Utc::now();
	// let record = StageRecord {
	// 	stage: evaluation.stage,
	// 	attempt: Attempt::new(),
	// 	status,
	// 	actor: evaluation.actor.clone(),
	// 	description: Some("Evaluation done".into()),
	//
	// 	time_started: time_readable(time_started),
	// 	time_completed: Some(time_readable(time_completed)),
	// 	time_total: duration_readable(time_completed - time_started),
	//
	// 	evaluation: Some(evaluation.clone()),
	// };
	session.stages.push(record);
	session.time_updated = Utc::now();

	Ok(())
}
fn time_readable(time: chrono::DateTime<chrono::Utc>) -> String {
	time.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()
}
fn short_duration_readable(duration: chrono::Duration) -> String {
	let seconds = duration.num_seconds();
	let hours = seconds / 3600;
	let minutes = (seconds % 3600) / 60;
	let seconds = seconds % 60;

	match (hours, minutes, seconds) {
		(h, m, _) if h > 0 => format!("{h} hours {m} minutes"),
		(_, m, s) if m > 0 => format!("{m} minutes {s} seconds"),
		(_, _, s) => format!("{s} seconds"),
	}
}
fn duration_readable(duration: chrono::Duration) -> String {
	let millis = duration.num_milliseconds();
	if millis < 1000 {
		return format!("{millis} ms");
	}

	let seconds = millis / 1000;
	let hours = seconds / 3600;
	let minutes = (seconds % 3600) / 60;
	let seconds = seconds % 60;

	match (hours, minutes, seconds) {
		(h, m, _) if h > 0 => format!("{h}h {m}m"),
		(_, m, s) if m > 0 => format!("{m}m {s}s"),
		(_, _, s) => format!("{s}s"),
	}
}
fn log_step_transition(from: Stage, to: Stage) -> Result<()> {
	let path: PathBuf = env::current_dir()?.join("current_step.txt");
	let mut file = OpenOptions::new().create(true).append(true).open(path)?;
	writeln!(file, "{:?} -> {:?}", from, to)?;
	Ok(())
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

mod constants {
	const TODO: &'static str = r#"
		- Question prompt
		- Add progressive disclosure
	"#;
	use super::*;
	pub const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(1);
	pub const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(1);
	pub const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(1);
	pub const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
	pub const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
	pub const MAX_STAGE_ATTEMPTS: u32 = 1;
	pub const STATUS_INTERVAL: Duration = Duration::from_secs(30);
}
pub use constants::*;

mod enums {
	use super::structs::*;
	use super::*;
	#[derive(Debug)]
	pub enum ExecutionResult {
		Completed(TaskResult),
		Failed(StageError),
	}

	#[derive(Debug, Clone)]
	pub enum Intervention {
		Human(String),
		Retry,
		ProvideContext(String),
		Reviewed,
		Abort,
		Revise,
	}
	#[derive(Debug, Clone, Copy)]
	pub enum GenerationProvider {
		Local,
		Api,
	}
	#[derive(Debug, Clone)]
	pub enum SdlcEvent {
		Activity {
			stage: Stage,
			attempt: Attempt,
			message: String,
		},
		Completed,
		Failed {
			stage: Option<Stage>,
			error: String,
		},
		InterventionRequired {
			stage: Stage,
			attempt: Attempt,
			reason: String,
		},
		InterventionResolved {
			stage: Stage,
			action: String,
		},
		Evaluated {
			stage: Stage,
			score: f64,
			confidence: f64,
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
			attempt: Attempt,
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
			number: u32,
		},
		StageStarted {
			stage: Stage,
			attempt: Attempt,
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
		Test,
		Build,
		Verify,
		Deploy,
		Maintain,
		Complete,
		Finalize,
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
	#[derive(Debug, Clone, Copy)]
	pub enum StageAction {
		Continue,
		Retry,
		Intervene,
		Abort,
	}
	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub enum StageActor {
		Human,
		Sdlc,
		Evaluator,
		Agent,
		System,
	}
	#[derive(Debug)]
	pub enum StageDecision {
		Continue,
		Retry,
		Revise,
		AwaitHuman,
		Fail,
		Complete,
	}
	#[derive(Debug)]
	pub enum StageOutcome {
		Complete {
			execution: StageExecution,
			evaluation: StageEvaluation,
		},
		NeedsRevision {
			execution: StageExecution,
			evaluation: StageEvaluation,
		},
		ExecutionFailed {
			stage: e::Stage,
			attempt: Attempt,
			error: anyhow::Error,
		},

		EvaluationFailed {
			execution: StageExecution,
			error: anyhow::Error,
		},
	}
	impl StageOutcome {
		pub fn stage(&self) -> Stage {
			match self {
				Self::Complete { execution, .. }
				| Self::NeedsRevision { execution, .. }
				| Self::EvaluationFailed { execution, .. } => execution.stage,

				Self::ExecutionFailed { stage, .. } => *stage,
			}
		}
		pub fn attempt(&self) -> Attempt {
			match self {
				Self::Complete { execution, .. }
				| Self::NeedsRevision { execution, .. }
				| Self::EvaluationFailed { execution, .. } => execution.attempt,

				Self::ExecutionFailed { attempt, .. } => *attempt,
			}
		}
	}
	pub enum StageOutcomeEvaluation {
		Passed(Evaluation),
		FailedQuality(Evaluation),
		FailedRuntime(Error),
	}
	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub enum StageStatus {
		Running,
		Completed,
		Failed,
		NeedsRevision,
		EvaluationFailed,
		InterventionNeeded,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Step {
		Boot,
		Init,
		Intent,
		Spec,
		Plan,
		Build,
		Verify,
		Deploy,
		Maintain,
		Complete,
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
	pub enum InputMode {
		Normal,
		Human { buffer: String },
		AwaitingHuman { prompt: String },
	}
	#[derive(Debug)]
	pub enum RunControl {
		Continue,
		Exit,
	}

	pub enum RunState {
		Idle,
		Running,
		Paused,
		AwaitingInput,
		Completed,
		Failed,
		Cancelled,
	}
	pub enum RunResult {
		Completed,
		Failed,
		Cancelled,
	}
	pub type PipelineId = uuid::Uuid;
	pub struct PipelineState<S> {
		stage: S,
		attempt: u32,
		status: PipelineStatus,
	}
	pub struct PipelineStatus;
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageResult {
		Intent,
		Spec,
		Plan,
		Build,
		Verification(Verification),
		Complete,
		Finalize,
	}
	pub enum StageTransition {
		Next,
		Repeat,
		Goto(Stage),
		Complete,
		Fail,
		AwaitHuman,
	}
}
pub use e::*;
use enums as e;
pub use prompt as agent_prompts;
use prompt::*;
pub mod prompt {
	use super::*;
	const INTENT_PROMPT: &str = include_str!("../../../ai/template/INITIAL_PROMPT.md");
	const PROMPT_FROM_USER: &str = include_str!("../../../ai/template/user.goal.md");
	pub fn for_intent(user_request: &str) -> String {
		INTENT_PROMPT.replace("{{PROMPT_FROM_USER}}", user_request)
	}
	pub fn gen_intent(goal: &str) -> Result<String> {
		Ok(format!(
			r#"
				You are defining the intent for an SDLC task.
				The user's goal is authoritative.

				## User Goal
				{goal}

				## Instructions
				Create `intent.md`.
				Describe what the user is trying to accomplish and why.

				The intent should:
				- preserve the user's actual goal without changing its meaning
				- describe the desired outcome
				- establish the problem or need being addressed
				- identify the important constraints explicitly stated by the user
				- avoid inventing requirements that the user did not state
				- remain implementation-independent where possible

				Do not write the specification, implementation plan, or tests yet.

				Use this format:

				# Intent <one sentence title summary for the goal>

				## Goal

				<what the user wants to accomplish>

				## Why

				<why this work is needed>

				## Constraints

				- <constraint>

				## Outcome

				<what successful completion should accomplish>

				Return only the contents of `intent.md`.
				"#,
		))
	}
	pub fn gen_spec(intent: &str) -> Result<String> {
		if intent.trim().is_empty() {
			return Err(anyhow!("cannot generate Spec prompt from empty Intent"));
		}

		Ok(format!(
			r#"
       	You are the Specification stage of an SDLC pipeline.

       	Your job is to transform the approved Intent artifact below into a concrete,
       	implementation-independent Specification.

       	You are NOT implementing the feature.
       	You are NOT writing source code.
       	You are NOT creating a plan.
       	You are NOT merely summarizing the Intent.

       	You are defining WHAT must be built, the boundaries of the work, the
       	constraints that apply, the expected system behavior, and the important
       	architectural concerns that must be resolved before implementation.

       	The resulting document will be written directly to:
          spec.md

       	Therefore, your entire response MUST be the specification document itself.
       	Do not include commentary before or after the specification.
       	Do not wrap the document in a Markdown code fence.

       	The Specification MUST use exactly this structure:

       	# Specification: [Feature or Project Name]

       	## 1. Overview & Inherited Intent

       	- **Source Intent:** intent.md
       	- **Core Objective:** [Brief summary of what this specification builds,
          explicitly inheriting the approved outcome from the Intent]

       	## 2. Requirements & Functional Scope

       	- **In-Scope:**
          - [Core capability 1]
          - [Core capability 2]

       	- **Out-of-Scope:**
          - [Explicit boundary / what is deferred]

       	Requirements must describe observable or verifiable behavior where possible.
       	Do not invent requirements that contradict the Intent.
       	If the Intent leaves something unspecified, identify that uncertainty rather
       	than silently inventing a product decision.

       	## 3. Policy & Governance Constraints (Applied Skills)

       	- **Brand & UX Guidelines:** [Applicable constraints, or "None identified"]
       	- **Security & Compliance:** [Applicable data handling, access control,
          privacy, security, and boundary constraints, or "None identified"]

       	Do not invent organizational policies.
       	Only state constraints supported by the Intent, existing project context,
       	or explicitly applicable system/project rules.

       	## 4. Proposed Design & Architecture

       	- **System Impact:** [Affected components, modules, services, files,
          persistence, APIs, integrations, or runtime boundaries]

       	- **User Experience Flow:** [Expected user-visible behavior and interaction
          flow, if applicable]

       	Describe the proposed system behavior and architecture at the level needed
       	for implementation to begin later.

       	Do NOT write implementation code.
       	Do NOT turn this section into an implementation plan.
       	Do NOT prescribe arbitrary technologies unless required by the existing
       	project context or the Intent.

       	## 5. Flagged Areas of Concern & Conflicts

       	- [Potential technical, UX, security, compatibility, performance, or
          architectural concern]
       	- [Unresolved contradiction or product decision requiring human input]

       	If no concerns or conflicts are identified, explicitly state:

       	- None identified.

       	CRITICAL RULES:

       	1. The Intent is the source of truth for the desired outcome.
       	2. Preserve the Intent's objective when converting it into requirements.
       	3. Separate requirements from implementation details.
       	4. Explicitly define both scope and boundaries.
       	5. Surface ambiguity instead of inventing decisions.
       	6. Surface conflicts instead of resolving product-policy conflicts yourself.
       	7. The Specification must be useful to a later Plan/Build stage.
       	8. The output must be a complete Markdown specification.
       	9. Do not discuss this prompt or your role.
       	10. Do not output anything except the completed specification.

       	Here is the approved Intent:

       	---

       	{intent}

       	---

       	Now produce the complete Specification.
     	"#,
		))
	}
	pub fn gen_plan(intent: &str, spec: &str) -> Result<String> {
		if intent.trim().is_empty() {
			return Err(anyhow!("cannot generate Plan prompt from empty Intent"));
		}
		if spec.trim().is_empty() {
			return Err(anyhow!("cannot generate Plan prompt from empty Spec"));
		}
		Ok(format!(
			r#"
       	You are the Plan stage of an SDLC pipeline.

       	Your job is to transform the approved Intent and Specification into a
       	concrete, repository-aware Implementation Plan.

       	You are NOT implementing the feature.
       	You are NOT writing source code.
       	You are NOT changing files.
       	You are NOT merely summarizing the Specification.

       	You are determining HOW the approved Specification should be implemented.

       	The resulting document will be written directly to:

            plan.md

       	Therefore, your entire response MUST be the implementation plan itself.
       	Do not include commentary before or after the plan.
       	Do not wrap the document in a Markdown code fence.

       	# REQUIRED OUTPUT STRUCTURE

       	Your response MUST follow this structure:

       	# Implementation Plan: [One-Sentence Strategy Summary]

       	The title MUST be a single sentence summarizing the primary
       	implementation strategy or technique that this plan will use.

       	The title should describe HOW the work will be accomplished, not merely
       	repeat the feature name.

       	For example:

       	# Implementation Plan: Introduce a normalized event pipeline that separates raw HID observation from semantic action dispatch.

       	Do not use generic titles such as:

       	- Implementation Plan: Keyboard Support
       	- Implementation Plan: New Feature
       	- Implementation Plan: Fix Bug

       	The title should communicate the central technical strategy.

       	## Overview

       	Briefly describe what this plan accomplishes and how it directly implements
       	the approved Specification.

       	The Overview must establish the relationship:

            Intent → Specification → Implementation Plan

       	Do not introduce functionality that is absent from the Specification.

       	## Context & References

       	- **Intent Reference**: `intent.md`
       	- **Specification Reference**: `spec.md`
       	- **Target Repository State**: [Current branch / relevant baseline]

       	Use the actual repository context available to you when known.

       	## Proposed Changes

       	List the precise files to create, modify, or delete.

       	For every meaningful implementation change, identify the concrete file path
       	and the action that will occur there.

       	Use this structure:

       	### [COMPONENT / MODULE NAME]

       	- **File Path**: `path/to/file.ext`
       	- **Action**: [Create | Modify | Delete]
       	- **Description of changes**:
          - Describe the structural changes.
          - Describe the logic changes.
          - Describe relevant API/type/interface changes.
          - Describe how this file connects to the rest of the implementation.

       	Do NOT invent arbitrary files.

       	Prefer existing repository files when they already provide the appropriate
       	extension point.

       	If the repository structure is not available, clearly identify the path as
       	requiring repository inspection rather than pretending that a path is known.

       	The plan must be specific enough that another agent or developer can execute
       	it without having to rediscover the architecture from scratch.

       	## Verification & Testing Strategy

       	Describe how the implementation will be verified.

       	### Unit Tests

       	Identify concrete tests to add or modify.

       	Use:

       	- [ ] Add/update test cases in `path/to/test.ext`
       	- [ ] Verify [specific behavior]

       	Tests should correspond directly to requirements in `spec.md`.

       	### Integration / End-to-End Checks

       	Identify integration tests, runtime checks, commands, or manual verification
       	needed to demonstrate that the implementation works.

       	Use concrete verification mechanisms where known.

       	For example:

       	- [ ] Run `cargo test`
       	- [ ] Run the relevant binary
       	- [ ] Verify the event flow produces the expected action
       	- [ ] Verify behavior in the affected application/runtime

       	Do not claim a test exists if it has not been identified.

       	### Expected Constraints/Risks

       	Identify potential regressions, compatibility issues, architectural risks,
       	performance concerns, policy boundaries, or unresolved assumptions.

       	Every significant concern should be connected to something identified in the
       	Specification.

       	## Execution Work Order

       	Provide an ordered sequence of implementation steps for an agent or human.

       	Each step must identify:

       	1. What is being changed.
       	2. Where it is being changed.
       	3. What dependency or prerequisite it has.
       	4. How it should be verified before proceeding.

       	Example:

       	1. Establish the new abstraction in `path/to/file.rs` and verify its unit tests.
       	2. Integrate the abstraction into `path/to/module.rs`.
       	3. Update dependent callers and tests.
       	4. Run the integration checks.
       	5. Perform the final regression suite.

       	The work order must reflect actual dependencies between changes.

       	Do not simply repeat the Proposed Changes section.

       	# PLANNING RULES

       	1. The Specification is the source of truth for WHAT must be built.
       	2. The Plan defines HOW that Specification will be implemented.
       	3. Do not expand scope beyond the Specification.
       	4. Do not resolve unresolved product decisions from the Specification by
          silently choosing one.
       	5. Identify unresolved decisions as risks or blockers.
       	6. Prefer the existing architecture and extension points over unnecessary
          new abstractions.
       	7. Identify concrete files, modules, types, interfaces, and tests whenever
          repository information makes that possible.
       	8. Do not write implementation code.
       	9. Do not produce pseudo-code as a substitute for a plan.
       	10. Do not produce generic advice.
       	11. Every proposed change must have a reason tied to the Specification.
       	12. Every verification step must prove a specific requirement or behavior.
       	13. The Execution Work Order must be actionable by another agent.
       	14. The final response must be a complete Markdown document.
       	15. Output ONLY the Implementation Plan.

       	# APPROVED INTENT

       	---

       	{intent}

       	---

       	# APPROVED SPECIFICATION

       	---

       	{spec}

       	---

       	Now produce the complete Implementation Plan.
      "#,
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
pub use structs::*;
pub mod structs {
	use super::*;
	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub struct Attempt {
		pub stage: Stage,
		pub number: u32,
		pub max: u32,
	}
	impl std::fmt::Display for Attempt {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{} attempt {}", self.stage, self.number)
		}
	}
	#[derive(Clone, Debug)]
	pub struct SprintPipeline {
		pub evaluator: Evaluator,
		pub generator: Box<dyn ArtifactGenerator>,
		pub session: Option<SdlcSession>,
		pub state_path: PathBuf,
		// pub event_tx: broadcast::Sender<SdlcEvent>,
		pub event_tx: broadcast::Sender<SdlcEvent>,
		pub stage_attempt: u32,
	}
	pub struct SprintRunner<'a> {
		pub pipeline: &'a mut SprintPipeline,
	}
	#[derive(Debug, Clone)]
	pub struct PipelineRuntime {
		pub activity: Vec<String>,
		pub pipeline: SprintPipeline,
		pub attempt: u32,
		pub stage: e::Stage,
		pub time_started: Instant,
		pub stage_time_started: Instant,
		pub phase: SdlcPhase,
		pub score: Option<f64>,
		pub confidence: Option<f64>,
		pub passed: Option<bool>,
		pub message: Option<String>,
		pub error: Option<String>,
		pub total_tokens: u64,
		pub total_agent_calls: u32,
		pub history: Vec<SdlcEvent>,
		pub events: Vec<SdlcEvent>,
	}
	impl PipelineRuntime {
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
	#[derive(Debug, Clone)]
	pub struct PipelineRuntimeView {
		pub activity: Vec<String>,
		pub attempt: u32,
		pub stage: e::Stage,
		pub time_started: Instant,
		pub stage_time_started: Instant,
		pub phase: SdlcPhase,
		pub score: Option<f64>,
		pub confidence: Option<f64>,
		pub passed: Option<bool>,
		pub message: Option<String>,
		pub error: Option<String>,
		pub total_tokens: u64,
		pub total_agent_calls: u32,
		pub history: Vec<SdlcEvent>,
		pub events: Vec<SdlcEvent>,
	}
	#[derive(Debug)]
	pub struct StageExecution {
		pub stage: e::Stage,
		pub attempt: Attempt,
		pub time_started: chrono::DateTime<Utc>,
		pub time_completed: chrono::DateTime<Utc>,
		pub result: StageResult,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct StageRecord {
		pub attempt: Attempt,
		pub stage: e::Stage,
		pub status: StageStatus,
		pub description: Option<String>,

		/// What actually performed the work.
		pub actor: StageActor,

		pub time_started: DateTime<Utc>,
		pub time_completed: Option<DateTime<Utc>>,
		// time_started: time_readable(time_started),
		// time_completed: time_readable(time_completed),
		// time_total: duration_readable(time_completed - time_started),
		/// Semantic evaluation of the resulting artifact/work.
		pub evaluation: Option<StageEvaluation>,
	}
	#[derive(Debug)]
	pub struct StageError;

	pub struct RetryPolicy {
		pub max_attempts: u32,
		pub retry_execution: bool,
		pub retry_evaluation: bool,
		pub retry_quality: bool,
		pub allow_human_intervention: bool,
	}
	#[derive(Clone, Debug)]
	pub struct Evaluator {
		pub jev: TypeSafeClient,
		pub session: SdlcSession,
	}
	#[derive(Debug, Clone)]
	pub struct Evaluation {
		pub score: f64,
		pub confidence: f64,
		pub meets_bar: bool,
		pub feedback: String,
		pub criteria: Vec<CriterionResult>,
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct EvaluationResult {
		pub name: String,
		pub passed: bool,
		pub score: f64,
		pub confidence: f64,
		pub explanation: String,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct StageEvaluation {
		pub stage: e::Stage,
		pub actor: StageActor,

		pub time_started: DateTime<Utc>,
		pub time_completed: DateTime<Utc>,
		pub time_total: chrono::Duration,
		pub score: f64,
		pub confidence: f64,
		pub passed: bool,

		pub evaluations: Vec<EvaluationResult>,
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct EvaluationRecord {
		pub score: f64,
		pub confidence: f64,
		pub meets_bar: bool,
		pub feedback: String,
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

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Verification {
		pub passed: bool,
		// pub score: u32,
		pub checks: Vec<CheckResult>,
		pub evaluations: Vec<EvaluationResult>,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct CriterionResult;

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct SdlcSession {
		pub id: Uuid,
		pub title: String,
		pub goal: String,
		pub stage: e::Stage,
		pub stages: Vec<StageRecord>,
		pub dir: PathBuf,
		pub time_created: DateTime<Utc>,
		pub time_updated: DateTime<Utc>,
		pub workspace: PathBuf,
	}
	pub struct SdlcView {
		pub runtime: PipelineRuntimeView,
		pub paused: bool,
		pub show_logs: bool,
		pub input: String,
		pub input_active: bool,
		pub events: Vec<Event>,
	}
	impl SdlcView {
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
	}

	#[derive(Clone, Debug)]
	pub struct ApiGenerator {
		// whatever API client you decide to use
	}
	#[derive(Clone, Debug)]
	pub struct LocalGenerator {
		pub runtime: AgentRuntime,
		pub model: String,
	}
	pub struct TerminalGuard;

	#[derive(Debug, Deserialize)]
	pub struct OllamaResponse {
		pub model: String,
		pub response: String,
		pub done: bool,
		pub done_reason: Option<String>,
	}
	pub struct WorkspaceSnapshot {
		pub git_status: String,
	}
	#[derive(Debug, Clone)]
	pub struct WorkspaceChanges {
		pub git_status: String,
	}
	pub struct BuildResult {
		pub changed_files: Vec<PathBuf>,
		pub created_files: Vec<PathBuf>,
		pub modified_files: Vec<PathBuf>,
		pub deleted_files: Vec<PathBuf>,
		pub commands: Vec<CommandResult>,
		pub agent_summary: Option<String>,
	}
	pub struct CommandResult {}
}
use traits as t;
use traits::*;
mod traits {
	use super::*;
	use crate::{
		model::{
			AgentTask,
			agent::{Agent, AgentContext},
			resolver::*,
			task::TaskResult,
		},
		prelude::*,
	};
	use anyhow::anyhow;
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

	pub struct Context {}

	// Steps to complete the pipeline
	pub trait Pipeline {
		type Stage: Stage;
		fn id(&self) -> &PipelineId;
		fn state(&self) -> &PipelineState<Self::Stage>;
		fn name(&self) -> &'static str;
		fn description(&self) -> &'static str {
			""
		}
		fn stages(&self) -> &[Self::Stage];
	}
	#[async_trait::async_trait]
	pub trait Runner {
		type Context;
		type Output;

		async fn run(&mut self, ctx: &mut Self::Context) -> Result<Self::Output>;

		fn cancel(&mut self);

		fn is_running(&self) -> bool;
	}
	trait Stage {
		fn name(&self) -> &'static str;
		fn description(&self) -> &'static str {
			""
		}
		fn prepare(&self, _ctx: &mut Context) -> Result<()> {
			Ok(())
		}
		fn run(&self, ctx: &mut Context) -> Result<StageResult>;
		fn evaluate(&self, _ctx: &Context, _result: &StageResult) -> Result<Option<Evaluation>> {
			Ok(None)
		}
		fn transition(
			&self,
			_ctx: &Context,
			_result: &StageResult,
			_evaluation: Option<&Evaluation>,
		) -> Result<StageTransition> {
			Ok(StageTransition::Next)
		}
		fn cleanup(&self, _ctx: &mut Context) -> Result<()> {
			Ok(())
		}
	}

	// Identity & Persistence
	#[async_trait::async_trait]
	pub trait ArtifactGenerator: Send + Sync + Debug {
		async fn generate(&self, prompt: &str) -> Result<String>;
		fn clone_box(&self) -> Box<dyn ArtifactGenerator>;
	}
	impl Clone for Box<dyn ArtifactGenerator> {
		fn clone(&self) -> Self {
			self.as_ref().clone_box()
		}
	}
	pub trait TextModel {
		async fn generate(&self, prompt: &str) -> Result<String>;
	}
}
mod ui {
	use super::*;
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
		let elapsed = format_elapsed(view.runtime.time_started.elapsed());
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

		let spinner_text = spinner(view.runtime.stage_time_started.elapsed());

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
				let frame = spinner(runtime.stage_time_started.elapsed());

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
					format_elapsed(runtime.stage_time_started.elapsed()),
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
		let spinner = spinner(runtime.stage_time_started.elapsed());
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

			SdlcEvent::StageRetrying { stage, number } => Line::from(vec![
				Span::styled("↻ ", Style::default().fg(Color::Yellow)),
				Span::styled(
					format!("{stage:?} · retry #{number}"),
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
			SdlcEvent::Activity {
				stage,
				attempt,
				message,
			} => Line::from(Span::styled(
				format!("↳ {message}"),
				Style::default().fg(Color::Gray),
			)),
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
impl LocalGenerator {
	pub fn new(runtime: AgentRuntime, model: impl Into<String>) -> Self {
		Self {
			runtime,
			model: model.into(),
		}
	}
	pub async fn run_agent(&self, prompt: &str) -> Result<String> {
		let task = AgentTask::new(prompt.to_string());

		let result = self.runtime.run_agent(task).await?;

		Ok(
			result
				.chat
				.or(result.summary)
				.unwrap_or_else(|| "Agent completed".to_string()),
		)
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
	fn clone_box(&self) -> Box<dyn ArtifactGenerator> {
		Box::new(self.clone())
	}
}
#[async_trait]
impl ArtifactGenerator for ApiGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		todo!("API generate")
	}
	fn clone_box(&self) -> Box<dyn ArtifactGenerator> {
		Box::new(self.clone())
	}
}
impl Attempt {
	pub fn new() -> Self {
		Self {
			stage: Stage::Intent,
			number: 0,
			max: 3,
		}
	}
}
struct EvaluationContext {
	pub stage: Stage,
	pub intent: Option<String>,
	pub spec: Option<String>,
	pub plan: Option<String>,
	pub tests: Option<String>,
	pub progress: Option<String>,
	pub verification: Option<String>,
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
	fn load(session: &SdlcSession, stage: Stage) -> Result<Self> {
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
impl Evaluator {
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
	async fn evaluate(
		&self,
		session: &SdlcSession,
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
impl SdlcSession {
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
	fn create_readable(&self) -> String {
		let current = Utc::now();
		current.format(FMT_HUMAN_READABLE).to_string()
	}
	pub fn workspace(&self) -> &Path {
		&self.workspace
	}
}

impl crate::traits::DateableSession for SdlcSession {
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
			Self::Abort | Self::Retry | Self::Reviewed => None,
		}
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
impl SprintPipeline {
	pub async fn new(runtime: AgentRuntime) -> anyhow::Result<Self> {
		dotenvy::dotenv().ok();

		let state_path = SpecialFile::SdlcCurrent.path()?;

		let session = match SpecialFile::SdlcCurrent
			.load::<SdlcSession>()
			.context("loading current SdlcSession")?
		{
			Some(session) => session,

			None => {
				let intent = "Do the work required to build this CLI";
				let title = Self::summarize_title(intent).await?;
				let dir = Self::init_session_dir(&title)?;

				Self::init_templates(&dir)?;

				let session = SdlcSession::new(title, dir)?;
				Self::session_save(&session)?;

				session
			}
		};

		if !state_path.exists() {
			Self::session_save(&session)?;
		}

		let evaluator = Evaluator {
			session: session.clone(),
			jev: TypeSafeClient::from_env()?,
		};

		let generator = Box::new(LocalGenerator::new(runtime.clone(), "qwen3:8b"));

		let (event_tx, _event_rx) = tokio::sync::broadcast::channel::<SdlcEvent>(256);

		Ok(Self {
			evaluator,
			generator,
			session: Some(session),
			state_path,
			event_tx,
			stage_attempt: 0,
		})
	}
	pub async fn init(&mut self, intent: impl Into<String>) -> Result<()> {
		let intent = intent.into();
		let title = Self::summarize_title(&intent).await?;
		let dir = Self::init_session_dir(&title)?;
		Self::init_templates(&dir)?;
		let session = SdlcSession::new(title, dir)?;
		self.session = Some(session);
		self.stage_attempt = 0;
		self.persist_session()?;
		Ok(())
	}
	pub fn subscribe(&self) -> broadcast::Receiver<SdlcEvent> {
		self.event_tx.subscribe()
	}
	async fn summarize_title(intent: &str) -> Result<String> {
		Ok(String::from("create-sdlc-pipeline"))
	}
	pub fn stage(&self) -> Option<Stage> {
		self.session.as_ref().map(|session| session.stage)
	}
	pub fn complete(&mut self) -> Result<()> {
		self.transition(e::Stage::Finalize)
	}
	pub fn is_complete(&self) -> bool {
		self.stage() == Some(e::Stage::Finalize)
	}
	fn stage_attempt(&self) -> u32 {
		self.stage_attempt
	}
	pub fn cancel(&mut self) {
		todo!("cancel")
	}
	pub fn fail(&mut self, _outcome: StageOutcome) -> Result<()> {
		todo!("fail")
	}
	pub fn decide(&self, outcome: &StageOutcome) -> Result<StageDecision> {
		println!(">>> DECIDE");
		println!(">>> outcome = {outcome:#?}");
		let decision = match outcome {
			StageOutcome::Complete { execution, .. } => {
				println!(
					">>> Complete: stage={:?} attempt={}/{}",
					execution.stage, execution.attempt.number, execution.attempt.max
				);
				match &execution.result {
					StageResult::Verification(verification) => {
						if verification.passed {
							println!(">>> verification passed -> Continue");
							StageDecision::Continue
						} else {
							println!(">>> verification failed -> Retry");
							StageDecision::Retry
						}
					}
					_ if execution.stage == e::Stage::Complete => {
						println!(">>> decision = Complete");
						StageDecision::Complete
					}
					_ => {
						println!(">>> decision = Continue");
						StageDecision::Continue
					}
				}
			}

			StageOutcome::NeedsRevision { .. } => {
				println!(">>> NeedsRevision");
				println!(">>> decision = Revise");
				StageDecision::Revise
			}

			StageOutcome::ExecutionFailed { attempt, .. } => {
				println!(
					">>> ExecutionFailed: stage={:?} attempt={}/{}",
					attempt.stage, attempt.number, attempt.max
				);

				if attempt.number < 3 {
					println!(">>> attempt {} < 3 -> decision = Retry", attempt.number);
					StageDecision::Retry
				} else {
					println!(
						">>> attempt {} >= 3 -> decision = AwaitHuman",
						attempt.number
					);
					StageDecision::AwaitHuman
				}
			}
			StageOutcome::EvaluationFailed { execution, .. } => {
				println!(
					">>> EvaluationFailed: stage={:?} attempt={}/{}",
					execution.stage, execution.attempt.number, execution.attempt.max
				);

				if execution.attempt.number < 3 {
					println!(
						">>> attempt {} < 3 -> decision = Retry",
						execution.attempt.number
					);
					StageDecision::Retry
				} else {
					println!(
						">>> attempt {} >= 3 -> decision = AwaitHuman",
						execution.attempt.number
					);
					StageDecision::AwaitHuman
				}
			}
		};
		println!(">>> FINAL DECISION = {decision:?}");
		Ok(decision)
	}
	async fn apply(
		&mut self,
		stage: e::Stage,
		attempt: Attempt,
		outcome: StageOutcome,
		decision: StageDecision,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
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
			StageDecision::Retry => Ok(RunControl::Continue),
			StageDecision::Revise => Ok(RunControl::Continue),
			StageDecision::AwaitHuman => {
				self
					.wait_for_intervention(stage, attempt, String::from("Evaluator Decision"), input_rx)
					.await?;
				Ok(RunControl::Continue)
			}
			StageDecision::Complete => {
				self.complete()?;
				Ok(RunControl::Exit)
			}
			StageDecision::Fail => {
				self.fail(outcome)?;
				Ok(RunControl::Exit)
			}
		}
	}
	async fn wait_for_intervention(
		&mut self,
		stage: e::Stage,
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
		}
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
			.map_or(0, |number| number + 1);
		Ok(Attempt { stage, number, max })
	}
	fn init_session_dir(title: &str) -> Result<PathBuf> {
		let sessions_dir = FS::ensure_dir(SpecialFile::SessionsDir.path()?)?;
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
	fn emit(&self, event: SdlcEvent) {
		let _ = self.event_tx.send(event);
	}
	async fn evaluate(&self, execution: &StageExecution) -> Result<StageEvaluation> {
		self
			.evaluator
			.evaluate(&self.session.as_ref().unwrap(), execution)
			.await
	}
	fn session(&mut self) -> Result<&mut SdlcSession> {
		self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))
	}
	fn session_read(&mut self, name: &str) -> Result<String> {
		read_from_session(name, self.session()?)
	}
	fn session_record(&mut self) -> Result<()> {
		let session = self.session()?;
		let index_path = SpecialFile::SessionsIndex.path()?;
		let mut sessions: Vec<SdlcSession> = FS::load(&index_path)?.unwrap_or_default();
		sessions.retain(|existing| existing.id != session.id);
		sessions.push(session.clone());
		FS::save(index_path, &sessions)?;
		Ok(())
	}
	fn session_save(session: &SdlcSession) -> Result<()> {
		FS::save(SpecialFile::SdlcCurrent.path()?, session)?;
		Ok(())
	}
	fn persist(&self) -> Result<()> {
		FS::save(&self.state_path, &self.session)
	}
	fn persist_evaluation(&mut self, evaluation: &StageEvaluation) -> Result<()> {
		let session = self.session()?;
		persist_evaluation(session, evaluation)?;
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

	fn persist_session(&mut self) -> Result<()> {
		let session = self.session()?;
		Self::session_save(session)
	}
	async fn resume(&mut self) -> Result<()> {
		if self.session.is_none() {
			return Err(anyhow::anyhow!("no active SDLC session to resume"));
		}
		self.update_progress("SDLC session resumed")?;
		self.persist()?;
		Ok(())
	}
	fn push_stage_record(&mut self, record: StageRecord) -> Result<()> {
		let session = self.session()?;
		session.stages.push(record);
		session.time_updated = Utc::now();
		self.persist()
	}
	fn retry(&mut self, _stage: e::Stage) -> Result<()> {
		Ok(())
	}
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
	fn transition(&mut self, next: e::Stage) -> Result<()> {
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
	fn update_progress(&mut self, message: &str) -> Result<()> {
		let session = self.session()?;
		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
		let entry = format!("\n## {timestamp}\n\n{message}\n");
		SessionFile::Progress.append(&session.dir, entry)?;
		Ok(())
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
	fn write(path: PathBuf, contents: String) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}
}
impl PipelineRuntime {
	pub fn new(pipeline: SprintPipeline) -> Self {
		let stage = pipeline.stage().unwrap().clone();
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
	fn retry(&mut self, _stage: e::Stage) -> Result<()> {
		Ok(())
	}
	pub async fn run(&mut self, input_rx: &mut UnboundedReceiver<SdlcInput>) -> Result<()> {
		println!(">>> PipelineRuntime::run");
		let mut runner = SprintRunner {
			pipeline: &mut self.pipeline,
		};
		println!(">>> SprintRunner constructed");
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
}

#[async_trait::async_trait]
impl traits::Runner for SprintRunner<'_> {
	type Context = UnboundedReceiver<SdlcInput>;
	type Output = ();
	async fn run(&mut self, input_rx: &mut Self::Context) -> Result<Self::Output> {
		let mut pending_input = None;
		let mut stage = self.load_state().context("load_state")?;
		let mut attempt_number = 0;

		self.emit(SdlcEvent::RunStarted);

		loop {
			println!(">>> stage={stage:?} attempt={attempt_number}");

			let attempt = Attempt {
				stage,
				number: attempt_number,
				max: 3,
			};

			let outcome = self
				.run_stage(stage, attempt, &mut pending_input)
				.await
				.with_context(|| format!("run_stage({stage:?})"))?;

			self.pipeline.persist_outcome(&outcome)?;
			let decision = self.decide(&outcome).await?;

			match decision {
				StageDecision::Retry => {
					attempt_number += 1;

					let control = self
						.apply(outcome, StageDecision::Retry, input_rx, &mut pending_input)
						.await?;

					match control {
						RunControl::Continue => continue,
						RunControl::Exit => return Ok(()),
					}
				}

				decision => {
					let control = self
						.apply(outcome, decision, input_rx, &mut pending_input)
						.await?;

					match control {
						RunControl::Continue => {
							stage = self
								.pipeline
								.stage()
								.ok_or_else(|| anyhow::anyhow!("pipeline has no stage"))?;

							attempt_number = 0;
						}

						RunControl::Exit => return Ok(()),
					}
				}
			}
		}
	}
	fn cancel(&mut self) {
		self.pipeline.cancel();
	}
	fn is_running(&self) -> bool {
		true
	}
}
impl SprintRunner<'_> {
	fn load_state(&mut self) -> Result<e::Stage> {
		let session = self
			.pipeline
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no SDLC session loaded"))?;
		let path = SpecialFile::SdlcCurrent.path()?;
		println!(">>> load_state path = {:?}", path);
		println!(">>> exists = {}", path.exists());
		let json = std::fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
		println!(">>> loaded state = {}", json);
		#[derive(serde::Deserialize)]
		struct PersistedStage {
			stage: e::Stage,
		}
		let state: PersistedStage = serde_json::from_str(&json)?;
		Ok(state.stage)
	}
	fn begin_attempt(&mut self, stage: Stage) -> Result<Attempt> {
		self.pipeline.next_attempt(stage)
	}
	fn transition(&mut self, next: e::Stage) -> Result<()> {
		self.pipeline.transition(next)
	}
	fn retry(&mut self, stage: e::Stage) -> Result<()> {
		self.pipeline.retry(stage)
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
			.evaluate(&self.pipeline.session.as_ref().unwrap(), &execution)
			.await?;
		Ok(semantic.with_evaluations(structural))
	}

	async fn evaluate_execution(&mut self, execution: StageExecution) -> Result<StageOutcome> {
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
				self.pipeline.persist_evaluation(&evaluation);
				Ok(StageOutcome::Complete {
					execution,
					evaluation,
				})
			}
			Err(error) => Ok(StageOutcome::EvaluationFailed { execution, error }),
		}
	}
	async fn stage_intent(&mut self) -> Result<StageResult> {
		let (stage, session_dir, goal) = {
			let session = self
				.pipeline
				.session
				.as_ref()
				.ok_or_else(|| anyhow!("no active SDLC session"))?;
			(session.stage, session.dir.clone(), session.goal.clone())
		};
		if stage != Stage::Intent {
			return Err(anyhow!("cannot execute Intent stage while at {stage:?}"));
		}
		if goal.trim().is_empty() {
			return Err(anyhow!("SDLC session goal is empty"));
		}
		let prompt = prompt::gen_intent(&goal)?;
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
		self.pipeline.update_progress("Intent stage completed")?;
		Ok(StageResult::Intent)
	}
	async fn stage_spec(&mut self) -> Result<StageResult> {
		let (stage, session_dir) = {
			let session = self
				.pipeline
				.session
				.as_ref()
				.ok_or_else(|| anyhow!("no active SDLC session"))?;
			(session.stage, session.dir.clone())
		};
		if stage != Stage::Spec {
			return Err(anyhow!("cannot execute Spec stage while at {stage:?}"));
		}
		let intent = self.pipeline.session_read("intent.md")?;
		if intent.trim().is_empty() {
			return Err(anyhow!("Intent artifact is empty"));
		}
		let prompt = prompt::gen_spec(&intent)?;
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
		self.pipeline.update_progress("Spec stage completed")?;
		Ok(StageResult::Spec)
	}
	async fn stage_plan(&mut self) -> Result<StageResult> {
		let (stage, session_dir) = {
			let session = self
				.pipeline
				.session
				.as_ref()
				.ok_or_else(|| anyhow!("no active SDLC session"))?;

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
		let prompt = prompt::gen_plan(&intent, &spec)?;
		if prompt.trim().is_empty() {
			return Err(anyhow!("generated Plan prompt is empty"));
		}
		std::fs::write("/tmp/estate-plan-prompt.md", &prompt)
			.context("writing Plan prompt debug file")?;
		let generated = self.pipeline.generator.generate(&prompt).await?;
		if generated.trim().is_empty() {
			return Err(anyhow!("generated Plan artifact is empty"));
		}
		SprintPipeline::write(session_dir.join("plan.md"), generated)?;
		self.pipeline.update_progress("Plan stage completed")?;
		Ok(StageResult::Plan)
	}
	async fn stage_build(&mut self) -> Result<StageResult> {
		let (stage, session_dir) = {
			let session = self
				.pipeline
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

			(session.stage.clone(), session.dir.clone())
		};
		if stage != Stage::Build {
			return Err(anyhow::anyhow!(
				"cannot execute Build stage while at {:?}",
				stage
			));
		}
		self.pipeline.update_progress("Build started")?;
		let workspace = self
			.pipeline
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?
			.workspace()
			.to_path_buf();
		let workspace_before = WorkspaceSnapshot::capture(&workspace)?;

		let task = AgentTask::new(
			"Implement the software described by the SDLC intent, specification,
			and plan.

			You are in the BUILD stage.

			Inspect the workspace using the available tools before making changes.

			Implement the planned functionality by:
			- creating required files,
			- modifying existing source files,
			- modifying configuration when required,
			- adding appropriate tests,
			- running relevant formatting, compilation, linting, and test commands,
			- fixing errors discovered during implementation.
			- respond with CODE ONLY. Do not response with markdown wrapping code blocks like literal ```
			- write directly to the files. Do not

			Do not merely describe an implementation. Perform the work in the
			workspace.

			Use the existing project structure and conventions whenever possible.

			When implementation is complete, verify the result using the most
			relevant available commands."
				.into(),
		);

		let result = self.pipeline.generator.generate(&task.prompt).await?;
		let workspace_after = WorkspaceSnapshot::capture(&workspace)?;
		let changes = workspace_before.diff(&workspace_after);
		SprintPipeline::write(session_dir.join("build.md"), changes.to_markdown(&result))?;
		self.pipeline.update_progress(&format!(
			"Build completed: {} file(s) changed",
			changes.file_count()
		))?;

		Ok(StageResult::Build)
	}
	async fn stage_verify(&mut self) -> Result<StageResult> {
		self.pipeline.update_progress("Verification started")?;
		let verification = self.verify_stage(self.pipeline.stage().unwrap()).await?;
		self.pipeline.update_progress(&format!(
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
	async fn run_stage(
		&mut self,
		stage: e::Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<StageOutcome> {
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Executing,
		});
		let execution = match self.run_current_stage(stage, attempt, pending_input).await {
			Ok(execution) => {
				self.emit(SdlcEvent::ExecutionComplete { stage });
				execution
			}
			Err(error) => {
				eprintln!("STAGE EXECUTION FAILED [{stage:?}]: {error:#}");
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

		self.evaluate_execution(execution).await
	}
	async fn run_current_stage(
		&mut self,
		stage: e::Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<StageExecution> {
		let time_started = Utc::now();
		let result = match stage {
			Stage::Intent => self.stage_intent().await?,
			Stage::Spec => self.stage_spec().await?,
			Stage::Plan => self.stage_plan().await?,
			Stage::Build => self.stage_build().await?,
			Stage::Verify => self.stage_verify().await?,
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
	async fn decide(&mut self, outcome: &StageOutcome) -> Result<StageDecision> {
		self.pipeline.decide(outcome)
	}

	async fn wait_for_intervention(
		&mut self,
		stage: e::Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self
			.pipeline
			.wait_for_intervention(stage, attempt, reason, input_rx)
			.await
	}
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
				self
					.pipeline
					.update_progress(&format!("Retrying stage {}", stage))?;
				self.handle_retry(stage, attempt).await?;

				Ok(RunControl::Continue)
			}
			StageDecision::Revise => {
				self
					.pipeline
					.update_progress(&format!("Revising stage {}", stage))?;
				self
					.handle_revision(stage, attempt, outcome, input_rx, pending_input)
					.await?;

				Ok(RunControl::Continue)
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
						self.handle_retry(stage, attempt).await?;
						Ok(RunControl::Continue)
					}
					Intervention::Revise => {
						self
							.handle_revision(stage, attempt, outcome, input_rx, pending_input)
							.await?;
						Ok(RunControl::Continue)
					}
					Intervention::Reviewed => Ok(RunControl::Continue),
					Intervention::ProvideContext(context) => {
						*pending_input = None;
						self
							.pipeline
							.update_progress(&format!("Human provided context: {context}"))?;
						Ok(RunControl::Continue)
					}
					Intervention::Abort => Ok(RunControl::Exit),
					Intervention::Human(str) => Ok(RunControl::Exit),
				}
			}
			StageDecision::Fail => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: format!("Stage {stage:?} failed on attempt {}", attempt.number),
				});
				Ok(RunControl::Exit)
			}
			StageDecision::Complete => {
				self.emit(SdlcEvent::PhaseChanged {
					phase: SdlcPhase::Completed,
				});
				self
					.pipeline
					.update_progress(&format!("{} completed", stage))?;
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

			SdlcInput::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_retry(&mut self, stage: e::Stage, attempt: Attempt) -> Result<()> {
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
	async fn handle_revision(
		&mut self,
		stage: e::Stage,
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
		stage: e::Stage,
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
		stage: e::Stage,
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
		}
	}
	async fn handle_failure_of_quality(
		&mut self,
		stage: e::Stage,
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
}
