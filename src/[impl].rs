//! ## [Impl]
//!
//! Kitchen draw placement for impls until API stabilizes.
//!
use crate::{
	prelude::*,
	structs::{C, S},
};

pub async fn sleep(duration: Duration) {
	#[cfg(not(target_arch = "wasm32"))]
	tokio::time::sleep(duration).await;

	#[cfg(target_arch = "wasm32")]
	gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}

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

/// ## Default implementation for [S]
///
impl Default for S<C> {
	fn default() -> Self {
		S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
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

impl structs::State {
	pub fn new() -> Self {
		Self {}
	}
}

/// [HostRenderer]
///
/// It's unclear which of Eguis APIs work on all target platforms.
///
/// When we're confident that all features can render on all platforms thi guy should hold the root
/// rendering logic.
///
impl traits::Renders for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn render(&mut self) {
		// native rendering
	}
}
