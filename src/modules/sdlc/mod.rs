use crate::model::{
	AgentTask,
	agent::{Agent, AgentContext},
};
use anyhow::{Ok, Result};
use chrono::{DateTime, Local, Utc};
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
use serde::{Deserialize, Serialize};
use std::{
	io::Write,
	path::{Path, PathBuf},
	process::Command,
};

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

pub fn prompt_for_intent() -> Result<String> {
	Ok(String::from(
		"Finish SDLC Module which creates a loop for my SDLC. ",
	))
}
fn output_status(_output: &str) -> std::process::ExitStatus {
	// placeholder
	unimplemented!()
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
				state_path,
				session: None,
				evaluator,
			}));
		}

		let contents = std::fs::read_to_string(&state_path)?;
		let session = serde_json::from_str(&contents)?;

		Ok(Some(Self {
			state_path,
			session: Some(session),
			evaluator,
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

		std::fs::write(intent_path, format!("# Intent\n\n{}\n", intent))?;

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
	pub async fn intent(&mut self) -> Result<()> {
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

		self.update_progress("Intent stage completed")?;

		self.transition(Stage::Spec)?;

		Ok(())
	}

	pub async fn spec(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		if session.stage != Stage::Spec {
			return Err(anyhow::anyhow!(
				"cannot execute Spec stage while at {:?}",
				session.stage
			));
		}

		let intent = std::fs::read_to_string(session.dir.join("intent.md"))?;

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

		std::fs::write(session.dir.join("spec.md"), spec)?;

		self.update_progress("Spec stage completed")?;
		self.transition(Stage::Plan)?;

		Ok(())
	}

	pub async fn plan(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		if session.stage != Stage::Plan {
			return Err(anyhow::anyhow!(
				"cannot execute Plan stage while at {:?}",
				session.stage
			));
		}
		let spec = std::fs::read_to_string(session.dir.join("spec.md"))?;
		let plan = format!(
			"# Implementation Plan\n\n\
			 ## Specification\n\n\
			 {}\n\n\
			 ## Steps\n\n\
			 1. Inspect the existing implementation.\n\
			 2. Implement the required functionality.\n\
			 3. Add or update tests.\n\
			 4. Run deterministic checks.\n\
			 5. Fix any failures.\n\
			 6. Verify the resulting implementation.\n",
			spec.trim()
		);
		std::fs::write(session.dir.join("plan.md"), plan)?;
		self.update_progress("Plan stage completed")?;
		self.transition(Stage::Build)?;
		Ok(())
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
	pub async fn build(&mut self) -> Result<()> {
		let session = self
			.session()?
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let context = AgentContext::from_session(session)?;
		self.update_progress("Build started")?;
		let task = context.task.clone();
		let agent = Agent::new();
		// let result = agent.run_agent_loop(task).await?;
		// self.update_progress(&format!("Agent completed: {}", result.summary()))?;
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
	pub async fn deploy(&mut self) -> Result<()> {
		todo!("sdlc deploy")
	}

	/// Execute the Maintain stage.
	///
	/// Records the deployed state and determines whether a new lifecycle
	/// session should be created.
	pub async fn maintain(&mut self) -> Result<()> {
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

	fn root() -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
				std::fs::write(destination, format!("# {}\n\n", name))?;
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
		std::fs::write(index_path, contents)?;
		Ok(())
	}

	pub async fn run(&mut self) -> Result<()> {
		loop {
			match self.stage() {
				// None => self.start(...).await?,
				Some(Stage::Intent) => self.intent().await?,
				Some(Stage::Spec) => self.spec().await?,
				Some(Stage::Plan) => self.plan().await?,
				Some(Stage::Build) => self.build().await?,

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

	/// Run deterministic verification.
	///
	/// Examples:
	///
	///     cargo test
	///     npm test
	///     cargo check
	///     npm run build
	///     lint
	///     typecheck
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

	/// Run semantic verification through JEV.
	///
	/// JEV evaluates the relationship between:
	///
	///     intent.md
	///     spec.md
	///     implementation
	///     deterministic evidence
	///
	/// It does not mutate the lifecycle itself.
	async fn evaluate(&self, checks: &[CheckResult]) -> Result<Vec<EvaluationResult>> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let intent = std::fs::read_to_string(session.dir.join("intent.md"))?;
		let spec = std::fs::read_to_string(session.dir.join("spec.md"))?;
		let plan = std::fs::read_to_string(session.dir.join("plan.md"))?;
		let progress = std::fs::read_to_string(session.dir.join("progress.md"))?;

		// Eventually:
		//
		// let diff = git.diff()?;
		//
		// let result = jev.evaluate()?;
		//
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
	fn diagram() {
		//           SdlcAgent / Runner
		//                  │
		//                  │ owns lifecycle
		//                  ▼
		//                Sdlc
		//                  │
		//     ┌────────────┼────────────┐
		//     ▼            ▼            ▼
		//  Intent         Spec         Plan
		//     │            │            │
		//     └────────────┼────────────┘
		//                  │
		//                  ▼
		//             AgentContext
		//                  │
		//                  ▼
		//                Agent
		//                  │
		//     ┌────────────┼─────────────┐
		//     ▼            ▼             ▼
		// read_file     write_file    run_command
		//     │            │             │
		//     └────────────┼─────────────┘
		//                  ▼
		//                Verify
		//                  │
		//           ┌──────┴──────┐
		//           ▼             ▼
		//        checks          JEV
		//           │             │
		//           └──────┬──────┘
		//                  ▼
		//           pass / failure
		//             │         │
		//           pass       fail
		//             │         │
		//             ▼         ▼
		//           commit     Agent
		//                         │
		//                         └──→ fix
	}
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
/// Persistent state for the lifecycle runner.
///
/// The global location means there can be one active lifecycle at a time,
/// regardless of the current working directory.
pub struct Sdlc {
	state_path: PathBuf,
	session: Option<SdlcSession>,
	// jev: TypeSafeClient,
	evaluator: Evaluator,
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
	pub dir: PathBuf,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}
/// Result of verification.
///
/// Deterministic checks and JEV evaluations should eventually be represented
/// separately here. A session should only advance when its required gates pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	/// Results from deterministic tooling.
	pub checks: Vec<CheckResult>,
	/// Results produced by JEV.
	pub evaluations: Vec<EvaluationResult>,
}
