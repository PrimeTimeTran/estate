use crate::daemon::*;
use cli::{AnalyzeRequest, CliCommand, Context};
use revelation::analyzer::{AnalyzerOptions, Workspace};
use serde::Deserialize;
use std::path::Path;
use std::{
	io::Write,
	process::{Command, Stdio},
};
use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
};
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
///--------------------------------------------------------------------------------
///--------------------------------------------------------------------------------
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
		if daemon_running().await {
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
async fn daemon_running() -> bool {
	UnixStream::connect(SOCKET_PATH).await.is_ok()
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
				handle_client(stream).await;
			});
		}
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
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
pub struct BackgroundDaemon {
	workspace: Workspace,
	rx: mpsc::Receiver<AnalyzeRequest>,
	tx: mpsc::Sender<AnalyzeRequest>,
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
impl BackgroundDaemon {
	pub fn new(workspace: Workspace) -> (Self, mpsc::Sender<AnalyzeRequest>) {
		let (tx, rx) = mpsc::channel(32);
		let daemon = Self {
			workspace,
			rx,
			tx: tx.clone(),
		};
		(daemon, tx)
	}
	/// Returns a cloned sender handle that can be handed to socket tasks
	pub fn get_sender(&self) -> mpsc::Sender<AnalyzeRequest> {
		self.tx.clone()
	}
	/// Spawns the Unix socket listener and routes incoming connections into the daemon's channel
	pub async fn run_socket_server(
		mut self,
		socket_path: &str,
		live: bool,
	) -> Result<(), Box<dyn std::error::Error>> {
		let _ = std::fs::remove_file(socket_path);
		let listener = UnixListener::bind(socket_path)?;
		if live {
			eprintln!("[DAEMON LIVE] Listening on socket: {}", socket_path);
		}
		// let payload = format!("{}\n", path.display());
		// Clone the sender so we can hand it to incoming socket connections
		let tx = self.get_sender(); // Assuming you expose or keep a handle to `tx`
		// Spawn the internal processing loop that handles `self.rx`
		let _processing_handle = tokio::spawn(async move {
			self.run_processing_loop(live).await;
		});
		loop {
			let (mut socket, _) = listener.accept().await?;
			let tx_clone = tx.clone();
			tokio::spawn(async move {
				let mut buf = vec![0; 8192];
				if let Ok(n) = socket.read(&mut buf).await
					&& n > 0
				{
					let raw_data = String::from_utf8_lossy(&buf[..n]);
					let raw_trimmed = raw_data.trim();
					let parsed: Result<IncomingRequest, _> =
						serde_json::from_str(raw_trimmed).or_else(|_| {
							let unquoted = raw_trimmed.trim_matches('"').replace("\\\"", "\"");
							serde_json::from_str(&unquoted)
						});
					let (path, line, column, mode) = match parsed {
						Ok(req) => (req.path, req.line, req.column, req.mode),
						Err(_) => {
							todo!("Daemon run socket server issue parsing")
						}
					};
					let (resp_tx, resp_rx) = oneshot::channel();
					if tx_clone
						.send(AnalyzeRequest::RunAnalysis {
							path,
							line,
							column,
							mode,
							respond_to: resp_tx,
						})
						.await
						.is_ok()
						&& let Ok(result_msg) = resp_rx.await
					{
						let response = format!("{}\n", result_msg);
						let _ = socket.write_all(response.as_bytes()).await;
					}
					// if let Ok(result_msg) = resp_rx.await {
					// 	socket.write_all(result_msg.as_bytes()).await?;
					// 	socket.shutdown().await?;
					// }
				}
			});
		}
	}
	/// Your core processing loop handling mpsc messages
	pub async fn run_processing_loop(&mut self, live: bool) {
		if live {
			eprintln!("Background daemon queue processor started...");
		}
		while let Some(request) = self.rx.recv().await {
			match request {
				AnalyzeRequest::RunAnalysis {
					path,
					line,
					column,
					mode,
					respond_to,
				} => {
					if live {
						eprintln!("-> Processing ownership analysis for path: {:?}", path);
					}
					let options = AnalyzerOptions {
						line,
						column,
						mode,
						include_private: true,
						include_tests: true,
					};
					let system_path = match path.to_str() {
						Some(s) if s.starts_with("file://") => match url::Url::parse(s) {
							Ok(url) => match url.to_file_path() {
								Ok(path) => path,
								Err(_) => {
									let _ = respond_to.send(
										serde_json::json!({
												"status": "error",
												"message": "Invalid file URI"
										})
										.to_string(),
									);
									return;
								}
							},
							Err(e) => {
								let _ = respond_to.send(
									serde_json::json!({
											"status": "error",
											"message": e.to_string()
									})
									.to_string(),
								);
								return;
							}
						},
						_ => path.clone(),
					};
					let analysis_result =
						revelation::analyzer::Workspace::analyze_ownership_on_click(&system_path, &options);

					match &analysis_result {
						Ok(report) => {
							if live {
								eprintln!("{}", report.formatted_output);
							}
						}
						Err(e) => {
							eprintln!("===== ANALYSIS ERROR =====");
							eprintln!("{:?}", e);
							eprintln!("==========================");
						}
					}

					let response_string = match analysis_result {
						Ok(report) => serde_json::to_string(&report).unwrap_or_else(|e| {
							serde_json::json!({
									"status": "error",
									"message": e.to_string()
							})
							.to_string()
						}),
						Err(e) => serde_json::json!({
								"status": "error",
								"message": e.to_string()
						})
						.to_string(),
					};
					respond_to.send(response_string);
				}
			}
		}
	}
}
