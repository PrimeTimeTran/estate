use crate::app::{monitor::Monitor, state::VeInputState, *};

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,
	pub input: VeInputState,
	#[cfg(not(target_arch = "wasm32"))]
	pub monitor: &'a mut monitor_native::NativeMonitor,
}

// Param Types
// - R is type param
// - 'a is lifetime param
impl<'a, R: Runtime> AppContext<'a, R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.app.state()
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn poll_state(&mut self) -> bool {
		self.monitor.poll()
	}
}
