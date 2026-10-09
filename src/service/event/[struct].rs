use crate::prelude::*;

// Subscribes to local events, transforms into a protobuf
pub struct EventService {
	pub events: EventBus,
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
	pub tx: tokio::sync::broadcast::Sender<e::Event>,
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

pub type IntrinsicEvent = Event;

/// ## [Klass] (Alias of EventKind)
///
/// Represents full event lifecycle for representing initial, pending,
/// failed, repeated when necessary.
///
pub type Klass = EventKind;
pub type Problem = ProtoProblem;
