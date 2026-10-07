use crate::prelude::*;
use crate::prelude::{shared::Binding as OldKeyBinding, *};
use anyhow::{Context as CtxAnyhow, Result};

async fn handle_connection(stream: UnixStream, events: EventBus) -> anyhow::Result<()> {
	tracing::info!("🔥 IPC HANDLE CONNECTION");
	let connection_id = Uuid::new_v4();

	let (read_half, mut write_half) = stream.into_split();
	let mut reader = BufReader::new(read_half);

	tracing::info!("🔥 IPC WAITING FOR CLIENT MESSAGE");
	let hello = read_hello(&mut reader, &mut write_half).await?;

	tracing::info!(
		connection = %connection_id,
		hello = ?hello,
		"🔥 IPC HELLO RECEIVED"
	);

	log_client_connected(connection_id, &hello);

	send_hello_ack(&mut write_half, connection_id).await?;

	tracing::info!(
		connection = %connection_id,
		"🔥 IPC HELLO ACK SENT"
	);

	let mut event_rx = events.tx.subscribe();
	let mut line = String::new();

	run_connection_loop(
		connection_id,
		&mut reader,
		&mut write_half,
		&mut event_rx,
		&mut line,
	)
	.await?;

	tracing::debug!(
		connection = %connection_id,
		"Estate IPC client disconnected"
	);

	Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Handshake
// ─────────────────────────────────────────────────────────────────────────────
async fn read_hello(
	reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<Hello> {
	let mut line = String::new();

	reader.read_line(&mut line).await?;

	let message: IpcMessage<EventKind> = serde_json::from_str(&line)?;

	match message {
		IpcMessage::Hello(hello) => {
			validate_protocol(&hello, write_half).await?;
			Ok(hello)
		}

		_ => {
			send(
				write_half,
				IpcMessage::Error(IpcError {
					code: IpcErrorCode::InvalidMessage,
					message: "expected Hello".into(),
				}),
			)
			.await?;

			anyhow::bail!("first IPC message was not Hello");
		}
	}
}
async fn validate_protocol(
	hello: &Hello,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<()> {
	if hello.protocol.major == ProtocolVersion::CURRENT.major {
		return Ok(());
	}

	send(
		write_half,
		IpcMessage::Error(IpcError {
			code: IpcErrorCode::ProtocolMismatch,
			message: format!(
				"unsupported protocol {}.{}",
				hello.protocol.major, hello.protocol.minor
			),
		}),
	)
	.await?;

	anyhow::bail!("IPC protocol mismatch");
}
fn log_client_connected(connection_id: Uuid, hello: &Hello) {
	tracing::debug!(
		connection = %connection_id,
		client = ?hello.client,
		pid = hello.pid,
		protocol_major = hello.protocol.major,
		protocol_minor = hello.protocol.minor,
		"Estate IPC client connected"
	);
}
async fn send_hello_ack(
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
	connection_id: Uuid,
) -> anyhow::Result<()> {
	send(
		write_half,
		IpcMessage::HelloAck(HelloAck {
			protocol: ProtocolVersion::CURRENT,
			server: ClientKind::Daemon,
			connection_id,
		}),
	)
	.await?;

	Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Connection loop
// ─────────────────────────────────────────────────────────────────────────────
async fn run_connection_loop(
	connection_id: Uuid,
	reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
	event_rx: &mut tokio::sync::broadcast::Receiver<Event>,
	line: &mut String,
) -> anyhow::Result<()> {
	tracing::info!("🔥 run_connection_loop HID RAW ← Swift: {}", line);
	loop {
		tokio::select! {
			/*
			 * Tauri → Estate
			 */
			result = reader.read_line(line) => {
			tracing::info!("🔥 run_connection_loop HID RAW ← Swift: {}", line);
				if !handle_client_input(
					connection_id,
					result?,
					line,
					write_half,
				)
				.await?
				{
					break;
				}

				line.clear();
			}

			/*
			 * Estate → Tauri
			 */
			result = event_rx.recv() => {
				if !handle_event_bus_message(
					connection_id,
					result,
					write_half,
				)
				.await?
				{
					break;
				}
			}
		}
	}

	Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tauri → Estate
// ─────────────────────────────────────────────────────────────────────────────
async fn handle_client_input(
	connection_id: Uuid,
	bytes: usize,
	line: &str,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<bool> {
	if bytes == 0 {
		return Ok(false);
	}

	tracing::debug!(
		connection = %connection_id,
		line = %line.trim_end(),
		"🔥 Estate IPC ← client"
	);

	let message = decode_client_message(connection_id, line)?;

	tracing::debug!(
		connection = %connection_id,
		"🔥 Estate IPC received client message"
	);

	handle_client_message(connection_id, message, write_half).await
}
fn decode_client_message(connection_id: Uuid, line: &str) -> anyhow::Result<IpcMessage<EventKind>> {
	match serde_json::from_str(line) {
		Ok(message) => Ok(message),

		Err(error) => {
			tracing::error!(
				connection = %connection_id,
				%error,
				line = %line.trim_end(),
				"🔥 Estate IPC failed to decode client message"
			);

			Err(error.into())
		}
	}
}
async fn handle_client_message(
	connection_id: Uuid,
	message: IpcMessage<EventKind>,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<bool> {
	match message {
		IpcMessage::Ping { id } => {
			handle_ping(write_half, id).await?;
		}

		IpcMessage::Shutdown => {
			return Ok(false);
		}

		IpcMessage::GetContext => {
			handle_get_context(connection_id, write_half).await?;
		}

		message => {
			tracing::debug!(
				connection = %connection_id,
				?message,
				"Estate IPC message"
			);
		}
	}

	Ok(true)
}
async fn handle_ping(
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
	id: u64,
) -> anyhow::Result<()> {
	send(write_half, IpcMessage::Pong { id }).await?;

	Ok(())
}
async fn handle_get_context(
	connection_id: Uuid,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<()> {
	tracing::info!(
		connection = %connection_id,
		"🔥 Estate IPC → sending ContextResult"
	);

	let context = EstateContext {
		connection_id,
		active_app: "Unknown".into(),
		workspace: None,
		project: None,
		mode: "Unknown".into(),
	};

	println!(
		"🔥 ESTATE CLIENT → GET CONTEXT JSON: {}",
		serde_json::to_string(&IpcMessage::<EventKind>::GetContext)?
	);

	send(write_half, IpcMessage::ContextResult(context)).await?;

	Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Estate → Tauri
// ─────────────────────────────────────────────────────────────────────────────
async fn handle_event_bus_message(
	connection_id: Uuid,
	result: Result<Event, tokio::sync::broadcast::error::RecvError>,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<bool> {
	match result {
		Ok(event) => {
			send_event(connection_id, event, write_half).await?;
			Ok(true)
		}

		Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
			tracing::warn!(
				connection = %connection_id,
				count,
				"Estate IPC event subscriber lagged"
			);

			Ok(true)
		}

		Err(tokio::sync::broadcast::error::RecvError::Closed) => {
			tracing::debug!(
				connection = %connection_id,
				"Estate EventBus closed"
			);

			Ok(false)
		}
	}
}

async fn send_event(
	connection_id: Uuid,
	event: e::Event,
	write_half: &mut tokio::net::unix::OwnedWriteHalf,
) -> anyhow::Result<()> {
	tracing::debug!(
		connection = %connection_id,
		?event,
		"📡 Estate IPC → event"
	);
	println!("🔥 IPC SERVER → EVENT BUS EVENT: {:?}", event.kind);
	let envelope = EventEnvelope {
		id: EventId {
			node: NodeId,
			sequence: event.id,
		},
		timestamp: event.timestamp,
		source: event.source,
		event: event.kind,
	};
	send(write_half, IpcMessage::Event(envelope)).await?;
	Ok(())
}
async fn send(
	writer: &mut tokio::net::unix::OwnedWriteHalf,
	message: IpcMessage<EventKind>,
) -> anyhow::Result<()> {
	let json = serde_json::to_string(&message)?;
	writer.write_all(json.as_bytes()).await?;
	writer.write_all(b"\n").await?;
	writer.flush().await?;
	Ok(())
}

impl IpcServer {
	pub fn new(socket: impl Into<PathBuf>, events: EventBus) -> Self {
		Self {
			socket: socket.into(),
			events,
		}
	}
	pub fn init<C>(&self, worker: &HostWorker<C>) -> Result<()>
	where
		C: Ctx,
	{
		// Unix socket setup
		// accept loop
		// connection handling
		// handshake
		// event forwarding
		// commands
		Ok(())
	}
	pub async fn start(&self) -> anyhow::Result<()> {
		if self.socket.exists() {
			tokio::fs::remove_file(&self.socket).await?;
		}
		let listener = UnixListener::bind(&self.socket)?;
		tracing::info!(
			socket = %self.socket.display(),
			"🛜 Estate IPC listening"
		);
		loop {
			let (stream, _) = listener.accept().await?;
			tracing::info!("🔥 IPC ACCEPTED connection");
			let events = self.events.clone();
			tokio::spawn(async move {
				if let Err(error) = handle_connection(stream, events).await {
					tracing::debug!(
						%error,
						"Estate IPC connection closed"
					);
				}
			});
		}
	}
}

pub struct IpcServer {
	socket: PathBuf,
	events: EventBus,
}
