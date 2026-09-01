use crate::{
	app::{state::*, *},
	e,
};

#[cfg(feature = "native")]
use tokio::sync::broadcast::error::TryRecvError;

// #[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "native")]
use crate::native::ui::IOState;

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,
	pub last_revision: u64,

	#[cfg(feature = "native")]
	#[cfg(not(target_arch = "wasm32"))]
	pub input: IOState,
	#[cfg(feature = "native")]
	#[cfg(not(target_arch = "wasm32"))]
	pub event_rx: tokio::sync::broadcast::Receiver<e::Event>,
}

impl<'a, R: Runtime + 'static> AppContext<'a, R> {
	pub fn load_problems(&mut self) {
		self.app.load_problems();
	}
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
	#[cfg(feature = "native")]
	#[cfg(not(target_arch = "wasm32"))]
	pub fn next_event(&mut self) -> Option<e::Event> {
		match self.event_rx.try_recv() {
			Ok(event) => Some(event),
			Err(TryRecvError::Empty) => None,
			Err(TryRecvError::Lagged(_)) => None,
			Err(TryRecvError::Closed) => None,
		}
	}
}
