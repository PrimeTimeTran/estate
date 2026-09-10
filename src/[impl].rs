//! ## [Impl]
//!
//! Kitchen draw placement for impls until API stabilizes.
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

/// Manually implement Default specifically for S<C>
///
impl Default for S<C> {
	fn default() -> Self {
		S {
			view: ViewType::MarkdownScreen,
			state: PhantomData,
			context: PhantomData,
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

impl<E> EventSink<E> for std::sync::mpsc::Sender<E>
where
	E: Send + 'static,
{
	fn send(&self, event: E) {
		let _ = self.send(event);
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

/// WIP
///
/// It's unclear which of Eguis APIs work on all target platforms.
///
/// When we're confident that all features can render on all platforms thi guy should hold the root
/// rendering logic.
///
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
