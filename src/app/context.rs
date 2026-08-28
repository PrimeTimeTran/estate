use crate::app::{monitor::Monitor, *};

#[cfg(not(target_arch = "wasm32"))]
use crate::app::state::VeInputState;

pub struct AppContext<'a, R: Runtime> {
	pub app: &'a mut App<R>,
	#[cfg(not(target_arch = "wasm32"))]
	pub input: VeInputState,
}

impl<'a, R: Runtime> AppContext<'a, R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.app.state()
	}
	pub fn poll_state(&mut self) -> bool {
		false
	}
}

// pub struct AppContext<'a, R: Runtime> {
// 	pub app: &'a mut App<R>,
// 	pub input: VeInputState,
// 	#[cfg(not(target_arch = "wasm32"))]
// 	pub monitor: &'a mut monitor_native::NativeMonitor,
// }

// // Param Types
// // - R is type param
// // - 'a is lifetime param
// impl<'a, R: Runtime> AppContext<'a, R> {
// 	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
// 		self.app.state()
// 	}
// }
