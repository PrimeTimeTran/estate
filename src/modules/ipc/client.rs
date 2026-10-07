use std::{path::PathBuf, process};
use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
	net::UnixStream,
};

use crate::prelude::*;

impl EstateClient {
	pub fn new() -> Self {
		Self {
			socket: PathBuf::from(ESTATE_SOCKET),
		}
	}
	pub async fn connect(&self) -> anyhow::Result<EstateConnection> {
		let stream = UnixStream::connect(&self.socket).await?;
		let (read_half, mut write_half) = stream.into_split();
		let hello = IpcMessage::<EventKind>::Hello(Hello {
			protocol: ProtocolVersion::CURRENT,
			client: ClientKind::Tauri,
			pid: process::id(),
		});
		let json = serde_json::to_string(&hello)?;
		write_half.write_all(json.as_bytes()).await?;
		write_half.write_all(b"\n").await?;
		write_half.flush().await?;
		let mut reader = BufReader::new(read_half);
		let mut line = String::new();
		reader.read_line(&mut line).await?;
		let message: IpcMessage<EventKind> = serde_json::from_str(&line)?;
		let ack = match message {
			IpcMessage::HelloAck(ack) => ack,
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate IPC error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected Estate IPC response: {other:?}");
			}
		};
		Ok(EstateConnection {
			reader,
			writer: write_half,
			ack,
		})
	}
}

impl EstateConnection {
	pub fn connection_id(&self) -> uuid::Uuid {
		self.ack.connection_id
	}
	async fn send(&mut self, message: IpcMessage<EventKind>) -> anyhow::Result<()> {
		let json = serde_json::to_string(&message)?;
		self.writer.write_all(json.as_bytes()).await?;
		self.writer.write_all(b"\n").await?;
		self.writer.flush().await?;
		Ok(())
	}
	async fn receive(&mut self) -> anyhow::Result<IpcMessage<EventKind>> {
		let mut line = String::new();
		let bytes = self.reader.read_line(&mut line).await?;
		if bytes == 0 {
			anyhow::bail!("Estate IPC connection closed");
		}
		let message = serde_json::from_str::<IpcMessage<EventKind>>(&line)?;
		Ok(message)
	}
	pub async fn ping(&mut self, id: u64) -> anyhow::Result<()> {
		self.send(IpcMessage::Ping { id }).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::Pong { id: response_id } if response_id == id => Ok(()),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate IPC error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected ping response: {other:?}");
			}
		}
	}
	pub async fn context(&mut self) -> anyhow::Result<EstateContext> {
		self.send(IpcMessage::GetContext).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::ContextResult(context) => Ok(context),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate context error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected context response: {other:?}");
			}
		}
	}
}

impl EstateConnection {
	pub async fn curl(&mut self, url: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Curl { url: url.into() })
			.await
	}
	pub async fn env(&mut self, name: Option<String>) -> anyhow::Result<EstateCommandResult> {
		self.run_command(EstateCommand::Env { name }).await
	}
	pub async fn shell_pipeline(
		&mut self,
		commands: Vec<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::ShellPipeline { commands })
			.await
	}
}
impl EstateConnection {
	pub async fn mkdir(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Mkdir { path: path.into() })
			.await
	}
	pub async fn touch(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Touch { path: path.into() })
			.await
	}
	pub async fn write_file(
		&mut self,
		path: impl Into<String>,
		content: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::WriteFile {
				path: path.into(),
				content: content.into(),
			})
			.await
	}
	pub async fn append_file(
		&mut self,
		path: impl Into<String>,
		content: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::AppendFile {
				path: path.into(),
				content: content.into(),
			})
			.await
	}
	pub async fn cat(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Cat { path: path.into() })
			.await
	}
	pub async fn cp(
		&mut self,
		source: impl Into<String>,
		destination: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Cp {
				source: source.into(),
				destination: destination.into(),
			})
			.await
	}
	pub async fn mv(
		&mut self,
		source: impl Into<String>,
		destination: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Mv {
				source: source.into(),
				destination: destination.into(),
			})
			.await
	}
	pub async fn rm(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Rm { path: path.into() })
			.await
	}
	pub async fn ls(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Ls { path: path.into() })
			.await
	}
	pub async fn find(
		&mut self,
		path: impl Into<String>,
		pattern: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Find {
				path: path.into(),
				pattern: pattern.into(),
			})
			.await
	}
	pub async fn rg(
		&mut self,
		pattern: impl Into<String>,
		path: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Rg {
				pattern: pattern.into(),
				path: path.into(),
			})
			.await
	}
	pub async fn head(
		&mut self,
		path: impl Into<String>,
		lines: Option<u64>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Head {
				path: path.into(),
				lines,
			})
			.await
	}
	pub async fn tail(
		&mut self,
		path: impl Into<String>,
		lines: Option<u64>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Tail {
				path: path.into(),
				lines,
			})
			.await
	}
	pub async fn sort(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Sort { path: path.into() })
			.await
	}
	pub async fn wc(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Wc { path: path.into() })
			.await
	}
	pub async fn sed(
		&mut self,
		expression: impl Into<String>,
		path: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Sed {
				expression: expression.into(),
				path: path.into(),
			})
			.await
	}
	pub async fn awk(
		&mut self,
		program: impl Into<String>,
		path: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Awk {
				program: program.into(),
				path: path.into(),
			})
			.await
	}
	pub async fn grep(
		&mut self,
		pattern: impl Into<String>,
		path: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::Grep {
				pattern: pattern.into(),
				path: path.into(),
			})
			.await
	}
}

impl EstateConnection {
	pub async fn git_status(
		&mut self,
		path: impl Into<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::GitStatus { path: path.into() })
			.await
	}
	pub async fn git_diff(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::GitDiff { path: path.into() })
			.await
	}
	pub async fn git_log(&mut self, path: impl Into<String>) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::GitLog { path: path.into() })
			.await
	}
	pub async fn git_show(
		&mut self,
		revision: impl Into<String>,
		path: Option<String>,
	) -> anyhow::Result<EstateCommandResult> {
		self
			.run_command(EstateCommand::GitShow {
				revision: revision.into(),
				path,
			})
			.await
	}
}

impl EstateConnection {
	pub async fn fs_list(&mut self, path: String) -> anyhow::Result<Vec<FileEntry>> {
		self.send(IpcMessage::FsList { path }).await?;
		loop {
			let message = self.receive().await?;
			match message {
				IpcMessage::FsListResult { entries } => {
					return Ok(entries);
				}
				IpcMessage::Event(event) => {
					// Don't return this as the fs_list result.
					// The dedicated event bridge should handle UI events.
					continue;
				}
				IpcMessage::Error(error) => {
					anyhow::bail!("Estate fs_list error {:?}: {}", error.code, error.message);
				}
				other => {
					anyhow::bail!("unexpected fs_list response: {other:?}");
				}
			}
		}
	}
	pub async fn fs_read(&mut self, path: String) -> anyhow::Result<String> {
		self.send(IpcMessage::FsRead { path }).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::FsReadResult { content } => Ok(content),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate fs_read error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected fs_read response: {other:?}");
			}
		}
	}
	pub async fn fs_create(&mut self, path: String, content: String) -> anyhow::Result<()> {
		self.send(IpcMessage::FsCreate { path, content }).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::FsCreateResult => Ok(()),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate fs_create error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected fs_create response: {other:?}");
			}
		}
	}
	pub async fn fs_update(&mut self, path: String, content: String) -> anyhow::Result<()> {
		self.send(IpcMessage::FsUpdate { path, content }).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::FsUpdateResult => Ok(()),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate fs_update error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected fs_update response: {other:?}");
			}
		}
	}
	pub async fn fs_delete(&mut self, path: String) -> anyhow::Result<()> {
		self.send(IpcMessage::FsDelete { path }).await?;
		let message = self.receive().await?;
		match message {
			IpcMessage::FsDeleteResult => Ok(()),
			IpcMessage::Error(error) => {
				anyhow::bail!("Estate fs_delete error {:?}: {}", error.code, error.message);
			}
			other => {
				anyhow::bail!("unexpected fs_delete response: {other:?}");
			}
		}
	}
	pub async fn run_command(
		&mut self,
		command: EstateCommand,
	) -> anyhow::Result<EstateCommandResult> {
		println!("🔥 ESTATE CLIENT → RUN COMMAND: {command:?}");
		self.send(IpcMessage::RunCommand { command }).await?;
		let message = self.receive().await?;

		println!("🔥 ESTATE CLIENT ← COMMAND RESULT: {message:?}");

		match message {
			IpcMessage::CommandResult { result } => Ok(result),

			IpcMessage::Error(error) => {
				anyhow::bail!("Estate command error {:?}: {}", error.code, error.message)
			}

			other => anyhow::bail!("unexpected command response: {other:?}"),
		}
	}
}
pub struct EstateConnection {
	reader: BufReader<tokio::net::unix::OwnedReadHalf>,
	writer: tokio::net::unix::OwnedWriteHalf,
	ack: HelloAck,
}
#[derive(Clone)]
pub struct EstateClient {
	socket: PathBuf,
}
