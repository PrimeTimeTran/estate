use crate::{ app::{ Runtime }, prelude::*, share::{ self }, share::vfs::* };

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	pub events: EventBus,
	pub state: Arc<RwLock<EstateState>>,
	pub tasks: Arc<RwLock<TaskManager>>,
}

impl Default for NativeRuntime {
	fn default() -> Self {
		Self::new()
	}
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
}

impl NativeRuntime {
	pub fn new() -> Self {
		tracing::info!("NativeRuntime new");
		Self {
			events: EventBus::new(),
			state: Arc::new(RwLock::new(EstateState::load())),
			tasks: Arc::new(RwLock::new(TaskManager::new())),
		}
	}
	pub fn event_processed(&self) {
		let mut state = self.state.write().unwrap();
		state.events_processed += 1;
	}
}
