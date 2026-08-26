use crate::{ app::*, prelude::* };

pub trait Runtime: Clone + Send + Sync {
	fn emit(&self, event: Event);
	fn start_dispatcher(self: &Arc<Self>);
	fn state(&self) -> &RuntimeState;
	fn save(&self, state: &EstateState) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub struct RuntimeState {
	pub state: Arc<RwLock<EstateState>>,
}

impl RuntimeState {
	pub fn new(state: EstateState) -> Self {
		Self {
			state: Arc::new(RwLock::new(state)),
		}
	}

	pub fn read(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.state.read().expect("RuntimeState lock poisoned")
	}

	pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, EstateState> {
		self.state.write().expect("RuntimeState lock poisoned")
	}
}
