//! ## [Impl]
//!
//! Kitchen draw placement for impls until placement has stabilized.
//!
use crate::{
	prelude::*,
	structs::{C, S},
};

impl<C> Clone for S<C> {
	fn clone(&self) -> Self {
		Self {
			context: PhantomData,
			state: PhantomData,
			view: self.view.clone(),
		}
	}
}
/// Manually implement Default specifically for S<C>
///
impl Default for S<C> {
	fn default() -> Self {
		S {
			view: ViewType::MarkdownScreen,
			context: PhantomData,
			state: PhantomData,
		}
	}
}

impl<C, S> structs::Renderer<C, S> {
	pub fn new(state: S, cancel: CancellationToken) -> Self {
		Self {
			cancel,
			state,
			phantom: PhantomData,
			view: ViewType::MarkdownScreen,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			windows: vec![],
		}
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl traits::Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn render(&mut self) {
		// native rendering
	}
}

impl Default for EstateState {
	fn default() -> Self {
		Self {
			events_processed: 0,
			files_indexed: 0,
			longest_run: 0,
			jobs: VecDeque::new(),
			revision: 0,
			started_at: 0,
			starts: 0,
			status_checks: 0,
			tasks_completed: 0,
			tasks_created: 0,
			session: Session::default(),
		}
	}
}
impl EstateState {
	pub fn save_workspace(path: &PathBuf) {
		println!("💾 save_workspace not implemented yet: {:?}", path);
	}
	pub fn now() -> u64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
	}
}
