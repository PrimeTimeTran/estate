pub use crate::prelude::*;
use cli::prelude::{CliCommand, Context as CliContext, FormatArgs};
use tokio_util::sync::CancellationToken;

// cargo run daemon
// Troubleshooting:
//
// Check whether the process is still running:
//  ps -p <PID> -o pid,ppid,stat,command
//  ps -p 35762 -o pid,ppid,stat,command
//  pgrep -af 'estate.*daemon'
//  ps aux | grep '[e]state.*daemon'
// Example:
//   ps -p {} -o pid,ppid,stat,command
//
// Check the daemon process directly:
//   ps aux | grep 'estate*.*daemon'
// Check whether the Unix socket exists:
//   ls -l /tmp/estate-daemon.sock
//
// Check which process owns the socket:
//   lsof /tmp/estate-daemon.sock
//
// Inspect the process's open files:
//   lsof -p <PID>
//
// If the process appears stuck, inspect it with:
//   kill -0 <PID>
//
// `kill -0` doesn't terminate the process; it only checks
// whether the process exists and is accessible.
//
// Stop the daemon:
//   kill <PID>
//
// Force-stop only if necessary:
//   kill -9 <PID>
//
// Inspect
// ps -p <PID> -o pid,ppid,stat,lstart,etime,command
/// Daemon domain API — execute(Action) -> Response, start(), stop().
/// Daemon transport/lifecycle — Unix socket, Tokio tasks, channels, request parsing.
#[async_trait]
pub trait EstateDaemon {
	async fn execute(&mut self, action: ActionRequest) -> Result<DaemonResponse>;
	async fn start(&mut self, options: DaemonOptions) -> Result<DaemonResponse>;
	async fn shutdown(&mut self) -> Result<DaemonResponse>;
}
pub struct Daemon {
	pub runtime: EstateRuntime,
	pub dispatcher: EventDispatcher,
	engine: EstateEngine,
	shutdown: CancellationToken,
}

impl Daemon {
	pub fn new(engine: EstateEngine) -> Self {
		let runtime = EstateRuntime::new();
		let mut dispatcher = EventDispatcher::new();
		dispatcher.register(LogHandler);
		dispatcher.register(StateHandler);
		dispatcher.register(CommandHandler);
		dispatcher.register(TaskHandler);
		dispatcher.register(FileWatcherHandler);
		Self {
			engine,
			runtime,
			dispatcher,
			shutdown: CancellationToken::new(),
		}
	}
}
impl Daemon {
	pub async fn run_foreground(&mut self) -> anyhow::Result<()> {
		tracing::info!("daemon running in foreground");
		let mut rx = self.runtime.events.subscribe();
		self.runtime.emit(Event::daemon(EventKind::DaemonStarted));
		loop {
			tokio::select! {
				event = rx.recv() => {
					match event {
						Ok(event) => {
							self.dispatcher
								.dispatch(event, &self.runtime)
								.await;
						}

						Err(broadcast::error::RecvError::Closed) => {
							break;
						}

						Err(broadcast::error::RecvError::Lagged(count)) => {
							tracing::warn!(
								count,
								"event dispatcher lagged behind"
							);
						}
					}
				}

				_ = self.shutdown.cancelled() => {
					break;
				}
			}
		}

		tracing::info!("daemon stopped");

		Ok(())
	}
	///--------------------------------------------------------------------------------
	/// Long lived runner ready todo something
	/// cargo run daemon
	///--------------------------------------------------------------------------------
	async fn run_background(&mut self) -> Result<DaemonResponse> {
		tracing::info!("Run Background");
		let exe = std::env::current_exe()?;
		let child = std::process::Command::new(exe)
			.arg("tray")
			.stdin(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.spawn()?;
		let pid = child.id();

		eprintln!("Daemon started");
		eprintln!("PID: {}", pid);
		eprintln!("Socket: {}", SOCKET_PATH);

		Ok(DaemonResponse {
			status: "ok".into(),
			message: Some(format!("Daemon started with PID {}", pid)),
			..Default::default()
		})
	}
}
impl Daemon {
	pub fn handle(&self) -> DaemonHandle {
		DaemonHandle {
			runtime: self.runtime.clone(),
		}
	}
}
#[async_trait]
impl EstateDaemon for Daemon {
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
		tracing::info!(
			target: "estate::daemon",
			"shutdown requested"
		);
		self.runtime.emit(Event::daemon(EventKind::DaemonStopped));
		self.shutdown.cancel();

		Ok(DaemonResponse::default())
	}
}
impl Daemon {
	pub async fn is_running() -> bool {
		UnixStream::connect(SOCKET_PATH).await.is_ok()
	}

	/// Spawns the Unix socket listener and routes incoming connections into the daemon's channel
	pub async fn run_socket_server(socket_path: &str, tx: mpsc::Sender<DaemonMessage>) -> Result<()> {
		let _ = std::fs::remove_file(socket_path);

		let listener = UnixListener::bind(socket_path)?;

		tracing::info!(
				target: "estate::daemon::socket",
				socket = socket_path,
				"socket server listening"
		);

		loop {
			let (mut socket, _) = listener.accept().await?;

			tracing::debug!(
					target: "estate::daemon::socket",
					"client connected"
			);

			let tx = tx.clone();

			tokio::spawn(async move {
				tracing::debug!(
						target: "estate::daemon::socket",
						"processing socket request"
				);

				let mut buf = vec![0; 8192];

				let n = socket.read(&mut buf).await?;

				tracing::debug!(
						target: "estate::daemon::socket",
						bytes = n,
						"request received"
				);

				if n == 0 {
					tracing::debug!(
							target: "estate::daemon::socket",
							"client sent empty request"
					);

					return Ok(());
				}

				let parsed = parse_action(buf, n)?;

				tracing::info!(
						target: "estate::daemon::socket",
						path = ?parsed.path,
						line = ?parsed.line,
						column = ?parsed.column,
						mode = ?parsed.mode,
						"request parsed"
				);

				let (resp_tx, resp_rx) = oneshot::channel();

				tracing::debug!(
						target: "estate::daemon::socket",
						"dispatching request to processing loop"
				);

				tx.send(DaemonMessage::Execute {
					action: ActionRequest::Analyze {
						path: parsed.path,
						line: parsed.line,
						column: parsed.column,
						mode: parsed.mode,
					},
					respond_to: resp_tx,
				})
				.await?;

				tracing::debug!(
						target: "estate::daemon::socket",
						"waiting for action response"
				);

				let result = resp_rx.await?;

				tracing::debug!(
						target: "estate::daemon::socket",
						"action response received"
				);

				let payload = match result {
					Ok(response) => {
						tracing::debug!(
								target: "estate::daemon::socket",
								"serializing successful response"
						);

						serde_json::to_string(&response)?
					}

					Err(error) => {
						tracing::error!(
								target: "estate::daemon::socket",
								error = %error,
								"action failed"
						);

						serde_json::to_string(&serde_json::json!({
								"status": "error",
								"message": error.to_string(),
						}))?
					}
				};

				socket.write_all(payload.as_bytes()).await?;
				socket.write_all(b"\n").await?;

				tracing::debug!(
						target: "estate::daemon::socket",
						"response sent"
				);

				Ok::<(), anyhow::Error>(())
			});
		}
	}
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

fn parse_action(buf: Vec<u8>, n: usize) -> Result<IncomingRequest, Error> {
	let data = String::from_utf8_lossy(&buf[..n]);
	let trimmed = data.trim();
	let parsed: IncomingRequest = serde_json::from_str(trimmed).or_else(|_| {
		let unquoted = trimmed.trim_matches('"').replace("\\\"", "\"");
		serde_json::from_str(&unquoted)
	})?;
	Ok(parsed)
}
/// Verb layer: format, rename, save, index, analyze, build, search, resolve, organize imports, find references, go to definition
// pub struct EstateDaemon {
// 	pub estate: EstateEngine,
// 	pub actions: ActionRegistry,
// 	pub discovery: EstateDiscovery,
// 	// pub vfs: EstateVfs,
// 	// pub graph: EstateGraph,
// 	// pub resolver: EstateResolver,
// 	// pub registry: EstateRegistry,
// }
#[derive(Debug, Default)]
pub struct DaemonOptions {
	pub foreground: bool,
}
#[derive(Deserialize, Debug)]
struct IncomingRequest {
	path: PathBuf,
	line: Option<u32>,
	column: Option<u32>,
	mode: Option<String>,
}
// #[derive(Debug, serde::Deserialize)]
// pub struct SocketPayload {
// 	pub path: PathBuf,
// 	pub subject: Option<AnalyzeSubjectDto>,
// }
// #[derive(Debug, serde::Deserialize)]
// pub struct AnalyzeSubjectDto {
// 	pub offset: usize,
// 	pub identifier: Option<String>,
// }
// pub struct DaemonError {}
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
///--------------------------------------------------------------------------------
///      Server like long lived tasks in the current shell
///--------------------------------------------------------------------------------
pub struct StartDaemon;
#[async_trait::async_trait]
impl ::cli::command::CliCommand for StartDaemon {
	async fn run(&self, _ctx: &cli::context::Context) {
		println!("🚀 starting estate daemon");
		let exe = std::env::current_exe().expect("failed finding current executable");
		let child = std::process::Command::new(exe)
			.arg("daemon")
			.spawn()
			.expect("failed starting daemon");
		println!("✅ daemon started pid={}", child.id());
	}
}
// pub async fn daemon() {
// 	println!("🚀 estate daemon running");
// 	let runtime = EstateRuntime::new();
// 	let listener = TcpListener::bind("127.0.0.1:7788").await.unwrap();
// 	let rx = runtime.events.subscribe();
// 	tokio::join!(
// 		serve(listener, runtime.clone()),
// 		// event_loop(rx, runtime.clone())
// 	);
// }
pub enum DaemonMessage {
	Stop,
	Execute {
		action: ActionRequest,
		respond_to: oneshot::Sender<Result<DaemonResponse>>,
	},
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

async fn serve(listener: TcpListener, runtime: EstateRuntime) {
	loop {
		let (mut socket, _) = listener.accept().await.unwrap();
		let mut buf = [0; 1024];
		let n = socket.read(&mut buf).await.unwrap();
		let cmd = String::from_utf8_lossy(&buf[..n]);
		println!("server");
		println!("server {:?}", cmd);
		match cmd.trim() {
			"status" => {
				runtime.emit(Event::daemon(EventKind::StatusRequested));
				let out = serde_json::to_string(&runtime.state).unwrap();
				socket.write_all(out.as_bytes()).await.unwrap();
			}
			other => {
				runtime.emit(Event::cli(EventKind::CommandExecuted {
					command: other.to_string(),
				}));
			}
		}
	}
}

fn init_runtime_state() {
	let mut s = EstateState::load();
	s.starts += 1;
	s.started_at = EstateState::now();
	EstateState::save(&s);
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

pub struct SocketServer {
	path: PathBuf,
	tx: mpsc::Sender<DaemonMessage>,
	engine: EstateEngine,
}

impl SocketServer {
	pub fn new(
		path: impl Into<PathBuf>,
		tx: mpsc::Sender<DaemonMessage>,
		engine: EstateEngine,
	) -> Self {
		Self {
			path: path.into(),
			tx,
			engine,
		}
	}

	pub async fn run(self) -> Result<()> {
		// socket implementation
		Ok(())
	}
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

// App
//  │
//  ├── Engine
//  │
//  └── Daemon
//       │
//       ├── Runtime
//       │    ├── EventBus
//       │    └── State
//       │
//       ├── SocketServer
//       ├── EventLoop
//       └── TaskManager

#[derive(Clone)]
pub struct DaemonHandle {
	runtime: EstateRuntime,
}
impl DaemonHandle {
	pub fn emit(&self, event: Event) {
		self.runtime.emit(event);
	}
}
pub struct StatusDaemon;
#[async_trait::async_trait]
impl CliCommand for StatusDaemon {
	async fn run(&self, _ctx: &CliContext) {
		// EstateState::record_status_check();
		let state = EstateState::load();
		let pid = std::fs::read_to_string(PID_PATH).unwrap_or_else(|_| "unknown".to_string());
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
		let options = analyzer::AnalyzerOptions::default();
		let workspace = analyzer.analyze(request, &options)?;
		let _metrics = workspace.metrics();
		Ok(workspace)
	}
}
pub enum AnalysisRequest {
	AnalyzeWorkspace,
}
pub struct AnalyzeLoop {
	rx: mpsc::Receiver<AnalysisRequest>,
	workspace: Workspace,
}
impl AnalyzeLoop {
	pub async fn run(mut self) {
		todo!("AnalyzeLoop");
		let actions = ActionRegistry::from_analysis(&self.workspace);
		while let Some(request) = self.rx.recv().await {
			match request {
				AnalysisRequest::AnalyzeWorkspace => {
					println!("Processing workspace request & building AST...");
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
				Err(_) => break,
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
