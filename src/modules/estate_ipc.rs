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
pub struct EventEnvelope {
	pub sequence: u64,
	pub timestamp: u64,
	pub event: Event,
}

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
// pub struct IpcServer {
	// socket: PathBuf,
	// events: EventBus,
	// // eventually:
	// // connections: ...
// }
// 
// impl IpcServer {
	// pub fn new(socket: PathBuf, events: EventBus) -> Self {
		// Self { socket, events }
	// }
// 
	// pub fn start<C>(&self, worker: &HostWorker<C>) -> Result<()>
	// where
		// C: Ctx,
	// {
		// // Unix socket setup
		// // accept loop
		// // connection handling
		// // handshake
		// // event forwarding
		// // commands
		// Ok(())
	// }
// }
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
pub enum IpcMessage {
	Hello(Hello),
	HelloAck(HelloAck),

	Event(EventEnvelope),

	Command(CommandEnvelope),
	Response(ResponseEnvelope),

	Ping { id: u64 },
	Pong { id: u64 },

	Error(IpcError),

	Shutdown,
}

// ─────────────────────────────────────────────────────────────────────────────
// Transport
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait IpcTransport {
	async fn connect(&mut self) -> anyhow::Result<()>;
	async fn send(&mut self, message: &IpcMessage) -> anyhow::Result<()>;
	async fn receive(&mut self) -> anyhow::Result<IpcMessage>;
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
