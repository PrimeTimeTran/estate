use crate::{RuntimeState, prelude::*, r#trait::Context};
use async_broadcast::{Sender, broadcast};

pub struct WebApp {
	host: WebHost,
	runtime: WebRuntime,
	services: WebServices,
}

impl WebApp {
	fn new() -> Result<Self> {
		let state = EstateState::default();
		let runtime = WebRuntime::new(state)?;

		Ok(Self {
			host: WebHost::default(),
			runtime,
			services: WebServices::new()?,
		})
	}
}

impl Context for WebApp {
	// "For this implementation of Context, Host means WasmHost."
	type Host = WebHost;
	type Runtime = WebRuntime;
	// type Services = WebServices;
	type Args = String;

	fn host(&self) -> &Self::Host {
		&self.host
	}

	fn runtime(&self) -> &Self::Runtime {
		&self.runtime
	}

	// fn services(&self) -> &Self::Services {
	// 	&self.services
	// }

	fn new() -> Result<Self> {
		WebApp::new()
	}

	fn run(&mut self, cli: Self::Args) -> Result<()> {
		self.run(cli)
	}
	fn foo(&self, args: String) -> Result<()> {
		WebApp::foo(&self, args)
	}
	fn bar(&self, args: String) -> Result<()> {
		WebApp::bar(&self, args)
	}
}

#[derive(Default)]
pub struct WebHost {
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

#[derive(Debug, Clone)]
pub struct WebServices {
	api: WebApiClient,
}

impl WebServices {
	pub fn new() -> anyhow::Result<Self> {
		let api = WebApiClient::new(String::from(""));
		Ok(Self { api })
	}
}
#[derive(Default)]
pub struct BrowserWindow;
#[derive(Default)]
pub struct WebStorage;
#[derive(Default)]
pub struct WebClock;

impl Services for WebServices {
	type Network = WebNetwork;
	type Persistence = WebStorage;
	type Clock = WebClock;
	type Client = WebApiClient;

	fn api(&self) -> &Self::Client {
		&self.api
	}

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
	fn now(&self) -> Instant {
		todo!("now")
	}
}
#[derive(Default)]
pub struct WebNetwork {}
impl Network for WebNetwork {
	fn is_available(&self) -> bool {
		todo!("")
	}
}

#[derive(Clone, Debug)]
pub struct WebRuntime {
	pub services: WebServices,
	state: Arc<RwLock<EstateState>>,
	events: Sender<e::Event>,
}
impl WebRuntime {
	pub fn new(state: EstateState) -> anyhow::Result<Self> {
		let (events, _) = broadcast::<e::Event>(256);
		let services = WebServices::new()?;
		Ok(Self {
			services,
			state: Arc::new(RwLock::new(state)),
			events,
		})
	}
}

#[derive(Clone, Debug, Default)]
pub struct WebExecutor;

impl Executor for WebExecutor {
	fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		wasm_bindgen_futures::spawn_local(future);
	}
}
impl Runtime for WebRuntime {
	fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		// ...
	}
	async fn sleep(&self, duration: Duration) {
		gloo_timers::future::sleep(duration).await;
	}
	// fn sleep(&self, duration: Duration) -> impl Future<Output = ()> {
	// 	async move {
	// 		gloo_timers::future::sleep(duration).await;
	// 	}
	// }
	type EventReceiver = WasmEventReceiver;
	fn emit(&self, event: e::Event) {
		let _ = self.events.try_broadcast(event);
	}
	fn subscribe(&self) -> Self::EventReceiver {
		WasmEventReceiver {
			receiver: self.events.new_receiver(),
		}
	}
	fn try_recv(&self) -> Option<e::Event> {
		todo!("WebRuntime::try_recv is not implemented");
	}
	fn start_dispatcher(self: &Arc<Self>) {
		todo!("WebRuntime::start_dispatcher is not implemented");
	}
	fn save(&self, state: &EstateState) -> Result<()> {
		todo!("WebRuntime::start_dispatcher is not implemented");
	}
	fn session(&self) -> Session {
		todo!("WebRuntime::start_dispatcher is not implemented");
	}
	fn state(&self) -> &RuntimeState {
		todo!("WebRuntime::state is not implemented");
	}

	fn event_processed(&self) {
		let mut state = self.state.write();
		// state.events_processed += 1;
	}
	fn tasks(&self) -> &Arc<RwLock<TaskManager>> {
		todo!("tasks")
	}

	fn state_service(&self) -> &Arc<StateService> {
		todo!("tasks")
	}
	fn session_service(&self) -> &Arc<SessionService> {
		todo!("tasks")
	}

	type Services = WebServices;

	fn services(&self) -> &Self::Services {
		&self.services
	}
}

impl EventReceiver for async_broadcast::Receiver<e::Event> {
	fn try_recv(&mut self) -> Option<e::Event> {
		self.try_recv().ok()
	}
}
pub struct WasmEventReceiver {
	receiver: async_broadcast::Receiver<e::Event>,
}

impl EventReceiver for WasmEventReceiver {
	fn try_recv(&mut self) -> Option<e::Event> {
		self.receiver.try_recv().ok()
	}
}

// use crate::EventReceiver;
#[derive(Clone)]
pub struct WasmExecutor;
impl WasmExecutor {
	pub fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static,
	{
		wasm_bindgen_futures::spawn_local(future);
	}
}
