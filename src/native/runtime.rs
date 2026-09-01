pub use crate::native::{
	job::TaskManager, monitor::NativeMonitor, prelude::*, state::NativeStateStore,
};
use crate::{
	app::{
		Runtime,
		state::{EstateState, StateStore},
	},
	e,
};
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	pub session: Session,
	pub store: NativeStateStore,
	pub events: EventBus,
	pub state: Arc<RuntimeState>,
	pub tasks: Arc<RwLock<TaskManager>>,
	handle: tokio::runtime::Handle,
	event_rx: Arc<Mutex<broadcast::Receiver<e::Event>>>,
}
impl NativeRuntime {
	pub fn new(handle: tokio::runtime::Handle) -> Result<Self> {
		let store = NativeStateStore::new()?;
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);
		let events = EventBus::new();
		let event_rx = Arc::new(Mutex::new(events.subscribe()));
		Ok(Self {
			event_rx,
			events,
			handle,
			session: Session::default(),
			state: Arc::new(runtime_state),
			store,
			tasks: Arc::new(RwLock::new(TaskManager::new())),
		})
	}
}
impl Runtime for NativeRuntime {
	fn emit(&self, event: e::Event) {
		self.events.emit(event);
	}
	fn state(&self) -> &RuntimeState {
		&self.state
	}
	fn save(&self, state: &EstateState) -> Result<()> {
		let session = self.session.clone();
		self.store.save(state)
	}
	fn session(&self) -> Session {
		self.session.clone()
	}
	type EventReceiver = NativeEventReceiver;
	fn subscribe(&self) -> Self::EventReceiver {
		NativeEventReceiver {
			rx: self.events.subscribe(),
		}
	}
	fn try_recv(&self) -> Option<e::Event> {
		self.event_rx.lock().unwrap().try_recv().ok()
	}
	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + Send + 'static,
	{
		let handle = self.handle.clone();
		handle.spawn(future);
	}
	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::new();
		// Creating, scheduling, executing, completing tasks
		dispatcher.register(crate::event::handler::TaskHandler);
		// Updating persisted/application state
		dispatcher.register(crate::event::handler::StateHandler);
		// User/application commands
		dispatcher.register(crate::event::handler::CommandHandler);
		// Filesystem change events
		dispatcher.register(crate::event::handler::FileWatcherHandler);
		dispatcher.register(crate::event::handler::AppHandler);
		dispatcher.register(crate::event::handler::NavigationHandler);
		tokio::spawn(async move {
			loop {
				match receiver.recv().await {
					Ok(event) => {
						tracing::debug!("🔥 native::runtime::dispatcher {:?}", event.kind);
						dispatcher.dispatch(event, &runtime).await;
					}
					Err(broadcast::error::RecvError::Lagged(count)) => {
						tracing::warn!(count, "native::runtime::start_disptcher lagged");
					}
					Err(broadcast::error::RecvError::Closed) => {
						tracing::warn!("native::runtime::start_disptcher closed");
						break;
					}
				}
			}
		});
	}
}
impl NativeRuntime {
	pub fn event_processed(&self) {
		let mut state = self.state.write();

		state.events_processed += 1;
	}
	pub fn subscribe(&self) -> broadcast::Receiver<e::Event> {
		self.events.subscribe()
	}
}
pub struct NativeAppContext<'a> {
	pub base: AppContext<'a, NativeRuntime>,
	pub monitor: &'a mut NativeMonitor,
}
impl<'a> NativeAppContext<'a> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.base.state()
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn poll_state(&mut self) -> bool {
		todo!("")
	}
}
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
