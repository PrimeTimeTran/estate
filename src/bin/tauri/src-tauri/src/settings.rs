use estate::prelude::{
	ipc::{EstateClient, *},
	*,
};
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tauri::command]
pub fn start_estate_event_bridge(app: tauri::AppHandle) {
	println!("🔥 ESTATE EVENT BRIDGE → STARTING");
	tokio::spawn(async move {
		println!("🔥 ESTATE EVENT BRIDGE → connecting");
		let stream = match UnixStream::connect(ESTATE_IPC_SOCKET).await {
			Ok(stream) => stream,
			Err(error) => {
				eprintln!("🔥 ESTATE EVENT BRIDGE → connect failed: {error}");
				return;
			}
		};
		println!("🔥 ESTATE EVENT BRIDGE → connected");
		let (reader, mut writer) = stream.into_split();
		let mut reader = BufReader::new(reader);
		// Keep the write half alive for the lifetime of this connection.
		// The daemon treats EOF on the client read side as disconnect.
		let _keepalive = &mut writer;
		let mut line = String::new();

		// Establish IPC connection.
		let hello = IpcMessage::<EventKind>::Hello(Hello {
			protocol: ProtocolVersion::CURRENT,
			client: ClientKind::Tauri,
			pid: std::process::id(),
		});

		let json = match serde_json::to_string(&hello) {
			Ok(json) => json,
			Err(error) => {
				eprintln!("🔥 ESTATE EVENT BRIDGE → serialize Hello failed: {error}");
				return;
			}
		};

		if let Err(error) = writer.write_all(json.as_bytes()).await {
			eprintln!("🔥 ESTATE EVENT BRIDGE → Hello write failed: {error}");
			return;
		}

		if let Err(error) = writer.write_all(b"\n").await {
			eprintln!("🔥 ESTATE EVENT BRIDGE → newline write failed: {error}");
			return;
		}

		if let Err(error) = writer.flush().await {
			eprintln!("🔥 ESTATE EVENT BRIDGE → Hello flush failed: {error}");
			return;
		}

		// Read HelloAck.
		line.clear();

		match reader.read_line(&mut line).await {
			Ok(0) => {
				eprintln!("🔥 ESTATE EVENT BRIDGE → daemon disconnected during Hello");
				return;
			}

			Ok(_) => {
				println!("🔥 ESTATE EVENT BRIDGE ← {}", line.trim_end());
			}

			Err(error) => {
				eprintln!("🔥 ESTATE EVENT BRIDGE → HelloAck failed: {error}");
				return;
			}
		}
		let _ = &writer;
		// Long-lived event stream.
		loop {
			line.clear();

			let bytes = match reader.read_line(&mut line).await {
				Ok(bytes) => bytes,

				Err(error) => {
					eprintln!("🔥 ESTATE EVENT BRIDGE → read failed: {error}");
					return;
				}
			};

			if bytes == 0 {
				println!("🔥 ESTATE EVENT BRIDGE → daemon disconnected");
				return;
			}

			let message: IpcMessage<estate::event::EventKind> =
				match serde_json::from_str(line.trim_end()) {
					Ok(message) => message,

					Err(error) => {
						eprintln!("🔥 ESTATE EVENT BRIDGE → decode failed: {error}");
						continue;
					}
				};

			match message {
				IpcMessage::Event(envelope) => {
					println!("🔥 ESTATE EVENT BRIDGE ← EVENT: {:?}", envelope.event);

					if let Err(error) = app.emit("estate-event", &envelope) {
						eprintln!("🔥 ESTATE EVENT BRIDGE → emit failed: {error}");
					}
				}

				other => {
					println!("🔥 ESTATE EVENT BRIDGE ← {:?}", other);
				}
			}
		}
	});
}

#[tauri::command]
pub fn tauri_context() -> Result<TauriContext, String> {
	Ok(TauriContext {
		pid: std::process::id(),
		platform: std::env::consts::OS.into(),
		arch: std::env::consts::ARCH.into(),
		cwd: std::env::current_dir()
			.map_err(|e| e.to_string())?
			.display()
			.to_string(),
	})
}

#[tauri::command]
pub async fn estate_context(
	client: tauri::State<'_, EstateClient>,
) -> Result<EstateContext, String> {
	println!("🔥 TAURI IPC → estate_context");

	let mut connection = client
		.connect()
		.await
		.map_err(|e| format!("Estate connect failed: {e}"))?;

	println!("🔥 TAURI IPC → EstateClient::connect() RETURNED");
	println!(
		"🔥 TAURI IPC → connection_id={}",
		connection.connection_id()
	);

	connection
		.context()
		.await
		.map_err(|e| format!("Estate context failed: {e}"))
}

#[tauri::command]
pub async fn estate_command(
	client: tauri::State<'_, EstateClient>,
	command: EstateCommand,
) -> Result<EstateCommandResult, String> {
	println!("🔥 TAURI IPC → estate_command: {command:?}");

	let mut connection = client
		.connect()
		.await
		.map_err(|e| format!("Estate connect failed: {e}"))?;

	println!(
		"🔥 TAURI IPC → command connection_id={}",
		connection.connection_id()
	);

	let result = connection
		.run_command(command)
		.await
		.map_err(|e| format!("Estate command failed: {e}"))?;

	println!("🔥 TAURI IPC ← estate_command: {result:?}");

	Ok(result)
}

#[tauri::command]
pub async fn fs_list(
	client: tauri::State<'_, EstateClient>,
	path: String,
) -> Result<Vec<FileEntry>, String> {
	let mut connection = client.connect().await.map_err(|e| e.to_string())?;

	connection.fs_list(path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fs_read(
	client: tauri::State<'_, EstateClient>,
	path: String,
) -> Result<String, String> {
	let mut connection = client.connect().await.map_err(|e| e.to_string())?;

	connection.fs_read(path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fs_create(
	client: tauri::State<'_, EstateClient>,
	path: String,
	content: String,
) -> Result<(), String> {
	let mut connection = client.connect().await.map_err(|e| e.to_string())?;

	connection
		.fs_create(path, content)
		.await
		.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fs_update(
	client: tauri::State<'_, EstateClient>,
	path: String,
	content: String,
) -> Result<(), String> {
	let mut connection = client.connect().await.map_err(|e| e.to_string())?;

	connection
		.fs_update(path, content)
		.await
		.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fs_delete(client: tauri::State<'_, EstateClient>, path: String) -> Result<(), String> {
	let mut connection = client.connect().await.map_err(|e| e.to_string())?;

	connection.fs_delete(path).await.map_err(|e| e.to_string())
}
