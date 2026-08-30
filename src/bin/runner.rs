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
	let problem = Problem::load("two-sum", language).await?;
	// let submission = Submission::for_success(&problem, language);
	// let input = RunInput::new(submission.source, &problem)?;
	let submission = Submission::for_success(&problem, language)?;
	let input = RunInput::new(submission.source.clone(), &problem)?;
	let run = Run::new(language, input.clone())?;
	run.prepare(input).await?;
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

	let mut results = Vec::with_capacity(problem.test_cases.len());

	for (_index, test_case) in problem.test_cases.iter().enumerate() {
		let telemetry = runner.run(&run, language, test_case).await?;
		results.push(telemetry);
	}

	flow.debug(&format!("Runner finished: {} test cases", results.len()));

	for (index, telemetry) in results.iter().enumerate() {
		if telemetry.exit_code == Some(0) {
			flow.info(&format!(
				"Test case {} completed without system error",
				index + 1
			));
		} else {
			flow_warn!(
				flow,
				"Test case {} completed with exit code {:?}",
				index + 1,
				telemetry.exit_code,
			);
		}

		telemetry.print();
	}

	flow.debug(&format!("Removing Run: {}", run.dir.display()));
	tokio::fs::remove_dir_all(&run.dir).await?;
	flow.debug("Cleanup complete");

	Ok(())
}
use tokio::{
	process::Child,
	time::{Duration, timeout},
};

/// Runner/Executors
pub struct Run {
	pub id: Uuid,
	pub language: Language,
	pub dir: PathBuf,
	pub input: RunInput,
}
impl Run {
	pub fn new(language: Language, input: RunInput) -> std::io::Result<Self> {
		let id = Uuid::new_v4();
		let dir = Path::new("/tmp/leetcode").join(id.to_string());
		std::fs::create_dir_all(&dir)?;
		Ok(Self {
			id,
			dir,
			language,
			input,
		})
	}
	pub async fn prepare(&self, input: RunInput) -> anyhow::Result<()> {
		let solution_filename = self.language.entry();
		tokio::fs::write(self.dir.join(solution_filename), input.solution).await?;
		tokio::fs::write(
			self.dir.join("test_cases.json"),
			serde_json::to_vec_pretty(&input.test_cases)?,
		)
		.await?;
		Ok(())
	}
}

#[async_trait::async_trait]
trait Runner {
	// async fn run(&self, run: &Run, language: Language) -> anyhow::Result<RunTelemetry>;
	async fn run(
		&self,
		run: &Run,
		language: Language,
		test_case: &TestCase,
	) -> anyhow::Result<RunTelemetry>;
}
struct NativeRunner;
#[async_trait::async_trait]
impl Runner for NativeRunner {
	async fn run(
		&self,
		run: &Run,
		language: Language,
		test_case: &TestCase,
	) -> anyhow::Result<RunTelemetry> {
		tracing::debug!("NativeRunner::run: language={language:?}");

		let result = match language {
			Language::Rust => {
				tracing::debug!("starting rust runner");
				run_rust(run, test_case).await
			}
			Language::Python => {
				tracing::debug!("starting python runner");
				run_python(run, test_case).await
			}
			Language::JavaScript => {
				tracing::debug!("starting javascript runner");
				run_javascript(run, test_case).await
			}
		};

		tracing::debug!("NativeRunner::run: finished");
		result
	}
}
struct DockerRunner;
#[async_trait::async_trait]
impl Runner for DockerRunner {
	async fn run(
		&self,
		run: &Run,
		language: Language,
		test_case: &TestCase,
	) -> anyhow::Result<RunTelemetry> {
		tracing::debug!("DockerRunner::run: language={language:?}");

		Self::docker_run(run, language, test_case).await
	}
}
impl DockerRunner {
	async fn docker_run(
		run: &Run,
		language: Language,
		test_case: &TestCase,
	) -> anyhow::Result<RunTelemetry> {
		let input_path = run.dir.join("input");
		tokio::fs::write(&input_path, &test_case.input).await?;
		tracing::debug!("DOCKER test input: {:?}", test_case.input);
		let child = Self::run_docker(run, language).await?;
		let output = timeout(Duration::from_secs(5), child.wait_with_output())
			.await
			.context("Docker execution timed out after 5 seconds")?
			.context("failed waiting for Docker")?;

		let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
		let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

		if !output.status.success() {
			anyhow::bail!(
				"Docker runner failed (exit code {:?})\n\
				 language: {:?}\n\
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

	async fn run_docker(run: &Run, language: Language) -> Result<Child> {
		let volume = format!("{}:/run:rw", run.dir.display());
		tracing::debug!("DOCKER volume: {volume}");
		tokio::process::Command::new("/usr/local/bin/docker")
			.args([
				"run",
				"--rm",
				"-i",
				"--network=none",
				"--read-only",
				"--tmpfs",
				"/tmp:rw,noexec,nosuid,size=64m",
				"--cpus=1",
				"--memory=256m",
				"--pids-limit=64",
				"--cap-drop=ALL",
				"--security-opt=no-new-privileges",
				"--user=1000:1000",
				"-e",
				&format!("RUN_ID={}", run.id),
				"-e",
				&format!("LANGUAGE={}", language.as_str()),
				"-v",
				&volume,
				"leetcode-runner",
			])
			.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::piped())
			.spawn()
			.context("failed to start Docker")
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
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
	#[default]
	Rust,
	Python,
	JavaScript,
}
impl Language {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Rust => "rust",
			Self::Python => "python",
			Self::JavaScript => "javascript",
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
async fn run_rust(run: &Run, test_case: &TestCase) -> anyhow::Result<RunTelemetry> {
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

	let mut child = tokio::process::Command::new(&binary)
		.stdin(std::process::Stdio::piped())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()?;

	if let Some(mut stdin) = child.stdin.take() {
		use tokio::io::AsyncWriteExt;

		stdin.write_all(test_case.input.as_bytes()).await?;
	}

	let output = child.wait_with_output().await?;

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
async fn run_python(run: &Run, test_case: &TestCase) -> anyhow::Result<RunTelemetry> {
	tracing::debug!("test input: {:?}", test_case.input);
	let execution_start = Instant::now();

	let source = run.dir.join(run.language.entry());

	let mut child = tokio::process::Command::new("python3")
		.arg(&source)
		.stdin(std::process::Stdio::piped())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()?;
	if let Some(mut stdin) = child.stdin.take() {
		use tokio::io::AsyncWriteExt;

		tracing::debug!("writing stdin: {:?}", test_case.input);

		stdin.write_all(test_case.input.as_bytes()).await?;
	}
	let output = child.wait_with_output().await?;
	let execution_ms = execution_start.elapsed().as_millis();

	Ok(RunTelemetry::new(
		Language::Python,
		0,
		0,
		execution_ms,
		output.status.code(),
		String::from_utf8_lossy(&output.stdout).into_owned(),
		String::from_utf8_lossy(&output.stderr).into_owned(),
	))
}
async fn run_javascript(run: &Run, test_case: &TestCase) -> anyhow::Result<RunTelemetry> {
	let source = run.dir.join(run.language.entry());

	let (output, execution_ms) = execute("node", &[&source], &test_case.input).await?;

	Ok(RunTelemetry::new(
		Language::JavaScript,
		0,
		0,
		execution_ms,
		output.status.code(),
		String::from_utf8_lossy(&output.stdout).into_owned(),
		String::from_utf8_lossy(&output.stderr).into_owned(),
	))
}
#[derive(Clone, Debug)]
pub struct Problem {
	pub id: Uuid,
	// pub title: String,
	pub slug: String,
	pub test_cases: Vec<TestCase>,
}

impl Problem {
	pub async fn load(slug: impl Into<String>, language: Language) -> anyhow::Result<Self> {
		let slug = slug.into();

		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("src/data/problems")
			.join(format!("{slug}.json"));

		let raw = tokio::fs::read_to_string(&path)
			.await
			.with_context(|| format!("failed to read {}", path.display()))?;

		let file: ProblemFile =
			serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;

		let test_cases = file
			.test_cases
			.get(&language)
			.cloned()
			.ok_or_else(|| anyhow::anyhow!("problem {slug} has no test cases for {language:?}"))?;

		anyhow::ensure!(
			!test_cases.is_empty(),
			"problem {slug} has no test cases for {language:?}"
		);

		Ok(Self {
			id: file.id.unwrap_or_else(Uuid::new_v4),
			slug: file.slug,
			test_cases,
		})
	}
	pub fn success_source(&self, language: Language) -> anyhow::Result<String> {
		match self.slug.as_str() {
			"two-sum" => Ok(match language {
				Language::Rust => include_str!("../data/problems/two-sum/success.rs").into(),
				Language::Python => include_str!("../data/problems/two-sum/success.py").into(),
				Language::JavaScript => include_str!("../data/problems/two-sum/success.js").into(),
			}),

			other => anyhow::bail!("no success fixture registered for problem `{other}`"),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestCase {
	pub input: String,
	pub expected: String,
}

#[derive(Debug, Deserialize)]
struct ProblemFile {
	pub id: Option<Uuid>,
	pub slug: String,
	pub test_cases: HashMap<Language, Vec<TestCase>>,
}

#[derive(Clone, Debug)]
pub struct RunInput {
	pub solution: String,
	pub test_cases: Vec<TestCase>,
}

impl RunInput {
	pub fn new(solution: String, problem: &Problem) -> anyhow::Result<Self> {
		anyhow::ensure!(
			!problem.test_cases.is_empty(),
			"cannot run solution without test cases"
		);
		Ok(Self {
			solution,
			test_cases: problem.test_cases.clone(),
		})
	}
}

pub struct TestResult {
	pub index: usize,
	pub input: String,
	pub expected: String,
	pub actual: String,
	pub passed: bool,
	pub execution_ms: u128,
}
pub struct Submission<'p> {
	pub id: Uuid,
	pub problem: &'p Problem,
	pub source: String,
}
impl<'p> Submission<'p> {
	pub fn new(problem: &'p Problem, source: impl Into<String>) -> Self {
		Self {
			id: Uuid::new_v4(),
			problem,
			source: source.into(),
		}
	}
	pub fn for_success(problem: &'p Problem, language: Language) -> anyhow::Result<Self> {
		let source = problem.success_source(language)?;
		Ok(Self::new(problem, source))
	}
	pub fn for_failure(problem: &'p Problem, language: Language) -> Self {
		Self::new(
			problem,
			match language {
				Language::Rust => {
					r#"
fn main() {
	panic!("intentional failure");
}
"#
				}
				Language::Python => {
					r#"
raise Exception("intentional failure")
"#
				}
				Language::JavaScript => {
					r#"
throw new Error("intentional failure");
"#
				}
			},
		)
	}
	pub fn for_wrong_answer(problem: &'p Problem, language: Language) -> Self {
		Self::new(
			problem,
			match language {
				Language::Rust => {
					r#"
fn main() {
	println!("0");
}
"#
				}
				Language::Python => {
					r#"
print("0")
"#
				}
				Language::JavaScript => {
					r#"
console.log("0");
"#
				}
			},
		)
	}
}
async fn execute(
	command: &str,
	args: &[&std::path::Path],
	input: &str,
) -> anyhow::Result<(std::process::Output, u128)> {
	let start = Instant::now();

	let mut child = tokio::process::Command::new(command)
		.args(args)
		.stdin(std::process::Stdio::piped())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()?;

	if let Some(mut stdin) = child.stdin.take() {
		use tokio::io::AsyncWriteExt;
		stdin.write_all(input.as_bytes()).await?;
	}

	let output = child.wait_with_output().await?;
	let execution_ms = start.elapsed().as_millis();

	Ok((output, execution_ms))
}
