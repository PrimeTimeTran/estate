use crate::{_core::EstateDiscovery, constants::*, daemon::*, estate::Estate};
use anyhow::{Error, Result};
use async_trait::async_trait;
use cli::{CliCommand, Context};
use revelation::analyzer::{
	AnalysisTarget, Analyze, Analyzer, AnalyzerOptions, RustAnalyzer, Workspace,
};
use serde::{Deserialize, Serialize};
use std::{
	path::{Path, PathBuf},
	process::{Command, Stdio},
};
use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
	sync::{mpsc, oneshot},
};
// cargo run daemon
// Troubleshooting:
//
// Check whether the process is still running:
//   ps -p <PID> -o pid,ppid,stat,command
//   ps -p 35762 -o pid,ppid,stat,command
//
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
pub trait Daemon {
	async fn execute(&mut self, action: ActionRequest) -> Result<DaemonResponse>;
	async fn start(&mut self, options: DaemonOptions) -> Result<DaemonResponse>;
	async fn stop(&mut self) -> Result<DaemonResponse>;
}
/// Verb layer: format, rename, save, index, analyze, build, search, resolve, organize imports, find references, go to definition
pub struct EstateDaemon {
	pub estate: Estate,
	pub actions: ActionRegistry,
	pub discovery: EstateDiscovery,
	// pub vfs: EstateVfs,
	// pub graph: EstateGraph,
	// pub resolver: EstateResolver,
	// pub registry: EstateRegistry,
}
pub struct BackgroundDaemon {
	estate: Estate,
	actions: ActionRegistry,
	discovery: EstateDiscovery,
	context: app::Context,
	workspace: Workspace,
	rx: mpsc::Receiver<DaemonMessage>,
	pub tx: mpsc::Sender<DaemonMessage>,
}
#[async_trait]
impl Daemon for BackgroundDaemon {
	async fn execute(&mut self, action: ActionRequest) -> Result<DaemonResponse> {
		eprintln!("[daemon] execute: {:?}", action);

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
			self.run_foreground().await
		} else {
			self.run_background().await
		}
	}
	async fn stop(&mut self) -> Result<DaemonResponse> {
		todo!("stop")
	}
}
impl BackgroundDaemon {
	pub fn new(workspace: Workspace, context: app::Context) -> Self {
		let (tx, rx) = mpsc::channel(32);
		Self {
			actions: ActionRegistry::default(),
			discovery: EstateDiscovery::default(),
			estate: Estate::default(),
			workspace,
			rx,
			tx,
			context,
		}
		// Self { workspace, rx, tx, context }
	}
	pub async fn run(&mut self) -> Result<()> {
		let _tx = self.tx.clone();
		// tokio::try_join!(
		// 	Self::run_socket_server(SOCKET_PATH, tx),
		// 	self.run_processing_loop(),
		// )?;
		Ok(())
	}
	// cargo run daemon --live
	pub async fn run_foreground(&mut self) -> Result<DaemonResponse> {
		let tx = self.tx.clone();
		let (socket_result, _processing_result) = tokio::join!(
			Self::run_socket_server(SOCKET_PATH, tx),
			self.run_processing_loop(),
		);
		socket_result?;
		Ok(DaemonResponse::default())
	}
	///--------------------------------------------------------------------------------
	/// Long lived runner ready todo something
	/// cargo run daemon
	///--------------------------------------------------------------------------------
	async fn run_background(&mut self) -> Result<DaemonResponse> {
		let exe = std::env::current_exe()?;
		let child = Command::new(&exe)
			.args(["daemon", "--live"])
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
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
	/// Spawns the Unix socket listener and routes incoming connections into the daemon's channel
	pub async fn run_socket_server(socket_path: &str, tx: mpsc::Sender<DaemonMessage>) -> Result<()> {
		let _ = std::fs::remove_file(socket_path);
		let listener = UnixListener::bind(socket_path)?;
		eprintln!("[daemon] listening on {}", socket_path);

		loop {
			let (mut socket, _) = listener.accept().await?;
			let tx = tx.clone();
			tokio::spawn(async move {
				let mut buf = vec![0; 8192];
				let n = socket.read(&mut buf).await?;
				if n == 0 {
					return Ok(());
				}
				let parsed = parse_action(buf, n)?;
				let (resp_tx, resp_rx) = oneshot::channel();
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
				let result = resp_rx.await?;
				let payload = match result {
					Ok(response) => serde_json::to_string(&response)?,
					Err(error) => serde_json::to_string(&serde_json::json!({
							"status": "error",
							"message": error.to_string(),
					}))?,
				};
				socket.write_all(payload.as_bytes()).await?;
				socket.write_all(b"\n").await?;
				Ok::<(), anyhow::Error>(())
			});
		}
	}
	pub async fn run_processing_loop(&mut self) {
		eprintln!("[daemon] processing loop started");
		while let Some(message) = self.rx.recv().await {
			match message {
				DaemonMessage::Execute { action, respond_to } => {
					eprintln!("[daemon] executing action");
					let result = self.execute(action).await;
					eprintln!("[daemon] execute completed");
					let _ = respond_to.send(result);
				}
			}
		}
		eprintln!("[daemon] processing loop stopped");
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
impl CliCommand for StartDaemon {
	async fn run(&self, _ctx: &Context) {
		println!("🚀 starting estate daemon");
		let exe = std::env::current_exe().expect("failed finding current executable");
		let child = std::process::Command::new(exe)
			.arg("daemon")
			.spawn()
			.expect("failed starting daemon");
		println!("✅ daemon started pid={}", child.id());
	}
}
pub async fn daemon() {
	println!("🚀 estate daemon running");
	let runtime = EstateRuntime::new();
	let listener = TcpListener::bind("127.0.0.1:7788").await.unwrap();
	let rx = runtime.events.subscribe();
	tokio::join!(
		serve(listener, runtime.clone()),
		event_loop(rx, runtime.clone())
	);
}
// pub struct BackgroundDaemon2;
// impl BackgroundDaemon2 {
// 	// 1. Find it.
// 	// ps aux | grep "daemon-server"
// 	// 2. Kill it
// 	// kill 17254
// 	// 3. Kill harder
// 	// kill -9 49189
// 	// 4. Remove
// 	// rm /tmp/estate-daemon.sock
// 	// Add stop
// 	// cg-rb loi daemon stop
// 	pub async fn run(_ctx: &Context, _args: &cli::FormatArgs) {
// 		if Self::daemon_running().await {
// 			println!("daemon already running");
// 			return;
// 		}
// 		if Path::new(SOCKET_PATH).exists() {
// 			std::fs::remove_file(SOCKET_PATH).expect("failed removing stale socket");
// 		}
// 		println!("🚀 starting background estate daemon");
// 		let exe = std::env::current_exe().expect("failed finding current executable");
// 		let child = Command::new(exe)
// 			.arg("daemon-server")
// 			.stdin(Stdio::null())
// 			.stdout(Stdio::null())
// 			.stderr(Stdio::null())
// 			.spawn()
// 			.expect("failed spawning daemon");
// 		let pid = child.id();
// 		std::fs::write("/tmp/estate-daemon.pid", pid.to_string()).expect("failed writing pid file");
// 		// 3. Log file.
// 		// use std::fs::OpenOptions;
// 		// use std::process::{Command, Stdio};
// 		// let log = OpenOptions::new()
// 		//     .create(true)
// 		//     .append(true)
// 		//     .open("/tmp/estate-daemon.log")
// 		//     .unwrap();
// 		// let child = Command::new(exe)
// 		//     .arg("daemon-server")
// 		//     .stdin(Stdio::null())
// 		//     .stdout(log.try_clone().unwrap())
// 		//     .stderr(log)
// 		//     .spawn()
// 		//     .unwrap();
// 		println!("✅ daemon started pid={}", child.id());
// 	}
// 	async fn daemon_running() -> bool {
// 		UnixStream::connect(SOCKET_PATH).await.is_ok()
// 	}
// }
// pub struct ProcessDaemon;
// impl ProcessDaemon {
//     pub async fn run(_ctx: &Context, _args: &cli::FormatArgs) {
//         println!("🚀 starting estate daemon");
//         let exe = std::env::current_exe().expect("failed finding current executable");
//         let child = Command::new(exe)
//             .arg("daemon")
//             .stdin(Stdio::null())
//             .stdout(Stdio::null())
//             .stderr(Stdio::null())
//             .spawn()
//             .expect("failed starting daemon");
//         println!("✅ daemon started pid={}", child.id());
//     }
// }
// async fn process_loop() {
//     loop {
//         println!("daemon heartbeat");
//         tokio::time::sleep(std::time::Duration::from_secs(10)).await;
//     }
// }

pub enum DaemonMessage {
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
	let mut s = DaemonState::load();
	s.starts += 1;
	s.started_at = DaemonState::now();
	DaemonState::save(&s);
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
