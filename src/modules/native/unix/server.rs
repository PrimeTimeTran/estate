use cli::context::Context as CliContext;

use crate::prelude::*;

impl DaemonServer {
	pub async fn run() {
		println!("🟢 daemon server running");
		if Path::new(PATH_SOCKET).exists() {
			std::fs::remove_file(PATH_SOCKET).unwrap();
		}
		let listener = UnixListener::bind(PATH_SOCKET).expect("failed binding socket");
		println!("listening on {}", PATH_SOCKET);
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

pub struct StatusDaemon;
#[async_trait::async_trait]
impl CliCommand for StatusDaemon {
	async fn run(&self, _ctx: &CliContext) {
		let state = EstateState::load_from_disk().unwrap();
		let pid =
			std::fs::read_to_string(crate::data::PATH_PID).unwrap_or_else(|_| "unknown".to_string());
		println!("📊 Estate Daemon Status");
		println!("──────────────────────");
		println!("✅ Status:          OK");
		println!("🆔 PID:             {}", pid);
		println!("🚀 Starts:          {}", state.starts);
		println!("🔎 Status checks:   {}", state.status_checks);
		println!("🕒 Started at:      {}", state.started_at);
		println!("⏱ Longest run:     {}s", state.longest_run);
		match tokio::net::UnixStream::connect(PATH_SOCKET).await {
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
