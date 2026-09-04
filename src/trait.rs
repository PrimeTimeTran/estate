use crate::{RuntimeState, e, prelude::*};

/// [Potential Renames]
/// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
//
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
	#[cfg(not(target_arch = "wasm32"))]
	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send;

	#[cfg(target_arch = "wasm32")]
	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()>;
	// type Api: Api;
	// fn api(&self) -> &Self::Api;

	// Services own long-lived responsibilities and their concurrency/lifecycle;
	//
	// Events are the standardized mechanism by which those services expose meaningful
	// changes to the rest of the application; the Runtime owns the services and EventBus,
	// while the Dispatcher routes those events to consumers.
	type EventReceiver: EventReceiver;
	fn emit(&self, event: e::Event);
	fn event_processed(&self);
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
	fn services(&self) -> &Self::Services;

	// fn spawn<F>(&self, future: F)
	// where
	// 	F: std::future::Future<Output = ()> + Send + 'static;
	// fn spawn(&self, future: impl Future<Output = ()> + 'static);
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
}

/// Context from runtime, host, platform
///
/// Do with that what you will.
pub trait Context: Sized {
	/// The host running the application
	type Host: Host;
	/// Runtime environment in which the app is running.
	/// Multiple factor influence runtime such as plaftorm, host, config & more.
	type Runtime: Runtime;
	/// Provide capabilities to the app
	// type Services: Services;
	// fn services(&self) -> &Self::Services;

	fn host(&self) -> &Self::Host;
	fn runtime(&self) -> &Self::Runtime;

	type Args;
	/// Stuff about generic news
	fn new() -> Result<Self>;

	fn run(&mut self, args: Self::Args) -> Result<()>;
	fn foo(&self, args: String) -> Result<()>;
	fn bar(&self, args: String) -> Result<()>;
}

// struct NativeHost;
pub trait Host {
	/// The concrete environment providing the resources through which the application runs.
	/// type Platform: Platform;
	type Window;
	type Storage;
	type Clock;

	// fn platform(&self) -> &Self::Platform;
	fn window(&self) -> &Self::Window;
	fn storage(&self) -> &Self::Storage;
	fn clock(&self) -> &Self::Clock;
}

pub trait Services {
	/// What capabilities are available?
	type Persistence: Persistence;
	type Network: Network;
	type Clock: Clock;
	type Client: Api;

	fn persistence(&self) -> &Self::Persistence;
	fn network(&self) -> &Self::Network;
	fn clock(&self) -> &Self::Clock;
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
