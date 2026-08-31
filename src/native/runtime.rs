use tokio::sync::broadcast;

use crate::{
	app::{
		Runtime,
		host::AppHost,
		state::{EstateState, StateStore},
	},
	e,
	native::NativeApp,
};

pub use crate::native::{
	job::TaskManager, monitor::NativeMonitor, prelude::*, state::NativeStateStore,
};

impl NativeRuntime {
	pub fn new() -> Result<Self> {
		let store = NativeStateStore::new()?;
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);
		let session = Session::default();

		Ok(Self {
			session,
			store,
			events: EventBus::new(),
			state: Arc::new(runtime_state),
			tasks: Arc::new(RwLock::new(TaskManager::new())),
		})
	}
	pub fn event_processed(&self) {
		let mut state = self.state.write();
		state.events_processed += 1;
	}
	pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<e::Event> {
		self.events.subscribe()
	}
}

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	pub session: Session,
	pub store: NativeStateStore,
	pub events: EventBus,
	pub state: Arc<RuntimeState>,
	pub tasks: Arc<RwLock<TaskManager>>,
}

impl Runtime for NativeRuntime {
	fn emit(&self, event: e::Event) {
		self.events.emit(event);
	}
	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::new();
		// Creating, scheduling, executing, completing tasks
		dispatcher.register(event::handler::TaskHandler);
		// Updating persisted/application state
		dispatcher.register(event::handler::StateHandler);
		// User/application commands
		dispatcher.register(event::handler::CommandHandler);
		// Filesystem change events
		dispatcher.register(event::handler::FileWatcherHandler);
		dispatcher.register(event::handler::AppHandler);
		dispatcher.register(event::handler::NavigationHandler);
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

	fn subscribe(&self) -> tokio::sync::broadcast::Receiver<e::Event> {
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

// impl AppHost<NativeRuntime> for NativeApp {
// 	fn app(&mut self) -> &mut App<NativeRuntime> {
// 		&mut self.app
// 	}
// }
