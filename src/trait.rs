use crate::{RuntimeState, e, prelude::*};

/// ## Runtime
///
/// Can hold resources that in other environments are thought of as
/// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
///
/// ### Types
/// - [EventReceiver](`crate::r#trait::EventReceiver`).
/// ### Methods
/// - [spawn](`Runtime::spawn`): Create background jobs/tasks/workers
/// - [services](`Runtime::services`): Exposes capabilities
///
///
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
	#[cfg(not(target_arch = "wasm32"))]
	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send;

	#[cfg(target_arch = "wasm32")]
	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()>;

	fn emit(&self, event: e::Event);
	fn event_processed(&self);

	/// ## EventReceiver
	///
	/// Enables clients to subscribe to events
	///
	type EventReceiver: EventReceiver;

	/// ## Subscribe
	///
	/// Mechanism to respond to event system
	///
	fn subscribe(&self) -> Self::EventReceiver;
	fn try_recv(&self) -> Option<e::Event>;
	fn start_dispatcher(self: &Arc<Self>);
	fn state(&self) -> &RuntimeState;
	fn save(&self, state: &EstateState) -> Result<()>;
	fn session(&self) -> Session;

	fn tasks(&self) -> &Arc<RwLock<TaskManager>>;
	fn state_service(&self) -> &Arc<StateService>;
	fn session_service(&self) -> &Arc<SessionService>;

	type Services: Services;

	/// ## Services: own long-lived responsibilities and their concurrency/lifecycle;
	///
	/// Events are the standardized mechanism by which those services expose meaningful
	/// changes to the rest of the application; the Runtime owns the services and EventBus,
	/// while the Dispatcher routes those events to consumers.
	fn services(&self) -> &Self::Services;

	// fn spawn<F>(&self, future: F)
	// where
	// 	F: std::future::Future<Output = ()> + Send + 'static;
	// fn spawn(&self, future: impl Future<Output = ()> + 'static);

	/// ## Spawn: background jobs/tasks/workers that don't block the main thread
	///
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
}

/// ## Context
///
/// from runtime, host, platform
///
/// ### [`Traits`](https://doc.rust-lang.org/rust-by-example/trait.html)
///
/// - [`Runtime`](Context::Runtime): Platform specific runtime
///
/// ### Methods
/// - [`runtime`](Context::runtime) to [`spawn`](Executor::spawn).
///
pub trait Context: Sized {
	/// The host on which the application is running.
	///
	/// An associated type whose concrete implementation is selected by
	/// the [`Context`] implementor.
	type Host: Host;

	/// The runtime environment in which the application is running.
	///
	/// The concrete runtime implementation is selected by the [`Context`]
	/// implementor and can vary based on the platform, host, configuration,
	/// and other runtime factors.
	type Runtime: Runtime;

	/// Returns a reference to the concrete [`Host`] associated with this context.
	///
	/// The returned type is [`Self::Host`], i.e. the associated type selected
	/// by the concrete [`Context`] implementation.
	fn host(&self) -> &Self::Host;

	/// Returns a reference to the concrete [`Runtime`] associated with this context.
	///
	/// The returned type is [`Self::Runtime`], i.e. the associated type selected
	/// by the concrete [`Context`] implementation.
	fn runtime(&self) -> &Self::Runtime;

	type Args;

	fn new() -> Result<Self>;

	fn run(&mut self, args: Self::Args) -> Result<()>;

	fn foo(&self, args: String) -> Result<()>;

	fn bar(&self, args: String) -> Result<()>;
}

pub trait Host {
	/// The concrete environment providing the resources through which the application runs.
	type Window;
	type Storage;
	type Clock;

	// fn platform(&self) -> &Self::Platform;
	fn window(&self) -> &Self::Window;
	fn storage(&self) -> &Self::Storage;
	fn clock(&self) -> &Self::Clock;
}

pub trait Services {
	type Persistence: Persistence;
	type Network: Network;
	type Clock: Clock;
	type Client: Api;

	fn persistence(&self) -> &Self::Persistence;
	fn network(&self) -> &Self::Network;
	fn clock(&self) -> &Self::Clock;

	/// ## Platform Generic API
	///
	/// Exposes capabilities for business logic to access server side resources
	///
	/// - [GRPC]
	///
	/// Has [`Native`] & [`Web`] implementations
	///
	fn api(&self) -> &Self::Client;
}
pub trait Persistence {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
	fn save(&self, key: &str, value: &[u8]) -> Result<()>;
}
pub trait Network {
	fn is_available(&self) -> bool;
}
pub trait Clock {
	fn now(&self) -> std::time::Instant;
}

/// ## Executor
///
/// Enables platform specific APIs for starting background tasks at the generic [app] layer.
///
/// Required abstraction because a native spawn using [Tokio](https://docs.rs/tokio/latest/tokio/)
/// requires [Send](https://doc.rust-lang.org/nomicon/send-and-sync.html) whereas Web/Wasm
/// builds wont compile with the [Tokio] dep.
///
/// This abstraction enables the app to create futures without worrying about how the future
/// is handled from an infrastructure perspective.
///
/// [`Executor::spawn`]
/// The runtime used by [`crate::app::App`].
pub trait Executor: Clone + 'static {
	#[cfg(not(target_arch = "wasm32"))]
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

	#[cfg(target_arch = "wasm32")]
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
}
#[async_trait::async_trait]
pub trait EventHandler<R: Runtime>: Send + Sync + 'static {
	async fn handle(&self, event: &e::Event, runtime: &R);
}

pub trait EventReceiver {
	fn try_recv(&mut self) -> Option<e::Event>;
}

pub trait Spawner: Clone + 'static {
	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static;
}

/// This struct is not [Bar]
pub struct Foo1;

/// This struct is also not [bar](Bar)
pub struct Foo2;

/// This struct is also not [bar][b]
///
/// [b]: Bar
pub struct Foo3;

/// This struct is also not [`Bar`]
pub struct Foo4;

// pub trait SendSpawnRuntime: Runtime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + Send + 'static;
// }
// pub trait SpawnRuntime: Runtime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + 'static;
// }

// pub trait Platform {
// 	type Input: Input;
// 	type Media: Media;

// 	fn input(&self) -> &Self::Input;
// 	fn media(&self) -> &Self::Media;

// 	/// What kind of execution/application environment am I targeting?
// 	fn is_available(&self) -> bool;
// 	fn has_camera(&self) -> bool;
// 	fn has_keyboard(&self) -> bool;
// 	fn has_pointer(&self) -> bool;
// 	fn has_touch(&self) -> bool;
// }
