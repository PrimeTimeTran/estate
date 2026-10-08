//! ## [Event System]
//!
//! There's a lot of moving parts to any meaningful event system. These are the most important ones that come to mind
//! when I think of what we a developer must be comfortable with to truly be able to jump in and make a impact on the code base.
//!
//! - async
//! - background
//! - blocking
//! - broadcast
//! - channel
//! - dispatch
//! - dispatcher
//! - emit
//! - handle
//! - handler
//! - process
//! - receiver
//! - schedule
//! - send
//! - sender
//! - subscribe
//! - sync
//! - thread
//!
use uuid::Timestamp;

use crate::{
	prelude::{SdlcEvent, *},
	proto::types as proto_types,
};

#[path = "[struct].rs"]
pub mod event_structs;
#[path = "[impl].rs"]
pub mod impls;

pub use event_structs::*;
pub use impls::*;

/// # Create Events
///
/// A namespace for creating events
///
/// ### Example
///
/// All of these would work. They're made difference because in some places app may or may not be available.
///
/// ```ignore
/// - e::create::app(e::EventKind::SessionStart)
/// - create::app(e::EventKind::SessionStart)
/// ```
///
/// Namespaces are used to make emitting of events easier
/// in downstream code.
///
pub mod create {
	use super::IntrinsicEvent as Event;
	use crate::prelude::*;

	pub fn daemon(kind: EventKind) -> Event {
		Event::daemon(kind)
	}
	pub fn cli(kind: EventKind) -> Event {
		Event::cli(kind)
	}
	pub fn fs(kind: EventKind) -> Event {
		Event::filesystem(kind)
	}
	pub fn editor(kind: EventKind) -> Event {
		Event::editor(kind)
	}
	pub fn app(kind: EventKind) -> Event {
		Event::app(kind)
	}
	pub fn sdlc(event: SdlcEvent) -> Event {
		Event::sdlc(EventKind::Sdlc(event))
	}
	pub fn ipc(event: Event) -> EventEnvelope<EventKind> {
		EventEnvelope {
			id: EventId {
				node: NodeId,
				sequence: event.id,
			},
			timestamp: event.timestamp,
			source: event.source,
			event: event.kind,
		}
	}
}

/// ## [AppEvent]
///
/// System events used to control winit/daemon lifecycle.
///
#[derive(Debug)]
pub enum AppEvent {
	AppEvent,
	Navigate(crate::ui::ViewType),
	RuntimeEvent,
	Shutdown,
	TickClock(String),
	CursorPosition {
		x: f64,
		y: f64,
	},
	ModifiersChanged {
		alt: bool,
		command: bool,
		ctrl: bool,
		shift: bool,
	},
}

/// ## [EventKind]
///
/// Represent event lifecycle of events.
/// Not every event is guaranteed to succeed so this abstraction enables to
/// model the lifecycle of events from creation until completion and everything in between
/// and subsequent
///
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EventKind {
	ApiError(String),
	CacheInvalidated {
		reason: String,
	},
	CommandExecuted {
		command: String,
	},
	DaemonStarted,
	DaemonStopped,
	EstateDiscovered {
		inode: Inode,
		path: String,
	},
	EstateRemoved {
		inode: Inode,
		path: String,
	},
	FileCreated {
		inode: Inode,
		path: String,
	},
	FileDeleted {
		inode: Inode,
		path: String,
	},
	FileModified {
		inode: Inode,
		path: String,
	},
	IndexUpdated {
		files_changed: u64,
	},
	Navigate(ViewType),
	ProblemLoaded(StoredProblem),
	ProblemLoadFailed(String),
	ProblemSampled(StoredProblem),
	ProblemSampleFailed(String),
	ProblemsRequested,
	ProblemsLoaded(Vec<StoredProblem>),
	ProblemsLoadFailed(String),
	SampleProblemsError(String),
	SampleProblemsLoaded(Vec<StoredProblem>),
	SampleProblemsLoading,
	SessionStart,
	SessionStop {
		session: Session,
	},
	StatusRequested,
	TaskCompleted {
		task_id: TaskId,
	},
	TaskCreated {
		task_id: Uuid,
		kind: TaskKind,
	},
	TaskDeleted {
		task_id: TaskId,
	},
	TaskFailed {
		task_id: TaskId,
		error: String,
	},
	TaskRequested {
		request: TaskRequest,
	},
	TasksCleared,
	TaskStarted {
		task_id: TaskId,
	},
	TaskStopped {
		task_id: TaskId,
	},
	WorkspaceIndexed {
		duration: u64,
	},
	KeyDown {
		key_code: u16,
	},
	KeyUp {
		key_code: u16,
	},
	MouseDown {
		button: i64,
		x: f64,
		y: f64,
	},
	MouseUp {
		button: i64,
		x: f64,
		y: f64,
	},
	Scroll {
		vertical: i64,
		horizontal: i64,
	},
	FlagsChanged {
		key_code: u16,
	},
	ModifierChanged {
		modifiers: ModifierSnapshot,
	},
	Sdlc(SdlcEvent),
	Agent(AgentEvent),
	ActiveAppChanged {
		name: String,
		bundle_id: String,
		pid: u32,
	},
	Unknown,
}

/// ## [EventSource]
///
/// Useful for creating more detailed logs & traces in the future.
///
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
	Sdlc,
}

pub enum EventScope {
	Local,
	Process,
	Node,
	Cluster,
}

pub trait EventPublisher<E> {
	fn publish(&self, event: E) -> Result<()>;
}

/// Transport used by [`EventClient`] to subscribe to remote events.
///
/// The transport owns the actual networking implementation.
///
/// Native might use tonic.
///
/// WASM might use gRPC-Web, WebSocket, or another browser-compatible
/// transport.
///
/// The service layer only cares that the transport can produce protobuf
/// events.
pub trait EventTransport {
	type Error;

	fn subscribe(
		&mut self,
		events: EventBus,
	) -> impl std::future::Future<Output = Result<(), Self::Error>>;
}

impl<T> EventClient<T> {
	pub fn new(events: EventBus, transport: T) -> Self {
		Self { events, transport }
	}
}

impl<T> EventClient<T>
where
	T: EventTransport,
{
	/// Subscribe to remote events and publish them into the local EventBus.
	pub async fn subscribe(&mut self) -> Result<(), T::Error> {
		self.transport.subscribe(self.events.clone()).await
	}
}

/// Publish a protobuf event into the local application EventBus.
///
/// This is the common final step for every transport.
pub fn emit_proto_event(events: &EventBus, event: proto_types::Event) -> Result<()> {
	let event = event_from_proto(event)?;
	events.emit(event);
	Ok(())
}

/// Convert a wire/protobuf event into an application event.
///
/// This is intentionally kept separate from the transport.
///
/// Both native and WASM transports ultimately produce the same
/// `proto_types::Event`, so the conversion only needs to exist once.
pub fn event_from_proto(event: proto_types::Event) -> Result<Event> {
	Ok(Event {
		id: event.id,
		kind: serde_json::from_str(&event.payload)?,
		source: match event.source.as_str() {
			"App" => EventSource::App,
			"Cli" => EventSource::Cli,
			"Daemon" => EventSource::Daemon,
			"Editor" => EventSource::Editor,
			"Filesystem" => EventSource::Filesystem,
			source => {
				anyhow::bail!("unknown event source: {source}");
			}
		},
		timestamp: event.timestamp,
	})
}

impl Event {
	pub fn app(kind: EventKind) -> Self {
		Self::new(EventSource::App, kind)
	}
	fn new(source: EventSource, kind: EventKind) -> Self {
		tracing::debug!("new Event {:?}", source);
		Self {
			id: EVENT_ID.fetch_add(1, Ordering::Relaxed),
			kind,
			source,
			timestamp: crate::util::now(),
		}
	}
	pub fn daemon(kind: EventKind) -> Self {
		Self::new(EventSource::Daemon, kind)
	}
	pub fn cli(kind: EventKind) -> Self {
		Self::new(EventSource::Cli, kind)
	}
	pub fn filesystem(kind: EventKind) -> Self {
		Self::new(EventSource::Filesystem, kind)
	}
	pub fn editor(kind: EventKind) -> Self {
		Self::new(EventSource::Editor, kind)
	}
	pub fn sdlc(kind: EventKind) -> Self {
		Self::new(EventSource::Sdlc, kind)
	}
}
impl EventBus {
	// #[cfg(target_arch = "wasm32")]
	pub fn emit(&self, event: e::Event) {
		match self.tx.send(event.clone()) {
			Ok(count) => {
				// tracing::info!("📡 Event emitted: {:?} → {} receiver(s)", event.kind, count);
			}
			Err(_) => {
				tracing::info!("⚠️ Event emitted with NO receivers: {:?}", event.kind);
			}
		}
	}
}
impl std::hash::Hash for EventBus {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.tx.same_channel(&self.tx).hash(state);
	}
}
impl EventSink<AppEvent> for EventLoopProxy<AppEvent> {
	fn send(&self, event: AppEvent) {
		let _ = self.send_event(event);
	}
}
impl From<ProtoProblem> for ProblemLoaded {
	fn from(problem: ProtoProblem) -> Self {
		Self {
			id: problem.id,
			title: problem.title,
			slug: problem.slug,
		}
	}
}

/// ## [Event]
///
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
	pub id: u64,
	pub kind: EventKind,
	pub source: EventSource,
	pub timestamp: u64,
}

/// ## [EventBus]
///
/// Enables disparate modules to talk to each other by sending and receiving
/// messages called events.
///
/// <details>
/// <summary>Diagram</summary>
///
/// ```mermaid
/// flowchart TD
///     EB["EventBus"]
///     EB --> RX["subscribe() → BroadcastReceiver"]
///     EB --> TX["sender() → BroadcastSender"]
///
///     RX --> R["Renderer"]
///     TX --> R
///
///     R --> AC["AppContext"]
///     AC --> PV["ProblemView::draw()"]
///
///     PV -->|send| PR["ProblemsRequested"]
///     PR --> EB
///
///     EB --> RT["AppRuntime"]
///     RT --> API["async API request"]
///
///     API -->|success| PL["ProblemsLoaded"]
///     API -->|failure| PF["ProblemsLoadFailed"]
///
///     PL --> EB
///     PF --> EB
///
///     RT --> S["Update application state"]
/// ```
/// </details>
///
/// The issue is it's not big enough. Doesn't scroll?
#[derive(Debug, Clone)]
pub struct EventBus {
	pub tx: broadcast::Sender<e::Event>,
	// id: usize,
}

/// Client-side event service.
///
/// `T` is the transport used to receive events.
///
/// The transport is intentionally generic so this type can exist on both
/// native and WASM without making the service itself depend on tonic.
pub struct EventClient<T> {
	pub events: EventBus,
	pub transport: T,
}

/// ## Events Roadmap
/// - 3 Even Paradigms/Types
/// 	- In process events (notification module tells ui module an event occured with dispatch)
/// 	- In network (K8s Cluster running inside of a VPN has multiple nodes that talk to each other)
/// 	- Over the wire (Server wants users to know someone 'signed in')
///
/// The following structs will cover those cases and enable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
	pub id: EventId,
	pub timestamp: u64,
	pub source: EventSource,
	pub event: E,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventId {
	pub node: NodeId,
	pub sequence: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId;
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct ProblemLoaded {
	pub id: String,
	pub title: String,
	pub slug: String,
}

type IntrinsicEvent = Event;

/// ## [Klass] (Alias of EventKind)
///
/// Represents full event lifecycle for representing initial, pending,
/// failed, repeated when necessary.
///
pub type Klass = EventKind;
pub type Problem = ProtoProblem;
