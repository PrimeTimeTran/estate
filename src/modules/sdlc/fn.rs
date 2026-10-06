use super::*;
use super::{c, e, f, s, t, u};

pub fn git_status() -> Result<String> {
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
pub fn format_elapsed(duration: Duration) -> String {
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
pub fn slugify(input: &str) -> String {
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
pub fn persist_evaluation(
	session: &mut SdlcSession,
	evaluation: &StageEvaluation,
	attempt: Attempt,
) -> Result<()> {
	let status = if evaluation.passed {
		StageStatus::Completed
	} else {
		StageStatus::NeedsRevision
	};
	let record = StageRecord {
		description: Some(String::from("Evaluation Complete")),
		actor: evaluation.actor.clone(),
		attempt,
		evaluation: Some(evaluation.clone()),
		stage: evaluation.stage,
		time_started: evaluation.time_started,
		time_completed: Some(Utc::now()),
		status,
	};
	let time_started = evaluation.time_started;
	let time_completed = Utc::now();
	session.stages.push(record);
	session.time_updated = Utc::now();
	Ok(())
}
pub fn time_readable(time: chrono::DateTime<chrono::Utc>) -> String {
	time.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()
}
pub fn short_duration_readable(duration: chrono::Duration) -> String {
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
pub fn duration_readable(duration: chrono::Duration) -> String {
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
pub fn log_step_transition(from: Stage, to: Stage) -> Result<()> {
	let path: PathBuf = env::current_dir()?.join("current_step.txt");
	let mut file = OpenOptions::new().create(true).append(true).open(path)?;
	writeln!(file, "{:?} -> {:?}", from, to)?;
	Ok(())
}
pub async fn monitor<F, T>(
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
	let mut ticker = tokio::time::interval(c::STATUS_INTERVAL);
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

pub fn build_steps() -> Vec<&'static str> {
	vec![
		"Read plan.md and identify the files that must be created or modified.",
		"Inspect the relevant existing files and repository structure for the files identified by the plan.",
		"Create the planned files and establish their basic structure.",
		"Implement the planned functionality in the files created or modified so far.",
		"Inspect the implementation and compare it against the specification and plan.",
		"Add the planned unit and integration tests.",
		"Run the relevant tests and verification commands.",
		"Inspect any failures and determine what implementation changes are required.",
		"Fix the implementation or tests based on the failures and rerun verification.",
		"Review the completed implementation against the plan and identify any remaining work.",
	]
}
pub fn build_step_prompt(
	instruction: &str,
	step: usize,
	total: usize,
	plan: &str,
	workspace: &str,
) -> String {
	format!(
		r#"
      You are executing BUILD STEP {step}/{total}.

      YOUR CURRENT TASK:
      {instruction}

      ---

      IMPLEMENTATION PLAN:
      {plan}

      ---

      CURRENT WORKSPACE:
      {workspace}

      ---

      RULES:

      - Perform the work directly in the workspace.
      - Inspect files before modifying them.
      - Do not merely describe what should be done.
      - Complete only the current build step.
      - Preserve existing project conventions.
      - Do not undo correct work from previous steps.
      - Use run_command when inspection, file creation, editing, or verification is required.
      - When this step is complete, stop.
    "#,
	)
}
