use crate::daemon::*;
use anyhow::Error;
use async_trait::async_trait;
use cli::{AnalyzeRequest, CliCommand, Context};
use revelation::analyzer::{AnalyzerOptions, Workspace};
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
use tower_lsp::jsonrpc::Response;

///--------------------------------------------------------------------------------
///      Long lived runner ready todo something
///--------------------------------------------------------------------------------
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
// - See current
// ps -p 6550 -o pid, ppid,user,%cpu,%mem,etime,command
// - Watch live
// top -pid 6550
// - Check status
// lsof -p 6550
// - Check if alive
// kill -0 6550
// - KIll
// kill 6550
// kill -9 6550
// 6550
const SOCKET_PATH: &str = "/tmp/estate-daemon.sock";

// async fn handle_client(stream: UnixStream) {
// 	let (reader, mut writer) = stream.into_split();
// 	let mut reader = BufReader::new(reader);
// 	let mut line = String::new();
// 	while reader.read_line(&mut line).await.unwrap() > 0 {
// 		let command = line.trim();
// 		println!("received command: {}", command);
// 		let response = match command {
// 			"status" => "daemon alive\n",
// 			"shutdown" => "shutdown requested\n",
// 			"hello" => "hello from daemon\n",
// 			_ => "unknown command\n",
// 		};
// 		writer.write_all(response.as_bytes()).await.unwrap();
// 		line.clear();
// 	}
// }
// async fn daemon_running() -> bool {
// 	UnixStream::connect(SOCKET_PATH).await.is_ok()
// }

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
///
///
///
///
///
///
///
/// Daemon domain API — execute(Action) -> Response, start(), stop().
/// Daemon transport/lifecycle — Unix socket, Tokio tasks, channels, request parsing.
#[async_trait]
pub trait Daemon {
	async fn execute(&mut self, action: ActionRequest) -> anyhow::Result<DaemonResponse>;
	async fn start(&mut self, options: DaemonOptions) -> anyhow::Result<DaemonResponse>;
	async fn stop(&mut self) -> anyhow::Result<DaemonResponse>;
}
pub struct BackgroundDaemon {
	workspace: Workspace,
	rx: mpsc::Receiver<DaemonMessage>,
	tx: mpsc::Sender<DaemonMessage>,
}
#[async_trait]
impl Daemon for BackgroundDaemon {
	async fn execute(&mut self, action: ActionRequest) -> anyhow::Result<DaemonResponse> {
		match action {
			ActionRequest::Analyze {
				path,
				line,
				column,
				mode,
			} => self.analyze(path, line, column, mode),
			ActionRequest::ScanWorkspace { path } => self.scan_workspace(path),
			ActionRequest::InitializeEstate { path } => self.initialize_estate(path),
		}
	}
	async fn start(&mut self, options: DaemonOptions) -> anyhow::Result<DaemonResponse> {
		if options.foreground {
			self.run_foreground().await
		} else {
			self.run_background().await
		}
	}
	async fn stop(&mut self) -> anyhow::Result<DaemonResponse> {
		todo!("stop")
	}
}
impl BackgroundDaemon {
	pub fn new(workspace: Workspace) -> Self {
		let (tx, rx) = mpsc::channel(32);
		Self { workspace, rx, tx }
	}
	async fn run_foreground(&mut self) -> anyhow::Result<DaemonResponse> {
		let socket_path = "/tmp/loi_daemon.sock";
		let tx = self.tx.clone();
		let _result = tokio::join!(
			Self::run_socket_server(socket_path, tx),
			self.run_processing_loop(),
		);
		Ok(DaemonResponse::default())
	}
	async fn run_background(&mut self) -> anyhow::Result<DaemonResponse> {
		let exe = std::env::current_exe()?;
		std::process::Command::new(exe)
			.args(["daemon", "--live"])
			.spawn()?;
		Ok(DaemonResponse::default())
	}
	/// Spawns the Unix socket listener and routes incoming connections into the daemon's channel
	pub async fn run_socket_server(
		socket_path: &str,
		tx: mpsc::Sender<DaemonMessage>,
	) -> anyhow::Result<()> {
		let _ = std::fs::remove_file(socket_path);
		let listener = UnixListener::bind(socket_path)?;
		loop {
			let (mut socket, _) = listener.accept().await?;
			let tx = tx.clone();
			tokio::spawn(async move {
				let mut buf = vec![0; 8192];

				let n = socket.read(&mut buf).await?;
				if n == 0 {
					return Ok(());
				}
				let raw_data = String::from_utf8_lossy(&buf[..n]);
				let raw_trimmed = raw_data.trim();
				let parsed: IncomingRequest = serde_json::from_str(raw_trimmed).or_else(|_| {
					let unquoted = raw_trimmed.trim_matches('"').replace("\\\"", "\"");
					serde_json::from_str(&unquoted)
				})?;
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
		while let Some(message) = self.rx.recv().await {
			match message {
				DaemonMessage::Execute { action, respond_to } => {
					let result = self.execute(action).await;
					let _ = respond_to.send(result);
				}
			}
		}
	}
	fn analyze(
		&mut self,
		path: PathBuf,
		line: Option<u32>,
		column: Option<u32>,
		mode: Option<String>,
	) -> anyhow::Result<DaemonResponse> {
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
		let report =
			revelation::analyzer::Workspace::analyze_ownership_on_click(&system_path, &options)?;
		Ok(DaemonResponse {
			data: Some(serde_json::to_value(report)?),
			..Default::default()
		})
	}
	fn scan_workspace(&mut self, _path: PathBuf) -> anyhow::Result<DaemonResponse> {
		todo!("scan_workspace")
	}
	fn initialize_estate(&mut self, _path: PathBuf) -> anyhow::Result<DaemonResponse> {
		todo!("initialize_estate")
	}

	// Your core processing loop handling mpsc messages
	// pub async fn run_processing_loop2(&mut self, live: bool) {
	// 	if live {
	// 		eprintln!("Background daemon queue processor started...");
	// 	}
	// 	while let Some(request) = self.rx.recv().await {
	// 		match request {
	// 			AnalyzeRequest::RunAnalysis {
	// 				path,
	// 				line,
	// 				column,
	// 				mode,
	// 				respond_to,
	// 			} => {
	// 				if live {
	// 					eprintln!("-> Processing ownership analysis for path: {:?}", path);
	// 				}
	// 				let options = AnalyzerOptions {
	// 					line,
	// 					column,
	// 					mode,
	// 					include_private: true,
	// 					include_tests: true,
	// 				};
	// 				let system_path = match path.to_str() {
	// 					Some(s) if s.starts_with("file://") => match url::Url::parse(s) {
	// 						Ok(url) => match url.to_file_path() {
	// 							Ok(path) => path,
	// 							Err(_) => {
	// 								let _ = respond_to.send(
	// 									serde_json::json!({
	// 											"status": "error",
	// 											"message": "Invalid file URI"
	// 									})
	// 									.to_string(),
	// 								);
	// 								return;
	// 							}
	// 						},
	// 						Err(e) => {
	// 							let _ = respond_to.send(
	// 								serde_json::json!({
	// 										"status": "error",
	// 										"message": e.to_string()
	// 								})
	// 								.to_string(),
	// 							);
	// 							return;
	// 						}
	// 					},
	// 					_ => path.clone(),
	// 				};
	// 				let analysis_result =
	// 					revelation::analyzer::Workspace::analyze_ownership_on_click(&system_path, &options);

	// 				match &analysis_result {
	// 					Ok(report) => {
	// 						if live {
	// 							eprintln!("{}", report.formatted_output);
	// 						}
	// 					}
	// 					Err(e) => {
	// 						eprintln!("===== ANALYSIS ERROR =====");
	// 						eprintln!("{:?}", e);
	// 						eprintln!("==========================");
	// 					}
	// 				}

	// 				let response_string = match analysis_result {
	// 					Ok(report) => serde_json::to_string(&report).unwrap_or_else(|e| {
	// 						serde_json::json!({
	// 								"status": "error",
	// 								"message": e.to_string()
	// 						})
	// 						.to_string()
	// 					}),
	// 					Err(e) => serde_json::json!({
	// 							"status": "error",
	// 							"message": e.to_string()
	// 					})
	// 					.to_string(),
	// 				};
	// 				respond_to.send(response_string);
	// 			}
	// 		}
	// 	}
	// }
}
#[derive(Debug, Default)]
pub struct DaemonOptions {
	pub foreground: bool,
}
#[derive(Debug, serde::Deserialize)]
pub struct SocketPayload {
	pub path: PathBuf,
	pub subject: Option<AnalyzeSubjectDto>,
}
#[derive(Debug, serde::Deserialize)]
pub struct AnalyzeSubjectDto {
	pub offset: usize,
	pub identifier: Option<String>,
}
#[derive(Deserialize, Debug)]
struct IncomingRequest {
	path: PathBuf,
	line: Option<u32>,
	column: Option<u32>,
	mode: Option<String>,
}
pub struct DaemonError {}
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
pub struct BackgroundDaemon2;
impl BackgroundDaemon2 {
	// 1. Find it.
	// ps aux | grep "daemon-server"
	// 2. Kill it
	// kill 17254
	// 3. Kill harder
	// kill -9 49189
	// 4. Remove
	// rm /tmp/estate-daemon.sock
	// Add stop
	// cg-rb loi daemon stop
	pub async fn run(_ctx: &Context, _args: &cli::FormatArgs) {
		if Self::daemon_running().await {
			println!("daemon already running");
			return;
		}
		if Path::new(SOCKET_PATH).exists() {
			std::fs::remove_file(SOCKET_PATH).expect("failed removing stale socket");
		}
		println!("🚀 starting background estate daemon");
		let exe = std::env::current_exe().expect("failed finding current executable");
		let child = Command::new(exe)
			.arg("daemon-server")
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.expect("failed spawning daemon");
		let pid = child.id();
		std::fs::write("/tmp/estate-daemon.pid", pid.to_string()).expect("failed writing pid file");
		// 3. Log file.
		// use std::fs::OpenOptions;
		// use std::process::{Command, Stdio};
		// let log = OpenOptions::new()
		//     .create(true)
		//     .append(true)
		//     .open("/tmp/estate-daemon.log")
		//     .unwrap();
		// let child = Command::new(exe)
		//     .arg("daemon-server")
		//     .stdin(Stdio::null())
		//     .stdout(log.try_clone().unwrap())
		//     .stderr(log)
		//     .spawn()
		//     .unwrap();
		println!("✅ daemon started pid={}", child.id());
	}
	async fn daemon_running() -> bool {
		UnixStream::connect(SOCKET_PATH).await.is_ok()
	}
}
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
		respond_to: oneshot::Sender<anyhow::Result<DaemonResponse>>,
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
