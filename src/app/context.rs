use crate::app::EstateState;
use crate::{ app::{ App, Runtime, monitor::Monitor } };

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,

	#[cfg(not(target_arch = "wasm32"))]
	pub monitor: &'a mut crate::app::monitor::NativeMonitor,
}

impl<'a, R: Runtime> AppContext<'a, R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.app.state()
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn poll_state(&mut self) -> bool {
		self.monitor.poll()
	}
}
