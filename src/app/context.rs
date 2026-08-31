use crate::app::{state::*, *};

// #[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "native")]
use crate::native::ui::VeInputState;

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,
	pub last_revision: u64,

	#[cfg(not(target_arch = "wasm32"))]
	pub input: VeInputState,
	#[cfg(not(target_arch = "wasm32"))]
	pub event_rx: broadcast::Receiver<Event>,
}

impl<'a, R: Runtime> AppContext<'a, R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.app.state()
	}

	pub fn state_changed(&mut self) -> bool {
		let revision = self.app.runtime().state().revision();
		if revision != self.last_revision {
			self.last_revision = revision;
			true
		} else {
			false
		}
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn next_event(&mut self) -> Option<Event> {
		match self.event_rx.try_recv() {
			Ok(event) => Some(event),
			Err(broadcast::error::TryRecvError::Empty) => None,
			Err(broadcast::error::TryRecvError::Lagged(_)) => None,
			Err(broadcast::error::TryRecvError::Closed) => None,
		}
	}
}
