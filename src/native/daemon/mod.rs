//! Estate's long-running application and runtime layer.
//! # Description
//!
//! The daemon coordinates Estate's services, configuration, lifecycle,
//! event handling, and runtime state. It provides the application boundary
//! between Estate's core domain and long-running frontends such as the CLI,
//! background daemon process, and future integrations.
//!
//! ## Architecture
//!
//! The daemon is organized into several layers of responsibility:
//! ```mermaid
//! graph TD
//!     Bootstrap --> App
//!
//!     subgraph Core Modules
//!         Config
//!         Events
//!         Resolver
//!     end
//!
//!     App --> Config
//!     App --> Events
//!     App --> Resolver
//!
//!     Config & Events & Resolver --> EstateCore[Estate Core]
//! ```
//! ## Responsibilities
//!
//! The daemon layer is responsible for:
//!
//! - **Application lifecycle** — initializing, starting, reloading, and
//!   stopping Estate services.
//! - **Configuration** — loading and resolving runtime configuration.
//! - **Events** — coordinating events between long-running services.
//! - **Projections** — maintaining derived runtime views of Estate state.
//! - **Resolution** — resolving runtime resources and dependencies.
//! - **Shell integration** — interacting with the host environment.
//! - **Linting** — running project and Estate-level lint operations.
//!
//!
//! ## Lifecycle
//!
//! A typical daemon lifecycle is:
//!
//! ```text
//! initialize → start → run → reload → stop
//! ```
//!
//! [`initialize`] prepares the runtime environment and dependencies.
//! [`start`] begins the daemon's active services.
//! [`reload`] refreshes runtime state or configuration without requiring a
//! complete restart.
//!
//! ## Modules
//!
//! - [`app`] — application-level daemon state and orchestration.
//! - [`bootstrap`] — runtime bootstrap and dependency initialization.
//! - [`config`] — daemon configuration.
//! - [`daemon`] — daemon process and lifecycle implementation.
//! - [`event`] — runtime event definitions and handling.
//! - [`initialize`] — initialization of daemon state and services.
//! - [`lint`] — linting operations.
//! - [`projection`] — derived views and projections of Estate state.
//! - [`reload`] — runtime reload operations.
//! - [`resolver`] — runtime resource and dependency resolution.
//! - [`shell`] — host shell and environment integration.
//! - [`start`] — daemon startup operations.
//!
//! ## Public API
//!
//! Commonly used daemon types are re-exported from this module so consumers
//! can access the primary API without depending on the internal module
//! layout.
//!
//! For example:
//!
//! ```ignore
//! use estate::daemon::EstateDaemon;
//! ```
//!
//! The module structure is intentionally subject to change while the daemon
//! architecture is being refined. Consumers should prefer the re-exported
//! API where possible.
// pub mod bootstrap;
// pub mod daemon_config;
pub mod initialize;
pub mod lint;
pub mod projection;
pub mod shell;

pub use lint::*;
pub use shell::*;

use crate::{
	app::{Runtime, state::EstateState, task, *},
	e,
	native::{daemon::DocCompiler, prelude::*, runtime::NativeRuntime},
	prelude::*,
};

use cli::prelude::Context as CliContext;

#[async_trait]
pub trait EstateDaemon {
	// # [Troubleshoot Daemon]
	// Set PID
	// PID=$(cat /tmp/estate-daemon.pid)
	//
	// ## Check Process Alive
	// ps -p "$PID" -o pid,ppid,stat,lstart,etime,command
	//  pgrep -af 'estate.*daemon'
	//  ps aux | grep '[e]state.*daemon'
	//
	// ### Example:
	// ps -p "$PID" -o pid,ppid,stat,lstart,etime,command
	//  PID  PPID STAT COMMAND
	// 79461 11424 S+   /Users/future/kb/project/target/debug/native
	//
	// ## Inspect PID File
	// ps -p "$(cat /tmp/estate-daemon.pid)" -o pid,ppid,stat,command
	//
	//
	// ## Check Unix socket exists:
	// ls -l /tmp/estate-daemon.sock
	//
	// Check which process owns the socket:
	// lsof /tmp/estate-daemon.sock
	//
	// Inspect the process's open files:
	// lsof -p "$PID"
	//
	async fn execute(&mut self, action: ActionRequest) -> Result<DaemonResponse>;
	async fn start(&mut self, options: DaemonOptions) -> Result<DaemonResponse>;
	// ## Inspect hanging process
	// kill -0 "$PID"
	//
	// - Doesn't terminate the process; it only checks
	// whether the process exists and is accessible.
	//
	// ## Graceful stop (handle cleanup, flush files, release resources, remove PID file, etc)
	// kill "$PID"
	//
	// ## Force-stop only if necessary:
	// kill -9 "$PID"
	async fn shutdown(&mut self) -> Result<DaemonResponse>;
}

pub struct Daemon<R: Runtime> {
	pub runtime: Arc<R>,
	pub dispatcher: EventDispatcher,
	pub shutdown_token: CancellationToken,
}

impl<R: Runtime> Daemon<R> {
	pub fn new(runtime: Arc<R>) -> Self {
		Self {
			runtime,
			dispatcher: EventDispatcher::new(),
			shutdown_token: CancellationToken::new(),
		}
	}
}

impl<R: Runtime> Daemon<R> {
	async fn run_background(&mut self) -> Result<DaemonResponse> {
		tracing::debug!("Run Background");
		let exe = std::env::current_exe()?;
		let child = std::process::Command::new(exe)
			.arg("tray")
			.stdin(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.spawn()?;

		let pid = child.id();

		Self::write_pid(pid)?;
		eprintln!("Daemon started");
		eprintln!("PID: {}", pid);

		Ok(DaemonResponse {
			status: "ok".into(),
			message: Some(format!("Daemon started with PID {}", pid)),
			..Default::default()
		})
	}

	pub async fn run_foreground(&mut self) -> Result<()> {
		let pid = std::process::id();
		Self::write_pid(pid)?;
		tracing::info!(pid, "[Daemon] Foreground run");
		self.runtime.emit(e::Event::daemon(e::Klass::DaemonStarted));
		self.shutdown_token.cancelled().await;
		tracing::info!(pid, "[Daemon] Foreground stop");
		self.runtime.emit(e::Event::daemon(e::Klass::DaemonStopped));
		Ok(())
	}

	fn write_pid(pid: u32) -> Result<()> {
		std::fs::write(crate::data::PID_PATH, pid.to_string())?;
		Ok(())
	}
}

#[async_trait]
impl<R> EstateDaemon for Daemon<R>
where
	R: Runtime,
{
	async fn execute(&mut self, action: ActionRequest) -> Result<DaemonResponse> {
		match action {
			ActionRequest::Analyze {
				path,
				line,
				column,
				mode,
			} => self.analyze(path, line, column, mode),
			ActionRequest::Metrics { path } => self.metrics(path),
			ActionRequest::ScanWorkspace { path } => self.scan_workspace(path),
			ActionRequest::InitializeEstate { path } => self.initialize_estate(path),
		}
	}
	async fn start(&mut self, options: DaemonOptions) -> Result<DaemonResponse> {
		if options.foreground {
			self.run_foreground().await?;
		} else {
			self.run_background().await?;
		}
		Ok(DaemonResponse::default())
	}
	async fn shutdown(&mut self) -> Result<DaemonResponse> {
		tracing::debug!("shutdown requested");
		self.shutdown_token.cancel();
		Ok(DaemonResponse::default())
	}
}
impl<R: Runtime> Daemon<R> {
	fn metrics(&mut self, path: PathBuf) -> Result<DaemonResponse> {
		let request = Analyze {
			target: AnalysisTarget::File(path),
			subject: None,
		};
		let analyzer = RustAnalyzer;
		let options = AnalyzerOptions::default();
		let workspace = analyzer.analyze(request, &options)?;
		let metrics = workspace.metrics();
		Ok(DaemonResponse {
			data: Some(serde_json::to_value(metrics)?),
			..Default::default()
		})
	}
	fn analyze(
		&mut self,
		path: PathBuf,
		line: Option<u32>,
		column: Option<u32>,
		mode: Option<String>,
	) -> Result<DaemonResponse> {
		let options = AnalyzerOptions {
			line,
			column,
			mode,
			include_private: true,
			include_tests: true,
		};
		let system_path = if let Some(s) = path.to_str() {
			if s.starts_with("file://") {
				url::Url::parse(s)?
					.to_file_path()
					.map_err(|_| anyhow::anyhow!("Invalid file URI"))?
			} else {
				path
			}
		} else {
			path
		};
		let report = revelation::analyzer::Workspace::analyze(&system_path, &options)?;
		Ok(DaemonResponse {
			data: Some(serde_json::to_value(report)?),
			..Default::default()
		})
	}
	fn scan_workspace(&mut self, _path: PathBuf) -> Result<DaemonResponse> {
		todo!("scan_workspace")
	}
	fn initialize_estate(&mut self, _path: PathBuf) -> Result<DaemonResponse> {
		todo!("initialize_estate")
	}
}

#[derive(Debug, Default)]
pub struct DaemonOptions {
	pub foreground: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionRequest {
	Analyze {
		path: PathBuf,
		line: Option<u32>,
		column: Option<u32>,
		mode: Option<String>,
	},
	ScanWorkspace {
		path: PathBuf,
	},
	Metrics {
		path: PathBuf,
	},
	InitializeEstate {
		path: PathBuf,
	},
}
pub struct StartDaemon;
#[async_trait::async_trait]
impl ::cli::command::CliCommand for StartDaemon {
	async fn run(&self, _ctx: &cli::context::Context) {
		let exe = std::env::current_exe().expect("failed finding current executable");
		let child = std::process::Command::new(exe)
			.arg("daemon")
			.spawn()
			.expect("failed starting daemon");
		println!("✅ daemon started pid={}", child.id());
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonResponse {
	pub status: String,
	pub message: Option<String>,
	pub data: Option<serde_json::Value>,
}
impl Default for DaemonResponse {
	fn default() -> Self {
		Self {
			status: "ok".into(),
			message: None,
			data: None,
		}
	}
}
pub struct DaemonServer;
impl DaemonServer {
	pub async fn run() {
		println!("🟢 daemon server running");
		if Path::new(SOCKET_PATH).exists() {
			std::fs::remove_file(SOCKET_PATH).unwrap();
		}
		let listener = UnixListener::bind(SOCKET_PATH).expect("failed binding socket");
		println!("listening on {}", SOCKET_PATH);
		loop {
			let (stream, _) = listener.accept().await.expect("accept failed");
			tokio::spawn(async move {
				Self::handle_client(stream).await;
			});
		}
	}
	async fn handle_client(stream: UnixStream) {
		let (reader, mut writer) = stream.into_split();
		let mut reader = BufReader::new(reader);
		let mut line = String::new();
		while reader.read_line(&mut line).await.unwrap() > 0 {
			let command = line.trim();
			println!("received command: {}", command);
			let response = match command {
				"status" => "daemon alive\n",
				"shutdown" => "shutdown requested\n",
				"hello" => "hello from daemon\n",
				_ => "unknown command\n",
			};
			writer.write_all(response.as_bytes()).await.unwrap();
			line.clear();
		}
	}
}
#[derive(Debug, Clone)]
pub struct DaemonMetrics {
	pub starts: u64,
	pub uptime: Duration,
	pub tasks_total: usize,
	pub tasks_running: usize,
	pub tasks_completed: usize,
	pub tasks_failed: usize,
}

#[derive(Clone, Debug)]
pub enum DaemonCommand {
	Stop,
	// Metrics,
	// Restart,
	// Refresh,
	// Enable,
	// Disable,
	// Status,
}
#[derive(Clone)]
pub struct DaemonHandle {
	runtime: NativeRuntime,
}
impl DaemonHandle {
	pub fn emit(&self, event: e::Event) {
		self.runtime.emit(event);
	}
}
pub struct StatusDaemon;
#[async_trait::async_trait]
impl CliCommand for StatusDaemon {
	async fn run(&self, _ctx: &CliContext) {
		let state = EstateState::load_from_disk().unwrap();
		let pid =
			std::fs::read_to_string(crate::data::PID_PATH).unwrap_or_else(|_| "unknown".to_string());
		println!("📊 Estate Daemon Status");
		println!("──────────────────────");
		println!("✅ Status:          OK");
		println!("🆔 PID:             {}", pid);
		println!("🚀 Starts:          {}", state.starts);
		println!("🔎 Status checks:   {}", state.status_checks);
		println!("🕒 Started at:      {}", state.started_at);
		println!("⏱ Longest run:     {}s", state.longest_run);
		match tokio::net::UnixStream::connect(SOCKET_PATH).await {
			Ok(mut stream) => {
				stream.write_all(b"status\n").await.unwrap();
				let mut buf = vec![0; 1024];
				let n = stream.read(&mut buf).await.unwrap();
				println!("Daemon response:");
				println!("{}", String::from_utf8_lossy(&buf[..n]));
			}
			Err(err) => {
				println!("❌ Daemon socket unavailable: {}", err);
			}
		}
	}
}
pub struct LintDaemon;
impl LintDaemon {
	pub async fn run(&self, args: &FormatArgs) {
		let compiler = DocCompiler::default();
		match compiler.run(&args.path) {
			Ok(_) => println!("Successfully formatted: {:?}", args.path),
			Err(e) => eprintln!("Error formatting file {:?}: {}", args.path, e),
		}
	}
}
pub struct AnalyzeDaemon;
impl AnalyzeDaemon {
	pub async fn run(
		&self,
		_ctx: &CliContext,
		args: &cli::context::AnalyzeArgs,
	) -> Result<Workspace, AnalysisError> {
		let target_path = PathBuf::from(&args.paths[0]);
		let request = Analyze {
			target: AnalysisTarget::File(target_path.clone()),
			subject: None,
		};
		let analyzer = RustAnalyzer;
		let options = revelation::analyzer::AnalyzerOptions::default();
		let workspace = analyzer.analyze(request, &options)?;
		let _metrics = workspace.metrics();
		Ok(workspace)
	}
}

#[derive(Clone, Copy)]
pub enum AnalysisRequest {
	AnalyzeWorkspace,
}
pub struct AnalyzeLoop {
	rx: mpsc::Receiver<AnalysisRequest>,
	workspace: Workspace,
}
impl AnalyzeLoop {
	pub async fn run(self) {
		todo!("AnalyzeLoop run");
		let actions = ActionRegistry::from_analysis(&self.workspace);
		while let Some(request) = self.rx.recv().await {
			match request {
				AnalysisRequest::AnalyzeWorkspace => {
					let _analyze_action = actions.iter().find(|a| a.title == "analyze.workspace");
				}
			}
		}
	}
	pub async fn run_cli(workspace: Workspace) {
		loop {
			Self::render_context(&workspace);
			let actions = ActionRegistry::from_analysis(&workspace);
			let options = actions
				.iter()
				.map(|action| demand::DemandOption::new(action.title.clone()))
				.collect::<Vec<_>>();
			let choice = demand::Select::new("What would you like to do?")
				.options(options)
				.run();
			match choice {
				Ok(selected) => {
					if let Some(action) = actions.iter().find(|a| a.title == selected) {
						action.execute(&workspace, ActionOptions::default());
					}
				}
				Err(_) => {
					break;
				}
			}
		}
	}
	fn render_context(result: &Workspace) {
		println!();
		println!("Workspace");
		println!("--------------------------------");
		let metrics = &result.metrics();
		println!("{:<20} {}", "Files", metrics.files.len());
		println!("{:<20} {}", "Packages", metrics.packages.len());
		println!("{:<20} {}", "Modules", metrics.modules.len());
		// println!("{:<20} {}", "Types", metrics.types.len());
		// println!("{:<20} {}", "Symbols", metrics.symbols.len());
		// println!("{:<20} {}", "Functions", metrics.functions.len());
		println!();
	}
}
