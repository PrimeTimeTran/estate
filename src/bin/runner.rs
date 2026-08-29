use estate::prelude::{logger, *};
use std::{
	env,
	path::{Path, PathBuf},
	time::Instant,
};
use uuid::Uuid;

/// cargo -q run --bin runner -- python
/// RUNNER=native cargo -q run --bin runner -- python
/// RUNNER=docker cargo -q run --bin runner -- python
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
	let mut flow = trace.flow("run");

	let language = Language::from_arg(env::args().nth(1).as_deref())?;
	let backend = env::var("RUNNER").unwrap_or_else(|_| "native".into());

	let run = Run::new(Path::new("/tmp/leetcode"))?;

	flow.debug(&format!(
		"Created Run: dir={}, exists={}",
		run.dir.display(),
		run.dir.exists()
	));

	run.prepare(language).await?;

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

	telemetry.print();

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
}
impl Run {
	pub fn new(root: &Path) -> std::io::Result<Self> {
		let id = Uuid::new_v4();
		let dir = root.join(id.to_string());
		std::fs::create_dir_all(&dir)?;
		Ok(Self { id, dir })
	}
	async fn prepare(&self, language: Language) -> anyhow::Result<()> {
		let (filename, source) = match language {
			Language::Rust => (
				"solution.rs",
				r#"fn main() {
		println!("hello rust");
	}"#,
			),
			Language::Python => ("solution.py", r#"print("hello python")"#),
			Language::JavaScript => ("solution.js", r#"console.log("hello js")"#),
		};
		tokio::fs::write(self.dir.join(filename), source).await?;
		Ok(())
	}
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
	async fn docker_run(run: &Run, language: Language, image: &str) -> anyhow::Result<RunTelemetry> {
		let volume = format!("{}:/run:rw", run.dir.display());

		tracing::debug!("DOCKER image: {image}");
		tracing::debug!("DOCKER volume: {volume}");

		let start = std::time::Instant::now();

		let result = timeout(
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
		.await;

		let execution_ms = start.elapsed().as_millis();

		let output = match result {
			Ok(output) => output?,
			Err(_) => {
				return Ok(RunTelemetry {
					language,
					setup_ms: 0,
					compile_ms: 0,
					execution_ms,
					exit_code: None,
					stdout: String::new(),
					stderr: "execution timed out".into(),
				});
			}
		};

		let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
		let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

		tracing::debug!("status: {}", output.status);
		tracing::debug!("stdout:\n{stdout}");
		tracing::debug!("stderr:\n{stderr}");

		let result_path = run.dir.join("result.json");
		let result: ContainerResult =
			serde_json::from_str(&tokio::fs::read_to_string(result_path).await?)?;

		anyhow::ensure!(result.run_id == run.id.to_string(), "run ID mismatch");

		Ok(RunTelemetry {
			language,
			setup_ms: 0,
			compile_ms: result.compile_ms,
			execution_ms,
			exit_code: result.exit_code,
			stdout,
			stderr,
		})
	}
}

///      Telemetry
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
	compile_ms: u128,
	// execution_ms: u128,
	exit_code: Option<i32>,
}

///      Language Specific
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
	fn from_arg(arg: Option<&str>) -> anyhow::Result<Self> {
		match arg {
			Some("rust") => Ok(Self::Rust),
			Some("python") | Some("py") => Ok(Self::Python),
			Some("javascript") | Some("js") => Ok(Self::JavaScript),
			Some(lang) => {
				// tracing::error!("unknown language '{lang}', expected rust, python, or javascript");
				anyhow::bail!("unknown language '{lang}', expected rust, python, or javascript")
			}
			None => Ok(Self::Rust),
		}
	}
}

///      Language
async fn run_rust(run: &Run) -> anyhow::Result<RunTelemetry> {
	let source = r#"
fn main() {
	println!("hello rust");
}
"#;
	let setup_start = Instant::now();
	let source_path = run.dir.join("solution.rs");
	let binary_path = run.dir.join("solution");
	tokio::fs::write(&source_path, source).await?;
	let setup_ms = setup_start.elapsed().as_millis();
	let compile_start = Instant::now();
	let compile = tokio::process::Command::new("rustc")
		.arg(&source_path)
		.arg("-O")
		.arg("-o")
		.arg(&binary_path)
		.output()
		.await?;
	let compile_ms = compile_start.elapsed().as_millis();
	if !compile.status.success() {
		return Ok(RunTelemetry {
			language: Language::Rust,
			setup_ms,
			compile_ms,
			execution_ms: 0,
			exit_code: compile.status.code(),
			stdout: String::from_utf8_lossy(&compile.stdout).into_owned(),
			stderr: String::from_utf8_lossy(&compile.stderr).into_owned(),
		});
	}
	let execution_start = Instant::now();
	let output = tokio::process::Command::new(&binary_path).output().await?;
	let execution_ms = execution_start.elapsed().as_millis();
	Ok(RunTelemetry {
		language: Language::Rust,
		setup_ms,
		compile_ms,
		execution_ms,
		exit_code: output.status.code(),
		stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
		stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
	})
}
async fn run_python(run: &Run) -> anyhow::Result<RunTelemetry> {
	let source = r#"
print("hello python")
"#;
	let setup_start = Instant::now();
	let source_path = run.dir.join("solution.py");
	tokio::fs::write(&source_path, source).await?;
	let setup_ms = setup_start.elapsed().as_millis();
	let execution_start = Instant::now();
	let output = tokio::process::Command::new("python3")
		.arg(&source_path)
		.output()
		.await?;
	let execution_ms = execution_start.elapsed().as_millis();
	Ok(RunTelemetry {
		language: Language::Python,
		setup_ms,
		compile_ms: 0,
		execution_ms,
		exit_code: output.status.code(),
		stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
		stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
	})
}
async fn run_javascript(run: &Run) -> anyhow::Result<RunTelemetry> {
	let source = r#"
console.log("hello js");
"#;
	let setup_start = Instant::now();
	let source_path = run.dir.join("solution.js");
	tokio::fs::write(&source_path, source).await?;
	let setup_ms = setup_start.elapsed().as_millis();
	let execution_start = Instant::now();
	let output = tokio::process::Command::new("node")
		.arg(&source_path)
		.output()
		.await?;
	let execution_ms = execution_start.elapsed().as_millis();
	Ok(RunTelemetry {
		language: Language::JavaScript,
		setup_ms,
		compile_ms: 0,
		execution_ms,
		exit_code: output.status.code(),
		stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
		stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
	})
}

fn setup_logging() -> anyhow::Result<()> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;
	// tracing::trace!("[dryrun] trace");
	// tracing::debug!("[dryrun] debug");
	// tracing::info!("[dryrun] info");
	// tracing::warn!("[dryrun] warn");
	// tracing::error!("[dryrun] error");
	Ok(())
}
