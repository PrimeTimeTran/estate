#[cfg(all(feature = "native", not(target_arch = "wasm")))]
pub mod native;

#[cfg(all(feature = "native", not(target_arch = "wasm")))]
pub use native::*;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;
#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use web::*;

use crate::prelude::*;

#[path = "./app.rs"]
pub mod app_runtime;
pub use app_runtime::*;

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
