use crate::{app::state::EstateState, e, prelude::*, r#trait};

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
