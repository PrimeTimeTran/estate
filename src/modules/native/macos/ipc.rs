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
}

impl IpcServer {
	pub fn new(socket: impl Into<PathBuf>) -> Self {
		Self {
			socket: socket.into(),
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

			tokio::spawn(async move {
				if let Err(error) = handle_connection(stream).await {
					tracing::debug!(
						%error,
						"Estate IPC connection closed"
					);
				}
			});
		}
	}
}

async fn handle_connection(stream: UnixStream) -> anyhow::Result<()> {
	let connection_id = Uuid::new_v4();

	let (read_half, mut write_half) = stream.into_split();
	let mut reader = BufReader::new(read_half);

	let mut line = String::new();

	reader.read_line(&mut line).await?;

	let message: IpcMessage = serde_json::from_str(&line)?;

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

	loop {
		line.clear();

		let bytes = reader.read_line(&mut line).await?;

		if bytes == 0 {
			break;
		}

		let message: IpcMessage = serde_json::from_str(&line)?;

		match message {
			IpcMessage::Ping { id } => {
				send(&mut write_half, IpcMessage::Pong { id }).await?;
			}

			IpcMessage::Shutdown => {
				break;
			}

			message => {
				tracing::debug!(
					connection = %connection_id,
					?message,
					"Estate IPC message"
				);
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
	message: IpcMessage,
) -> anyhow::Result<()> {
	let json = serde_json::to_string(&message)?;

	writer.write_all(json.as_bytes()).await?;
	writer.write_all(b"\n").await?;
	writer.flush().await?;

	Ok(())
}
