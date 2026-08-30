use anyhow::Context;
use estate::{flow_warn, prelude::*};
use std::{
	env,
	path::{Path, PathBuf},
	time::Instant,
};
use uuid::Uuid;
// cargo -q run --bin runner -- python
// RUNNER=native cargo -q run --bin runner -- python
// RUNNER=docker cargo -q run --bin runner -- python
#[tokio::main]
async fn main() {
	let _result = setup_logging();
	match run().await {
		Ok(()) => {
			tracing::info!("Runner completed successfully");
		}
		Err(error) => {
			tracing::error!("{error:#}");
			std::process::exit(1);
		}
	}
}
#[tracing::instrument]
async fn run() -> anyhow::Result<()> {
	let trace = Tracer::new("app");
	let flow = trace.flow("execution");
	let language = Language::from_arg(env::args().nth(1).as_deref())?;
	let backend = env::var("RUNNER").unwrap_or_else(|_| "native".into());
	let run = Run::new(Path::new("/tmp/leetcode"), language)?;
	flow.debug(&format!(
		"Created Run: id={}, dir={}, exists={}",
		run.id,
		run.dir.display(),
		run.dir.exists()
	));
	let input = RunInput::default_for(language);
	// let input = RunInput {
	// 	solution: submission.source,
	// 	test_cases: problem.test_cases,
	// };
	run.prepare(language, input).await?;
	flow.debug(&format!(
		"Prepared: dir={}, exists={}",
		run.dir.display(),
		run.dir.exists()
	));
	let mut entries = tokio::fs::read_dir(&run.dir).await?;
	while let Some(entry) = entries.next_entry().await? {
		flow.debug(&format!("Prepared file: {}", entry.path().display()));
	}
	let runner: Box<dyn Runner> = match backend.as_str() {
		"native" => Box::new(NativeRunner),
		"docker" => Box::new(DockerRunner),
		other => anyhow::bail!("unknown runner: {other}"),
	};
	flow.debug(&format!(
		"Starting runner: backend={backend}, language={language:?}"
	));
	let telemetry = runner.run(&run, language).await?;
	flow.debug("Runner finished");
	if telemetry.exit_code == Some(0) {
		flow.info("Runner completed without system error");
		telemetry.print();
	} else {
		flow_warn!(
			flow,
			"Runner completed with exit code {:?}",
			telemetry.exit_code,
		);
		telemetry.print();
		// flow.warn(telemetry.exit_code);
	}
	flow.debug(&format!("Removing Run: {}", run.dir.display()));
	tokio::fs::remove_dir_all(&run.dir).await?;
	flow.debug("Cleanup complete");
	Ok(())
}
use tokio::time::{Duration, timeout};

///      Runner/Executors
pub struct Run {
	pub id: Uuid,
	pub dir: PathBuf,
	pub language: Language,
}
impl Run {
	pub fn new(root: &Path, language: Language) -> std::io::Result<Self> {
		let id = Uuid::new_v4();
		let dir = root.join(id.to_string());
		std::fs::create_dir_all(&dir)?;
		Ok(Self { id, dir, language })
	}
	pub async fn prepare(&self, language: Language, input: RunInput) -> anyhow::Result<()> {
		let solution_filename = language.entry();
		tokio::fs::write(self.dir.join(solution_filename), input.solution).await?;
		tokio::fs::write(
			self.dir.join("test_cases.json"),
			serde_json::to_vec_pretty(&input.test_cases)?,
		)
		.await?;
		Ok(())
	}
}
pub struct RunInput {
	pub solution: String,
	pub test_cases: Vec<TestCase>,
}
impl RunInput {
	pub fn default_for(language: Language) -> Self {
		let solution = match language {
			Language::Rust => r#"fn main() {
	println!("hello rust");
}"#
				.into(),
			Language::Python => r#"print("hello python")"#.into(),
			Language::JavaScript => r#"console.log("hello js")"#.into(),
		};
		Self {
			solution,
			test_cases: vec![
				TestCase {
					input: "1 2".into(),
					expected: "3".into(),
				},
				TestCase {
					input: "10 20".into(),
					expected: "30".into(),
				},
			],
		}
	}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
	pub input: String,
	pub expected: String,
}
#[async_trait::async_trait]
trait Runner {
	async fn run(&self, run: &Run, language: Language) -> anyhow::Result<RunTelemetry>;
}
struct NativeRunner;
#[async_trait::async_trait]
impl Runner for NativeRunner {
	async fn run(&self, run: &Run, language: Language) -> anyhow::Result<RunTelemetry> {
		match language {
			Language::Rust => run_rust(run).await,
			Language::Python => run_python(run).await,
			Language::JavaScript => run_javascript(run).await,
		}
	}
}
struct DockerRunner;
#[async_trait::async_trait]
impl Runner for DockerRunner {
	async fn run(&self, run: &Run, language: Language) -> anyhow::Result<RunTelemetry> {
		Self::docker_run(run, language, language.image()).await
	}
}
impl DockerRunner {
	// HOST                              CONTAINER
	// ─────────────────────────         ─────────────────
	// /tmp/leetcode/<run-id>/    ───►   /run
	//       │                              │
	//       ├── solution.rs                └── /run/solution.rs
	//       ├── solution.py
	//       └── ...
	async fn docker_run(run: &Run, language: Language, image: &str) -> anyhow::Result<RunTelemetry> {
		let volume = format!("{}:/run:rw", run.dir.display());

		tracing::debug!("DOCKER image: {image}");
		tracing::debug!("DOCKER volume: {volume}");

		let output = timeout(
			Duration::from_secs(5),
			tokio::process::Command::new("/usr/local/bin/docker")
				.args([
					"run",
					"--rm",
					"--network=none",
					"--read-only",
					"--cpus=1",
					"--memory=256m",
					"--pids-limit=64",
					"--cap-drop=ALL",
					"--security-opt=no-new-privileges",
					"--user=1000:1000",
					"-e",
					&format!("RUN_ID={}", run.id),
					"-v",
					&volume,
					image,
				])
				.output(),
		)
		.await
		.context("Docker execution timed out after 5 seconds")?
		.context("failed to start Docker")?;

		let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
		let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

		if !output.status.success() {
			anyhow::bail!(
				"Docker runner failed (exit code {:?})\n\
				 language: {:?}\n\
				 image: {image}\n\
				 stderr:\n{stderr}\n\
				 stdout:\n{stdout}",
				output.status.code(),
				language,
			);
		}

		let result_path = run.dir.join("result.json");

		let raw_result = tokio::fs::read_to_string(&result_path)
			.await
			.with_context(|| {
				format!(
					"Docker runner completed but did not produce {}",
					result_path.display()
				)
			})?;

		let container: ContainerResult =
			serde_json::from_str(&raw_result).context("Docker runner produced invalid result.json")?;

		anyhow::ensure!(
			container.run_id == run.id.to_string(),
			"Docker runner returned mismatched run ID: expected {}, got {}",
			run.id,
			container.run_id
		);

		Ok(RunTelemetry::new(
			language,
			container.setup_ms,
			container.compile_ms,
			container.execution_ms,
			container.exit_code,
			stdout,
			stderr,
		))
	}
}
/// Telemetry
#[derive(Debug, Default)]
struct RunTelemetry {
	language: Language,
	setup_ms: u128,
	compile_ms: u128,
	execution_ms: u128,
	exit_code: Option<i32>,
	stdout: String,
	stderr: String,
}
impl RunTelemetry {
	fn new(
		language: Language,
		setup_ms: u128,
		compile_ms: u128,
		execution_ms: u128,
		exit_code: Option<i32>,
		stdout: String,
		stderr: String,
	) -> Self {
		Self {
			language,
			setup_ms,
			compile_ms,
			execution_ms,
			exit_code,
			stdout,
			stderr,
		}
	}

	fn from_compile(
		language: Language,
		setup_ms: u128,
		compile_ms: u128,
		exit_code: Option<i32>,
		stdout: String,
		stderr: String,
	) -> Self {
		Self {
			language,
			setup_ms,
			compile_ms,
			execution_ms: 0,
			exit_code,
			stdout,
			stderr,
		}
	}
	// fn _from_error(
	// 	language: Language,
	// 	setup_ms: u128,
	// 	compile_ms: u128,
	// 	exit_code: Option<i32>,
	// 	stdout: String,
	// 	stderr: String,
	// ) -> Self {
	// 	Self {
	// 		language,
	// 		setup_ms,
	// 		compile_ms,
	// 		execution_ms: 0,
	// 		exit_code,
	// 		stdout,
	// 		stderr,
	// 	}
	// }
	fn print(&self) {
		tracing::info!("Language:        {:?}", self.language);
		tracing::info!("Setup:           {} ms", self.setup_ms);
		tracing::info!("Compile:         {} ms", self.compile_ms);
		tracing::info!("Execution:       {} ms", self.execution_ms);
		tracing::info!("Exit code:       {:?}", self.exit_code);
		tracing::info!("Stdout:");
		tracing::info!("{}", self.stdout);
		tracing::info!("Stderr:");
		tracing::info!("{}", self.stderr);
	}
}
#[derive(Debug, serde::Deserialize)]
struct ContainerResult {
	run_id: String,
	setup_ms: u128,
	compile_ms: u128,
	execution_ms: u128,
	exit_code: Option<i32>,
}
/// Language Specific
#[derive(Debug, Default, Clone, Copy)]
pub enum Language {
	#[default]
	Rust,
	Python,
	JavaScript,
}
impl Language {
	fn image(self) -> &'static str {
		match self {
			Self::Rust => "leetcode-rust",
			Self::Python => "leetcode-python",
			Self::JavaScript => "leetcode-node",
		}
	}
	fn entry(self) -> &'static str {
		match self {
			Self::Rust => "solution.rs",
			Self::Python => "solution.py",
			Self::JavaScript => "solution.js",
		}
	}
	fn from_arg(arg: Option<&str>) -> anyhow::Result<Self> {
		match arg {
			Some("rust") => Ok(Self::Rust),
			Some("python") | Some("py") => Ok(Self::Python),
			Some("javascript") | Some("js") => Ok(Self::JavaScript),
			Some(lang) => {
				anyhow::bail!("unknown language '{lang}', expected rust, python, or javascript")
			}
			None => Ok(Self::Rust),
		}
	}
}
///      Language
async fn run_rust(run: &Run) -> anyhow::Result<RunTelemetry> {
	let setup_start = Instant::now();
	let source = run.dir.join(run.language.entry());
	let binary = run.dir.join("solution");
	let setup_ms = setup_start.elapsed().as_millis();
	let compile_start = Instant::now();
	let output = tokio::process::Command::new("rustc")
		.arg(&source)
		.arg("-O")
		.arg("-o")
		.arg(&binary)
		.output()
		.await?;
	let compile_ms = compile_start.elapsed().as_millis();
	if !output.status.success() {
		return Ok(RunTelemetry::from_compile(
			Language::Rust,
			setup_ms,
			compile_ms,
			output.status.code(),
			String::from_utf8_lossy(&output.stdout).into_owned(),
			String::from_utf8_lossy(&output.stderr).into_owned(),
		));
	}

	let execution_start = Instant::now();

	let output = tokio::process::Command::new(&binary).output().await?;

	let execution_ms = execution_start.elapsed().as_millis();

	Ok(RunTelemetry::new(
		Language::Rust,
		setup_ms,
		compile_ms,
		execution_ms,
		output.status.code(),
		String::from_utf8_lossy(&output.stdout).into_owned(),
		String::from_utf8_lossy(&output.stderr).into_owned(),
	))
}
async fn run_python(run: &Run) -> anyhow::Result<RunTelemetry> {
	let start = Instant::now();
	let output = tokio::process::Command::new("python3")
		.arg(run.dir.join(run.language.entry()))
		.output()
		.await?;
	let execution_ms = start.elapsed().as_millis();
	Ok(RunTelemetry::new(
		Language::Python,
		execution_ms,
		0,
		0,
		output.status.code(),
		String::from_utf8_lossy(&output.stdout).into_owned(),
		String::from_utf8_lossy(&output.stderr).into_owned(),
	))
}
async fn run_javascript(run: &Run) -> anyhow::Result<RunTelemetry> {
	let start = Instant::now();
	let output = tokio::process::Command::new("node")
		.arg(run.dir.join(run.language.entry()))
		.output()
		.await?;
	let execution_ms = start.elapsed().as_millis();
	Ok(RunTelemetry::new(
		Language::JavaScript,
		execution_ms,
		0,
		0,
		output.status.code(),
		String::from_utf8_lossy(&output.stdout).into_owned(),
		String::from_utf8_lossy(&output.stderr).into_owned(),
	))
}
