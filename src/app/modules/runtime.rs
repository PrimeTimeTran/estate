use crate::{
	app::{state::EstateState, *},
	e,
	prelude::*,
};

// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
//
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
	// Services own long-lived responsibilities and their concurrency/lifecycle;
	//
	// Events are the standardized mechanism by which those services expose meaningful
	// changes to the rest of the application; the Runtime owns the services and EventBus,
	// while the Dispatcher routes those events to consumers.
	type EventReceiver: EventReceiver;
	fn emit(&self, event: e::Event);
	fn subscribe(&self) -> Self::EventReceiver;
	fn try_recv(&self) -> Option<e::Event>;
	fn start_dispatcher(self: &Arc<Self>);
	fn state(&self) -> &RuntimeState;
	fn save(&self, state: &EstateState) -> Result<()>;
	fn session(&self) -> Session;
	fn spawn<F>(&self, future: F)
	where
		F: std::future::Future<Output = ()> + Send + 'static;
	fn sleep(
		&self,
		duration: std::time::Duration,
	) -> impl std::future::Future<Output = ()> + Send + '_;
}

#[derive(Debug)]
pub struct RuntimeState {
	state: Arc<RwLock<EstateState>>,
	revision: AtomicU64,
}

impl RuntimeState {
	pub fn new(state: EstateState) -> Self {
		Self {
			state: Arc::new(RwLock::new(state)),
			revision: AtomicU64::new(0),
		}
	}

	pub fn read(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.state.read().expect("RuntimeState lock poisoned")
	}

	pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, EstateState> {
		self.revision.fetch_add(1, Ordering::Relaxed);
		self.state.write().expect("RuntimeState lock poisoned")
	}

	pub fn revision(&self) -> u64 {
		self.revision.load(Ordering::Relaxed)
	}
}
