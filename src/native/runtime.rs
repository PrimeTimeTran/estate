use crate::{
	app::{Runtime, host::AppHost, modules::runtime::RuntimeState, state::StateStore, *},
	native::state::NativeStateStore,
	prelude::*,
};

pub struct NativeAppContext<'a> {
	pub base: AppContext<'a, NativeRuntime>,
	pub monitor: &'a mut monitor_native::NativeMonitor,
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

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	pub store: NativeStateStore,
	pub events: EventBus,
	pub state: Arc<RuntimeState>,
	pub tasks: Arc<RwLock<TaskManager>>,
}

impl Runtime for NativeRuntime {
	fn emit(&self, event: Event) {
		self.events.emit(event);
	}
	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::new();
		dispatcher.register(TaskHandler);
		dispatcher.register(StateHandler);
		dispatcher.register(CommandHandler);
		dispatcher.register(FileWatcherHandler);
		tokio::spawn(async move {
			loop {
				match receiver.recv().await {
					Ok(event) => {
						tracing::info!("🔥 DISPATCHER RECEIVED: {:?}", event.kind);
						dispatcher.dispatch(event, &runtime).await;
					}
					Err(broadcast::error::RecvError::Lagged(count)) => {
						tracing::warn!(count, "event dispatcher lagged");
					}
					Err(broadcast::error::RecvError::Closed) => {
						println!("🔥 EVENT BUS CLOSED");
						break;
					}
				}
			}
		});
	}
	fn state(&self) -> &RuntimeState {
		&self.state
	}
	fn save(&self, state: &EstateState) -> anyhow::Result<()> {
		self.store.save(state)
	}
	// fn poll_state(&mut self) -> bool {
	// 	self.state.revision()
	// }
}

impl NativeRuntime {
	pub fn new() -> anyhow::Result<Self> {
		let store = NativeStateStore::new()?;
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);

		Ok(Self {
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
}

impl AppHost<NativeRuntime> for NativeApp {
	fn app(&mut self) -> &mut App<NativeRuntime> {
		&mut self.app
	}
}
