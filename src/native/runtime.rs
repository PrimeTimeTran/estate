pub use crate::native::{monitor::NativeMonitor, prelude::*, state::NativeStateStore};
use crate::{
	EventReceiver,
	app::{
		Runtime,
		state::{EstateState, StateStore},
	},
	e,
};
use std::sync::Mutex;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	event_rx: Arc<Mutex<broadcast::Receiver<e::Event>>>,
	pub handle: tokio::runtime::Handle,
	proxy: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
	pub events: EventBus,
	pub session: Session,
	pub state: Arc<RuntimeState>,
	pub store: NativeStateStore,
	pub tasks: Arc<RwLock<TaskManager>>,
	pub state_service: Arc<StateService>,
	// pub api: Box<dyn Api>,
	pub session_service: Arc<SessionService>,
	pub executor: NativeExecutor,
	services: NativeServices,
}

impl NativeRuntime {
	pub async fn new(handle: tokio::runtime::Handle) -> anyhow::Result<Self> {
		let store = NativeStateStore::new()?;
		let state_service = Arc::new(StateService::new(crate::STATE_PATH));
		let session_service = Arc::new(SessionService::new(Arc::clone(&state_service)));
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);
		let events = EventBus::new();
		let event_rx = Arc::new(Mutex::new(events.subscribe()));
		let services = NativeServices::connect().await?;

		let executor = NativeExecutor {
			handle: handle.clone(),
		};
		Ok(Self {
			event_rx,
			events,
			handle,
			services,
			// api: Box::new(NativeApiClient::new()),
			proxy: Arc::new(Mutex::new(None)),
			session: Session::default(),
			state: Arc::new(runtime_state),
			store,
			tasks: Arc::new(RwLock::new(TaskManager::new())),
			state_service,
			session_service,
			executor,
		})
	}

	pub fn attach_event_proxy(&self, proxy: EventLoopProxy<AppEvent>) {
		*self.proxy.lock().unwrap() = Some(proxy);
	}

	pub fn event_processed(&self) {
		let mut state = self.state.write();
		state.events_processed += 1;
	}

	pub fn start_services(&self) {
		println!("NativeRuntime start_services");
		tracing::info!("NativeRuntime start_services")
	}
}

#[derive(Clone, Debug)]
pub struct NativeExecutor {
	pub handle: tokio::runtime::Handle,
}
impl NativeExecutor {
	pub fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + Send + 'static,
	{
		println!("🔥 NativeRuntime::spawn CALLED");
		tokio::spawn(async move {
			println!("🔥 NativeRuntime task STARTED");
			future.await;
		});
	}
}
impl Executor for NativeExecutor {
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
		self.handle.spawn(future);
	}
}
impl Executor for NativeRuntime {
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
		tokio::spawn(future);
	}
}
impl Runtime for NativeRuntime {
	fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		// ...
	}

	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send {
		tokio::time::sleep(duration)
	}

	type EventReceiver = NativeEventReceiver;

	fn emit(&self, event: e::Event) {
		tracing::debug!("NativeRuntime {:?}", event.kind.clone());
		self.events.emit(event);
		if let Some(proxy) = self.proxy.lock().unwrap().as_ref() {
			let _ = proxy.send_event(AppEvent::RuntimeEvent);
		}
	}

	fn state(&self) -> &RuntimeState {
		&self.state
	}

	fn save(&self, state: &EstateState) -> Result<()> {
		self.store.save(state)
	}

	fn session(&self) -> Session {
		self.session.clone()
	}

	fn subscribe(&self) -> Self::EventReceiver {
		NativeEventReceiver {
			rx: self.events.subscribe(),
		}
	}

	fn try_recv(&self) -> Option<e::Event> {
		self.event_rx.lock().unwrap().try_recv().ok()
	}
	fn event_processed(&self) {
		println!("event processed")
	}
	fn tasks(&self) -> &Arc<RwLock<TaskManager>> {
		&self.tasks
	}

	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let handle = runtime.handle.clone();
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::<NativeRuntime>::new();
		dispatcher.register(TaskHandler);
		dispatcher.register(StateHandler);
		dispatcher.register(CommandHandler);
		dispatcher.register(FileWatcherHandler);
		dispatcher.register(NavigationHandler);
		dispatcher.register(AppHandler);
		handle.spawn(async move {
			loop {
				match receiver.recv().await {
					Ok(event) => {
						tracing::debug!("🔥 native::runtime::dispatcher {:?}", event.kind);
						dispatcher.dispatch(event, &runtime).await;
					}
					Err(broadcast::error::RecvError::Lagged(count)) => {
						tracing::warn!(count, "native::runtime::start_dispatcher lagged");
					}
					Err(broadcast::error::RecvError::Closed) => {
						tracing::warn!("native::runtime::start_dispatcher closed");
						break;
					}
				}
			}
		});
	}
	fn state_service(&self) -> &Arc<StateService> {
		&self.state_service
	}
	fn session_service(&self) -> &Arc<SessionService> {
		&self.session_service
	}

	type Services = NativeServices;
	fn services(&self) -> &Self::Services {
		&self.services
	}
}

pub struct NativeAppContext<'a> {
	pub base: AppContext<'a, NativeRuntime, NativeExecutor>,
	pub monitor: &'a mut NativeMonitor,
}
impl<'a> NativeAppContext<'a> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.base.state()
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn poll_state(&mut self) -> bool {
		todo!("Unused for not.")
	}
}

// impl SendSpawnRuntime for NativeRuntime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + Send + 'static,
// 	{
// 		self.handle.spawn(future);
// 	}
// }

// impl SpawnRuntime for NativeRuntime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + std::marker::Send + 'static,
// 	{
// 		self.handle.spawn(future);
// 	}
// }

// impl EventReceiver for async_broadcast::Receiver<e::Event> {
// 	fn try_recv(&mut self) -> Option<e::Event> {
// 		self.try_recv().ok()
// 	}
// }

// #[derive(Clone)]
pub struct NativeEventReceiver {
	pub rx: broadcast::Receiver<e::Event>,
}

impl EventReceiver for NativeEventReceiver {
	fn try_recv(&mut self) -> Option<e::Event> {
		self.rx.try_recv().ok()
	}
}

// impl AppHost<NativeRuntime> for NativeApp {
// 	fn app(&mut self) -> &mut App<NativeRuntime> {
// 		&mut self.app
// 	}
// }

impl Drop for NativeRuntime {
	fn drop(&mut self) {
		tracing::info!("💀 NativeRuntime DROPPED");
	}
}
