use crate::prelude::*;

pub struct DaemonServer;

impl DaemonServer {
	pub async fn run() {
		println!("🟢 daemon server running");
		if Path::new(ESTATE_DAEMON_SOCKET).exists() {
			std::fs::remove_file(ESTATE_DAEMON_SOCKET).unwrap();
		}
		let listener = UnixListener::bind(ESTATE_DAEMON_SOCKET).expect("failed binding socket");
		println!("listening on {}", ESTATE_DAEMON_SOCKET);
		loop {
			let (stream, _) = listener.accept().await.expect("accept failed");
			tokio::spawn(async move {
				Self::handle_client(stream).await;
			});
		}
	}
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
}
