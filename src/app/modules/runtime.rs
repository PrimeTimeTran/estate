use crate::{app::*, native::session::Session, prelude::*};

pub trait Runtime: Clone + Send + Sync {
	/// Services own long-lived responsibilities and their concurrency/lifecycle;
	///
	/// Events are the standardized mechanism by which those services expose meaningful
	/// changes to the rest of the application; the Runtime owns the services and EventBus,
	/// while the Dispatcher routes those events to consumers.
	fn emit(&self, event: Event);
	fn start_dispatcher(self: &Arc<Self>);
	fn state(&self) -> &RuntimeState;
	fn save(&self, state: &EstateState) -> Result<()>;
	fn session(&self) -> Session;
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
