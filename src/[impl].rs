//! ## [Impl]
//!
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

impl Clone for Box<dyn Api> {
	fn clone(&self) -> Self {
		self.clone_box()
	}
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

impl<C> S<C>
where
	C: Ctx,
{
	pub fn window_count(&self, _context: &C) -> usize {
		todo!("window_count")
		// can't access `windows` here because Ctx doesn't guarantee it
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

impl<C, S> structs::Renderer<C, S>
where
	C: Ctx,
	S: 'static,
{
	pub fn new(
		context: Arc<C>,
		state: S,
		cancel: CancellationToken,
		event_rx: C::EventReceiver,
		event_tx: C::EventSender,
	) -> Self {
		Self {
			cancel,
			state,
			event_rx,
			event_tx,
			context,
			view: ViewType::MarkdownScreen,
			// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			windows: vec![],
		}
	}
	pub fn process_events(&mut self) {
		tracing::info!("renderer process_events");
		match self.event_rx.try_recv() {
			Some(event) => {
				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					// event_rx: &mut self.event_rx,
					input: IOState::default(),
					last_revision: 0,
				};

				tracing::info!("windows: {}", self.windows.len());

				for window in &mut self.windows {
					tracing::info!("dispatching event to screen");
					window.window.screen.event(&event, &mut ctx);
				}
			}

			None => {
				tracing::info!("RENDERER GOT NO EVENT");
			}
		}
	}
	pub fn _process_events(&mut self) {
		tracing::info!("renderer process_events");
		// loop {
		match self.event_rx.try_recv() {
			Some(event) => {
				tracing::info!(?event, "RENDERER GOT EVENT");

				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					// event_rx: &mut self.event_rx,
					input: IOState::default(),
					last_revision: 0,
				};

				tracing::info!("windows: {}", self.windows.len());

				for window in &mut self.windows {
					tracing::info!("dispatching event to screen");
					window.window.screen.event(&event, &mut ctx);
				}
			}

			None => {
				tracing::info!("RENDERER GOT NO EVENT");
				// break;
				// }
			}
		}
	}
	pub fn sync_views(&mut self) {
		tracing::info!("sync_views");
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		for window in &mut self.windows {
			window.view = self.view;
			window.window.sync_view(window.view);
			window.window.instance.set_title(self.view.name());
			window.window.instance.request_redraw();
		}
	}
	pub fn navigate_to(&mut self, view: ViewType) {
		tracing::debug!("navigating from {:?} to {:?}", self.view, view,);
		self.view = view;
		self.sync_views();
	}
	pub fn app_context(&mut self) -> AppContext<'_, C, S> {
		AppContext {
			context: self.context.as_ref(),
			state: &mut self.state,
			// event_rx: &mut self.event_rx,
			event_tx: &mut self.event_tx,
			input: IOState::default(),
			last_revision: 0,
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
