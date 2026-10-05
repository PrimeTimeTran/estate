use crate::prelude::{
	keymap::{Action, Binding, Context},
	shared::Binding as OldKeyBinding,
	*,
};
use anyhow::{Context as CtxAnyhow, Result};
use mach2::mach_time;
use std::process::{Child, Command, Stdio};

pub struct IpcServer {
	socket: PathBuf,
	events: EventBus,
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
			"Estate IPC listening"
		);

		loop {
			let (stream, _) = listener.accept().await?;

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

async fn handle_connection(stream: UnixStream, events: EventBus) -> anyhow::Result<()> {
	let connection_id = Uuid::new_v4();

	let (read_half, mut write_half) = stream.into_split();
	let mut reader = BufReader::new(read_half);

	let mut line = String::new();
	reader.read_line(&mut line).await?;

	let message: IpcMessage<EventKind> = serde_json::from_str(&line)?;

	let hello = match message {
		IpcMessage::Hello(hello) => hello,

		_ => {
			send(
				&mut write_half,
				IpcMessage::Error(estate_ipc::IpcError {
					code: estate_ipc::IpcErrorCode::InvalidMessage,
					message: "expected Hello".into(),
				}),
			)
			.await?;

			anyhow::bail!("first IPC message was not Hello");
		}
	};

	line.clear();

	if hello.protocol.major != ProtocolVersion::CURRENT.major {
		send(
			&mut write_half,
			IpcMessage::Error(estate_ipc::IpcError {
				code: estate_ipc::IpcErrorCode::ProtocolMismatch,
				message: format!(
					"unsupported protocol {}.{}",
					hello.protocol.major, hello.protocol.minor
				),
			}),
		)
		.await?;

		anyhow::bail!("IPC protocol mismatch");
	}

	tracing::info!(
		connection = %connection_id,
		client = ?hello.client,
		pid = hello.pid,
		protocol_major = hello.protocol.major,
		protocol_minor = hello.protocol.minor,
		"Estate IPC client connected"
	);

	send(
		&mut write_half,
		IpcMessage::HelloAck(HelloAck {
			protocol: ProtocolVersion::CURRENT,
			server: ClientKind::Daemon,
			connection_id,
		}),
	)
	.await?;

	let mut event_rx = events.tx.subscribe();

	loop {
		tokio::select! {
			/*
			 * Tauri → Estate
			 */
			result = reader.read_line(&mut line) => {
				let bytes = result?;

				if bytes == 0 {
					break;
				}

				tracing::debug!(
					connection = %connection_id,
					line = %line.trim_end(),
					"🔥 Estate IPC ← client"
				);

				let message: IpcMessage<EventKind> =
					match serde_json::from_str(&line) {
						Ok(message) => message,

						Err(error) => {
							tracing::error!(
								connection = %connection_id,
								%error,
								line = %line.trim_end(),
								"🔥 Estate IPC failed to decode client message"
							);

							return Err(error.into());
						}
					};
				tracing::info!(
					connection = %connection_id,
					"🔥 Estate IPC received client message"
				);
				match message {
					IpcMessage::Ping { id } => {
						send(
							&mut write_half,
							IpcMessage::Pong { id },
						)
						.await?;
					}

					IpcMessage::Shutdown => {
						break;
					}

					IpcMessage::GetContext => {
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

						send(
							&mut write_half,
							IpcMessage::ContextResult(context),
						)
						.await?;
					}

					message => {
						tracing::debug!(
							connection = %connection_id,
							?message,
							"Estate IPC message"
						);
					}
				}

				line.clear();
			}
			/*
			 * Estate → Tauri
			 */
			result = event_rx.recv() => {
				match result {
					Ok(event) => {
						tracing::debug!(
							connection = %connection_id,
							?event,
							"📡 Estate IPC → event"
						);
						println!(
							"🔥 IPC SERVER → EVENT BUS EVENT: {:?}",
							event.kind
						);
						let envelope = EventEnvelope {
									id: EventId {
										node: NodeId,
										sequence: event.id,
									},
									timestamp: event.timestamp,
									source: event.source,
									event: event.kind,
								};

						send(
							&mut write_half,
							IpcMessage::Event(envelope),
						)
						.await?;
					}

					Err(
						tokio::sync::broadcast::error::RecvError::Lagged(count)
					) => {
						tracing::warn!(
							connection = %connection_id,
							count,
							"Estate IPC event subscriber lagged"
						);
					}
					Err(
						tokio::sync::broadcast::error::RecvError::Closed
					) => {
						tracing::debug!(
							connection = %connection_id,
							"Estate EventBus closed"
						);
						break;
					}
				}
			}
		}
	}
	tracing::info!(
		connection = %connection_id,
		"Estate IPC client disconnected"
	);
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
