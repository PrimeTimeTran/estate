use crate::app::{state::*, *};

// #[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "native")]
use crate::native::ve::VeInputState;

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,
	pub last_revision: u64,

	#[cfg(not(target_arch = "wasm32"))]
	pub input: VeInputState,
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
}
