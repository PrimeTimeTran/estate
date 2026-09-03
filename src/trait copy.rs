use crate::{e, prelude::*};

pub struct App<C>
where
	C: Context,
{
	// C is a generic type parameter.
	// Caller supplies it.
	// App<WebApp>
	// App<NativeApp>
	context: C,
}

impl<C> App<C>
where
	C: Context,
{
	pub fn host(&self) -> &C::Host {
		self.context.host()
	}

	pub fn runtime(&self) -> &C::Runtime {
		self.context.runtime()
	}

	pub fn services(&self) -> &C::Services {
		self.context.services()
	}
}

struct NativeApp {
	host: NativeHost,
	runtime: NativeRuntime,
	services: NativeServices,
}
impl Context for NativeApp {
	type Host = NativeHost;
	type Runtime = NativeRuntime;
	type Services = NativeServices;

	fn host(&self) -> &Self::Host {
		&self.host
	}

	fn runtime(&self) -> &Self::Runtime {
		&self.runtime
	}

	fn services(&self) -> &Self::Services {
		&self.services
	}
}
struct NativeHost {
	window: NativeWindow,
	storage: NativeStorage,
	clock: NativeClock,
}

struct NativeWindow;
struct NativeStorage;
struct NativeClock;
impl Clock for NativeClock {
	fn now(&self) -> std::time::Instant {
		todo!("now")
	}
}

impl Host for NativeHost {
	type Window = NativeWindow;
	type Storage = NativeStorage;
	type Clock = NativeClock;

	fn window(&self) -> &Self::Window {
		&self.window
	}

	fn storage(&self) -> &Self::Storage {
		&self.storage
	}

	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
}

struct NativeServices {
	persistance: NativePersistance,
	network: NativeNetwork,
	clock: NativeClock,
}

impl Services for NativeServices {
	type Persistence = NativePersistance;
	type Network = NativeNetwork;
	type Clock = NativeClock;

	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn clock(&self) -> &Self::Clock {
		todo!("");
	}
}
struct NativePersistance;

impl Persistence for NativePersistance {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
		todo!("")
	}
	fn save(&self, key: &str, value: &[u8]) -> Result<()> {
		todo!("")
	}
}

struct NativeNetwork;

impl Network for NativeNetwork {
	fn is_available(&self) -> bool {
		todo!("")
	}
}

struct WebApp {
	host: WebHost,
	runtime: WebRuntime,
	services: WebServices,
}

impl Context for WebApp {
	// "For this implementation of Context, Host means WasmHost."
	type Host = WebHost;
	type Runtime = WebRuntime;
	type Services = WebServices;

	fn host(&self) -> &Self::Host {
		&self.host
	}

	fn runtime(&self) -> &Self::Runtime {
		&self.runtime
	}

	fn services(&self) -> &Self::Services {
		&self.services
	}
}

struct WebHost {
	window: BrowserWindow,
	storage: WebStorage,
	clock: WebClock,
}
impl Host for WebHost {
	type Window = BrowserWindow;
	type Storage = WebStorage;
	type Clock = WebClock;
	fn window(&self) -> &Self::Window {
		&self.window
	}

	fn storage(&self) -> &Self::Storage {
		&self.storage
	}

	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
}
struct WebServices;
struct BrowserWindow;
struct WebStorage;
struct WebClock;

impl Services for WebServices {
	type Network = NativeNetwork;
	type Persistence = WebStorage;
	type Clock = WebClock;

	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn clock(&self) -> &Self::Clock {
		todo!("");
	}
}

impl Persistence for WebStorage {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
		todo!("")
	}
	fn save(&self, key: &str, value: &[u8]) -> Result<()> {
		todo!("")
	}
}

impl Clock for WebClock {
	fn now(&self) -> std::time::Instant {
		todo!("now")
	}
}

pub trait Context {
	type Host: Host;
	type Runtime: Runtime;
	type Services: Services;
	fn host(&self) -> &Self::Host;
	fn runtime(&self) -> &Self::Runtime;
	fn services(&self) -> &Self::Services;
}

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
	fn persistence(&self) -> &Self::Persistence;
	fn network(&self) -> &Self::Network;
	fn clock(&self) -> &Self::Clock;
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

// [Potential Renames]
// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
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

	async fn sleep(&self, duration: std::time::Duration);

	fn tasks(&self) -> &Arc<RwLock<TaskManager>>;
	fn state_service(&self) -> &Arc<StateService>;
	fn session_service(&self) -> &Arc<SessionService>;
	// fn spawn<F>(&self, future: F)
	// where
	// 	F: std::future::Future<Output = ()> + Send + 'static;
}
pub trait Spawner: Clone + 'static {
	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static;
}
pub trait Executor: Clone + 'static {
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
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

#[async_trait::async_trait]
pub trait EventHandler<R: Runtime>: 'static {
	async fn handle(&self, event: &e::Event, runtime: &R);
}

pub trait EventReceiver {
	fn try_recv(&mut self) -> Option<e::Event>;
}

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
