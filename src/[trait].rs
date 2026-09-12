//! ## [Traits]
//!
//! The collection of traits used through the codebase
//!
use crate::{RuntimeState, e, prelude::*};

// https://github.com/rust-lang/rust/issues/41517
// https://github.com/rust-lang/rust/issues/55628
// https://github.com/rust-lang/rfcs/pull/1733

/// ## [Context]
///
/// from runtime, host, platform
///
/// ### [Traits][traits]
///
/// - [Runtime](Context::Runtime): Platform specific runtime
///
/// ### Methods
///
/// - [runtime](Context::runtime) to [`spawn`](Executor::spawn).
///
///
/// [traits]: https://doc.rust-lang.org/rust-by-example/trait.html
pub trait Context: Sized {
	/// The host on which the application is running.
	///
	/// An associated type whose concrete implementation is selected by
	/// the [Context] implementor.
	// type Host: Host;
	/// Returns a reference to the concrete [Host] associated with this context.
	///
	/// The returned type is [Self::Host], i.e. the associated type selected
	/// by the concrete [Context] implementation.
	// fn host(&self) -> &Self::Host;
	/// The runtime environment in which the application is running.
	///
	/// The concrete runtime implementation is selected by the [Context]
	/// implementor and can vary based on the platform, host, configuration,
	/// and other runtime factors.
	type Runtime: Runtime;
	/// ## [Context::runtime]
	///
	/// Returns a reference to the concrete [`Runtime`] associated with this context.
	///
	/// The returned type is [`Self::Runtime`], i.e. the associated type selected
	/// by the concrete [`Context`] implementation.
	fn runtime(&self) -> &Self::Runtime;

	type Args;

	fn new() -> Result<Self>;

	fn run(&mut self, args: Self::Args) -> Result<()>;

	// fn foo(&self, args: String) -> Result<()>;

	// fn bar(&self, args: String) -> Result<()>;
}

// pub trait State {}

pub trait ApiServices: Services {
	type Client: Api;
	/// ## Platform Generic API
	///
	/// Exposes capabilities for business logic to access server side resources
	///
	/// - [GRPC]
	///
	/// Has [`Native`] & [`Web`] implementations
	fn foo(&self);
	// fn api(&self) -> &Self::Client;
	// fn api(&self) -> Option<&Self::Client>;
}

/// ## [Ctx]
///
/// A type safe abstraction with room for growth via it's internal [associated types].
///
/// ### [Types][associated types]
///
/// - [State](Self::State).
///
/// [associated types]: https://doc.rust-lang.org/rust-by-example/generics/assoc_items/types.html
pub trait Ctx: Default {
	type State: Clone + Send + Sync + 'static;
	fn state(&self) -> &Self::State;
}

/// ## [Clock]
///
/// The literal heart beat of the engine. The clock has a handle on
/// the [runtime](Runtime) of [tokio] which enables spawning of workers
///
pub trait Clock: Clone {
	type Handle<C: Ctx, J>;

	// #[cfg(not(target_arch = "wasm32"))]
	// type WorkHandle<C: Ctx, J>;

	// #[cfg(target_arch = "wasm32")]
	// type WorkHandle<C: Ctx>;

	fn now(&self) -> String;

	/// Run once and return.
	fn run_once(&self) -> String;

	/// Run repeatedly in the foreground.
	#[cfg(not(target_arch = "wasm32"))]
	fn run_foreground(&self, interval: Duration);

	fn run_background<C, J>(&self, interval: Duration, msg: String) -> Self::Handle<C, J>
	where
		C: Ctx,
		J: From<tokio::task::JoinHandle<()>>;
	// #[cfg(not(target_arch = "wasm32"))]
	// fn run_background<C>(&self, interval: Duration, msg: String) -> Self::WorkHandle<C>
	// where
	// 	C: Ctx;
	// 	// J: From<tokio::task::JoinHandle<()>>;

	// #[cfg(target_arch = "wasm32")]
	// fn run_background<C: Ctx>(&self, interval: Duration, msg: String) -> Self::WorkHandle<C>;
}

/// ## [CursorEventSink]
///
/// A drain of all events from
///
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub trait CursorEventSink: Send + Sync + 'static {
	fn cursor_moved(&self, position: CursorPosition);
	fn modifiers_changed(&self, modifiers: Modifiers);
}

/// ## [Engine]
///
pub trait Engine {
	// IDE anchors/bookmarks
	fn upsert() -> Result<(), Error>;
	fn read() -> Result<(), Error>;
	fn delete() -> Result<(), Error>;
	// .estate workspace (initial personal and then public/repo)
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;

	// CRUD .estate registry for IDE/discovery services
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;

	// CRUD .estate index for IDE/discovery services
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;
}

/// ## [EventHandler]
///
/// Broken Link. Why? Others using same structure work
///
#[async_trait::async_trait]
pub trait EventHandler<R: Runtime>: Send + Sync + 'static {
	async fn handle(&self, event: &e::Event, runtime: &R);
}

/// ## [EventReceiver]
///
pub trait EventReceiver {
	fn try_recv(&mut self) -> Option<e::Event>;
}

/// ## [Executor]
///
/// Enables platform specific APIs for starting background tasks at the generic [app] layer.
///
/// Required abstraction because a native spawn using [tokio]
/// requires [Send] whereas Web/Wasm
/// builds wont compile with the [tokio] dep.
///
/// This abstraction enables the app to create futures without worrying about how the future
/// is handled from an infrastructure perspective.
///
/// [Executor::spawn]
/// The runtime used by [crate::app::App].
///
pub trait Executor: Clone + 'static {
	#[cfg(not(target_arch = "wasm32"))]
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

	#[cfg(target_arch = "wasm32")]
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
}

pub trait EventSink<E>: Send + Sync + 'static {
	fn send(&self, event: E);
}

/// ## [Index]
///
/// derived structure optimized for finding that knowledge.
pub trait Index {
	fn generation(&self) -> u64;
	fn lookup(&self, query: &Query) -> Vec<Uuid>;
	fn invalidate(&mut self, change: &Change);
}

/// ## [Input]
///
pub trait Input {}

/// ## [Media]
///
pub trait Media {}

/// ## [Network]
///
pub trait Network {
	fn is_available(&self) -> bool;
}

/// ## [Platform]
///
pub trait Platform {
	type Input: Input;
	type Media: Media;

	fn input(&self) -> &Self::Input;
	fn media(&self) -> &Self::Media;

	/// What kind of execution/application environment am I targeting?
	fn is_available(&self) -> bool;
	fn has_camera(&self) -> bool;
	fn has_keyboard(&self) -> bool;
	fn has_pointer(&self) -> bool;
	fn has_touch(&self) -> bool;
}

/// ## [Renders]
///
pub trait Renders {
	fn render(&mut self);
}

/// ## [Provide]
///
/// Ensures we have platform agnostic APIs to  enables behavior.
pub trait Provide<C: Ctx> {
	type Clock: Clock;
	type Worker;
	// type Renderer: Renderer;
	fn clock(&self) -> &Self::Clock;
	fn worker(&self) -> &Self::Worker;
	// fn renderer(&mut self) -> &mut Self::Renderer;
}

/// ## [Persistence]
pub trait Persistence {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
	fn save(&self, key: &str, value: &[u8]) -> Result<()>;
}

/// ## [Registry]
///
pub trait Registry {
	fn get(&self, id: Uuid) -> Option<Resource>;
	fn upsert(&mut self, resource: Resource);
	fn remove(&mut self, id: Uuid);
}

/// ## [Runtime]
///
/// Can hold resources that other environments think of as
/// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
///
/// ### [Trait](https://doc.rust-lang.org/rust-by-example/trait.html)
///
/// - [EventReceiver]
///
/// ### Methods
///
/// - [spawn](Runtime::spawn): Spawn background threads
/// - [services](Runtime::services): Exposes capabilities
/// - [subscribe](Runtime::subscribe): Subscribe to event broadcasts using a [event bus](crate::server::events::EventBus)
///
/// ## Note
///
/// A type implementing Runtime cannot contain non-'static borrowed references.
///
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
	#[cfg(not(target_arch = "wasm32"))]
	fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;

	/// [RPIT](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)

	#[cfg(target_arch = "wasm32")]
	fn sleep(&self, duration: Duration) -> impl Future<Output = ()>;

	fn session(&self) -> Session;
	fn emit(&self, event: e::Event);
	fn event_processed(&self);

	/// ## [EventReceiver]
	///
	/// Enables clients to [subscribe](Self::subscribe) to events
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

	fn tasks(&self) -> &Arc<RwLock<TaskManager>>;
	fn state_service(&self) -> &Arc<StateService>;
	fn session_service(&self) -> &Arc<SessionService>;

	type Services: Services;

	/// ## [Services]
	///
	/// Own long-lived responsibilities and their concurrency/lifecycle;
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

/// ## [Resolver]
///
/// 	"What does C mean?"
///
/// derived structure optimized for finding that knowledge.
///
/// - Inline IDE Anchor -> FS file for preview
/// - Inline wikilink -> FS asset for embed
///
pub trait Resolver {
	fn resolve(&self, reference: &Reference, context: &ResolveContext) -> Vec<Resolution>;
}

/// ## [Services]
pub trait Services {
	type Persistence: Persistence;
	type Network: Network;
	type Client: Api;

	fn persistence(&self) -> &Self::Persistence;
	fn network(&self) -> &Self::Network;

	/// ## Platform Generic API
	///
	/// Exposes capabilities for business logic to access server side resources.
	///
	/// Has [`Native`] & [`Web`] implementations.
	fn api(&self) -> &Option<Self::Client>;
}

/// ## [Spawner]
///
pub trait Spawner: Clone + 'static {
	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static;
}

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

/// ## [Worker]
///
pub trait Worker<C: Ctx> {
	type Handle;

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static;

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static;

	#[cfg(not(target_arch = "wasm32"))]
	fn run_background_blocking<F>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) + Send + 'static;

	#[cfg(not(target_arch = "wasm32"))]
	fn spawn<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce() -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static;

	// #[cfg(target_arch = "wasm32")]
	// fn interval<F, Fut>(&self, duration: Duration, task: F) -> Self::Handle<()>
	// where
	// 	F: Fn(CancellationToken) -> Fut + 'static,
	// 	Fut: Future<Output = ()> + 'static;
}

pub trait CtxWeb {}
pub trait CtxNative {}
pub trait NativeContext {
	fn handle(&self) -> tokio::runtime::Handle;
	fn shutdown(self);
	fn wait_for_shutdown(&self);
}

pub trait StateStore: Send + Sync {
	fn load(&self) -> Result<EstateState>;
	fn save(&self, state: &EstateState) -> Result<()>;
}
