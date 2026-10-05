use crate::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Protocol
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProtocolVersion {
	pub major: u16,
	pub minor: u16,
}

impl ProtocolVersion {
	pub const CURRENT: Self = Self { major: 1, minor: 0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientKind {
	Tauri,
	Cli,
	Daemon,
	NativeObserver,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handshake
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
	pub protocol: ProtocolVersion,
	pub client: ClientKind,
	pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloAck {
	pub protocol: ProtocolVersion,
	pub server: ClientKind,
	pub connection_id: Uuid,
}

// ─────────────────────────────────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcEvent {
	Sdlc(SdlcEvent),
	Native(NativeEvent),
	// Context(ContextEvent),
	// Workspace(WorkspaceEvent),
}

// ─────────────────────────────────────────────────────────────────────────────
// Commands
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
	GetStatus,
	GetContext,
	OpenContextPanel,
	ExecuteAction { action: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
	pub id: Uuid,
	pub command: IpcCommand,
}

// ─────────────────────────────────────────────────────────────────────────────
// Responses
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
	pub id: Uuid,
	pub result: CommandResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
	Ok,
	Error(IpcError),
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
	pub code: IpcErrorCode,
	pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IpcErrorCode {
	ProtocolMismatch,
	InvalidMessage,
	NotReady,
	NotFound,
	Unsupported,
	Internal,
}

// ─────────────────────────────────────────────────────────────────────────────
// Connection
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
	pub id: Uuid,
	pub client: ClientKind,
	pub pid: u32,
	pub protocol: ProtocolVersion,
	pub connected_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// IPC Message
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage<E> {
	// Connection
	Hello(Hello),
	HelloAck(HelloAck),

	// Server → client events
	Event(EventEnvelope<E>),

	// Commands
	Command(CommandEnvelope),
	Response(ResponseEnvelope),

	// Connection control
	Ping { id: u64 },
	Pong { id: u64 },

	Error(IpcError),

	Shutdown,

	// Context
	GetContext,
	ContextResult(EstateContext),

	// Filesystem
	FsList { path: String },

	FsListResult { entries: Vec<FileEntry> },

	FsRead { path: String },

	FsReadResult { content: String },

	FsCreate { path: String, content: String },

	FsCreateResult,

	FsUpdate { path: String, content: String },

	FsUpdateResult,

	FsDelete { path: String },

	FsDeleteResult,

	// Command execution
	RunCommand { command: EstateCommand },

	CommandResult { result: EstateCommandResult },
}
pub type EstateIpcMessage = IpcMessage<EventKind>;
// ─────────────────────────────────────────────────────────────────────────────
// Transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait IpcTransport<E>
where
	E: Serialize + for<'de> Deserialize<'de> + Send,
{
	async fn connect(&mut self) -> anyhow::Result<()>;
	async fn send(
		&mut self,
		message: &IpcMessage<E>,
	) -> anyhow::Result<()>;
	async fn receive(
		&mut self,
	) -> anyhow::Result<IpcMessage<E>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Estate paths
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EstatePaths {
	pub data_dir: PathBuf,
	pub log_dir: PathBuf,
	pub socket_dir: PathBuf,
	pub state_file: PathBuf,
	pub settings_file: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriContext {
	pub pid: u32,
	pub platform: String,
	pub arch: String,
	pub cwd: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EstateContext {
	pub connection_id: Uuid,
	pub active_app: String,
	pub workspace: Option<String>,
	pub project: Option<String>,
	pub mode: String,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
	pub path: String,
	pub kind: String,
	pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstateCommand {
	Mkdir {
		path: String,
	},

	Touch {
		path: String,
	},

	WriteFile {
		path: String,
		content: String,
	},

	AppendFile {
		path: String,
		content: String,
	},

	Cat {
		path: String,
	},

	Cp {
		source: String,
		destination: String,
	},

	Mv {
		source: String,
		destination: String,
	},

	Rm {
		path: String,
	},

	Ls {
		path: String,
	},

	Find {
		path: String,
		pattern: String,
	},

	Rg {
		pattern: String,
		path: String,
	},

	Head {
		path: String,
		lines: Option<u64>,
	},

	Tail {
		path: String,
		lines: Option<u64>,
	},

	Sort {
		path: String,
	},

	Wc {
		path: String,
	},

	Sed {
		expression: String,
		path: String,
	},

	Awk {
		program: String,
		path: String,
	},

	Grep {
		pattern: String,
		path: String,
	},

	GitStatus {
		path: String,
	},

	GitDiff {
		path: String,
	},

	GitLog {
		path: String,
	},

	GitShow {
		revision: String,
		path: Option<String>,
	},

	Curl {
		url: String,
	},

	Env {
		name: Option<String>,
	},

	ShellPipeline {
		commands: Vec<String>,
	},
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstateCommandResult {
	pub command: EstateCommand,
	pub success: bool,
	pub exit_code: Option<i32>,
	pub stdout: String,
	pub stderr: String,
}
