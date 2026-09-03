pub mod bridge;

use crate::{EventReceiver, app::prelude::*, prelude::*, r#trait};
use async_broadcast::{Receiver, Sender, broadcast};

#[derive(Clone, Debug)]
pub struct WasmRuntime {
	// pub api: WasmApiClient,
	state: Arc<RwLock<EstateState>>,
	events: Sender<e::Event>,
}

impl WasmRuntime {
	pub fn new(state: EstateState) -> Self {
		let (events, _) = broadcast(256);

		Self {
			api: WasmApiClient::new(),
			state: Arc::new(RwLock::new(state)),
			events,
		}
	}
}

impl Runtime for WasmRuntime {
	// type Api = WasmApiClient;
	// fn api(&self) -> &Self::Api {
	// 	&self.api
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
		todo!("WasmRuntime::try_recv is not implemented");
	}
	fn start_dispatcher(self: &Arc<Self>) {
		todo!("WasmRuntime::start_dispatcher is not implemented");
	}
	fn save(&self, state: &EstateState) -> Result<()> {
		todo!("WasmRuntime::start_dispatcher is not implemented");
	}
	fn session(&self) -> Session {
		todo!("WasmRuntime::start_dispatcher is not implemented");
	}
	fn state(&self) -> &RuntimeState {
		todo!("WasmRuntime::state is not implemented");
	}
	async fn sleep(&self, duration: std::time::Duration) {
		gloo_timers::future::sleep(duration).await;
	}
	fn event_processed(&self) {
		let mut state = self.state.write();
		state.events_processed += 1;
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

// #[derive(Clone, Debug)]
// pub struct WasmRuntime {
// 	event_rx: Arc<Mutex<broadcast::Receiver<e::Event>>>,
// 	handle: tokio::runtime::Handle,
// 	proxy: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
// 	pub events: EventBus,
// 	pub session: Session,
// 	pub state: Arc<RuntimeState>,
// 	pub store: NativeStateStore,
// 	pub tasks: Arc<RwLock<TaskManager>>,
// 	pub state_service: Arc<StateService>,
// 	pub session_service: Arc<SessionService>,
// }

// impl WasmRuntime {
// 	pub fn new(handle: tokio::runtime::Handle) -> Result<Self> {
// 		let store = NativeStateStore::new()?;
// 		let session_service = Arc::new(SessionService::new(Arc::new(StateService::new(
// 			crate::STATE_PATH,
// 		))));
// 		let state_service = Arc::new(StateService::new(crate::STATE_PATH));
// 		let state = store.load()?;
// 		let runtime_state = RuntimeState::new(state);
// 		let events = EventBus::new();
// 		let event_rx = Arc::new(Mutex::new(events.subscribe()));
// 		Ok(Self {
// 			event_rx,
// 			events,
// 			handle,
// 			proxy: Arc::new(Mutex::new(None)),
// 			session: Session::default(),
// 			state: Arc::new(runtime_state),
// 			store,
// 			tasks: Arc::new(RwLock::new(TaskManager::new())),
// 			state_service,
// 			session_service,
// 		})
// 	}

// 	pub fn attach_event_proxy(&self, proxy: EventLoopProxy<AppEvent>) {
// 		*self.proxy.lock().unwrap() = Some(proxy);
// 	}

// 	pub fn start_services(&self) {
// 		println!("WasmRuntime start_services");
// 		tracing::info!("WasmRuntime start_services")
// 	}
// }

// impl Runtime for WasmRuntime {
// 	type EventReceiver = NativeEventReceiver;

// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: std::future::Future<Output = ()> + Send + 'static,
// 	{
// 		self.handle.spawn(future);
// 	}

// 	async fn sleep(&self, duration: std::time::Duration) {
// 		tokio::time::sleep(duration).await;
// 	}

// 	fn emit(&self, event: e::Event) {
// 		tracing::debug!("NativeRuntime {:?}", event.kind.clone());
// 		self.events.emit(event);
// 		if let Some(proxy) = self.proxy.lock().unwrap().as_ref() {
// 			let _ = proxy.send_event(AppEvent::RuntimeEvent);
// 		}
// 	}

// 	fn state(&self) -> &RuntimeState {
// 		&self.state
// 	}

// 	fn save(&self, state: &EstateState) -> Result<()> {
// 		self.store.save(state)
// 	}

// 	fn session(&self) -> Session {
// 		self.session.clone()
// 	}

// 	fn subscribe(&self) -> Self::EventReceiver {
// 		NativeEventReceiver {
// 			rx: self.events.subscribe(),
// 		}
// 	}

// 	fn try_recv(&self) -> Option<e::Event> {
// 		self.event_rx.lock().unwrap().try_recv().ok()
// 	}

// 	fn start_dispatcher(self: &Arc<Self>) {
// 		let runtime = Arc::clone(self);
// 		let handle = runtime.handle.clone();
// 		let mut receiver = runtime.events.subscribe();
// 		let mut dispatcher = EventDispatcher::new();

// 		dispatcher.register(crate::event::handler::TaskHandler);
// 		dispatcher.register(crate::event::handler::StateHandler);
// 		dispatcher.register(crate::event::handler::CommandHandler);
// 		dispatcher.register(crate::event::handler::FileWatcherHandler);
// 		dispatcher.register(crate::event::handler::NavigationHandler);
// 		dispatcher.register(crate::event::handler::AppHandler);

// 		handle.spawn(async move {
// 			loop {
// 				match receiver.recv().await {
// 					Ok(event) => {
// 						tracing::debug!("🔥 native::runtime::dispatcher {:?}", event.kind);

// 						dispatcher.dispatch(event, &runtime).await;
// 					}
// 					Err(broadcast::error::RecvError::Lagged(count)) => {
// 						tracing::warn!(count, "native::runtime::start_dispatcher lagged");
// 					}
// 					Err(broadcast::error::RecvError::Closed) => {
// 						tracing::warn!("native::runtime::start_dispatcher closed");
// 						break;
// 					}
// 				}
// 			}
// 		});
// 	}
// }

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
