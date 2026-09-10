use crate::{
	doc,
	prelude::{traits::Ctx, *},
	ui, ui_prelude as gui,
};

use anyhow::anyhow;

mod impls {
	use super::structs::*;
	use crate::prelude::*;

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
}

mod structs {
	use super::impls::*;
	use crate::prelude::*;

	pub struct Linux;
	pub struct MacOS;
	pub struct Windows;

	pub struct C;

	/// State vs Context is like "Nature vs Nurture",there is no perfect answer to what drives what.
	/// Every state depends on some context which depending on how you think of it, might be considered "state" as well.
	///
	/// So for now, in order to implement a Type State system robustly, we're going to agree that all apps/processes must come from a context.
	///
	/// Linux, MacOS, Windows, they're all contexts in which the app can run so we begin our app with that assumption for modeling more robustly.
	///
	#[derive(Debug)]
	pub struct S<C> {
		pub context: PhantomData<C>,
		pub state: PhantomData<C>,
		pub view: ViewType,
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use crate::app::host::NativeContext;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
use crate::app::host::WebContext;

impl<C> App<C>
where
	C: Ctx + 'static,
{
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::debug!("App New");

		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		let (cursor_event_tx, cursor_events) = std::sync::mpsc::channel();
		let state = structs::S::default();
		Ok(Self {
			state,
			host,
			workers: vec![],
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			cursor_events,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			cursor_event_tx,
		})
	}
	pub fn context(self) -> Arc<C> {
		self.host.context()
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn init_services(&mut self) -> Result<()> {
		tracing::debug!("App init services");
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			let handle = self.start_clock()?;
			self.workers.push(handle);
			let handle = self.start_cargo_watcher()?;
			tracing::debug!("cargo handle created");
			self.workers.push(handle);

			let handle = self.start_cursor_watcher_from_app()?;
			tracing::debug!("cursor handle created");
			self.workers.push(handle);
		}
		tracing::debug!("App init services complete");
		Ok(())
	}
	pub fn run(&mut self) -> Result<()> {
		tracing::debug!("App run");
		// self.init_services()?;
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			self.run_gui()?;
		}

		#[cfg(target_arch = "wasm32")]
		{
			// self.host.run()?;
		}

		Ok(())
	}

	fn run_gui(&mut self) -> Result<()> {
		let cancel = CancellationToken::new();
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.build()
			.expect("failed to build GUI event loop");
		let proxy = event_loop.create_proxy();
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		self.start_app_events(proxy.clone());
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		let mut renderer =
			Renderer::<NativeContext, structs::S<structs::C>>::new(self.state.clone(), cancel);
		#[cfg(all(feature = "web", target_arch = "wasm32"))]
		let mut renderer: Renderer<WebContext, structs::S<structs::C>> =
			Renderer::<WebContext, structs::S<structs::C>>::new(self.state.clone(), cancel);

		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		event_loop
			.run_app(&mut renderer)
			.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"));
		Ok(())
	}
	pub fn shutdown(&mut self) {
		tracing::debug!("App shutdown");
		for worker in &self.workers {
			worker.stop();
		}
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn start_app_events(
		&mut self,
		proxy: EventLoopProxy<AppEvent>,
	) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let handle = self.host.worker().run_background(move |cancel| async move {
			tracing::info!("🔥 APP EVENTS TASK STARTED");
			loop {
				tokio::select! {
					_ = cancel.cancelled() => {
						tracing::info!("🔥 APP EVENTS CANCELLED");
						break;
					}
					_ = tokio::time::sleep(Duration::from_secs(5)) => {
						tracing::info!("🔥 APP EVENTS SENDING");
							match proxy.send_event(AppEvent::RuntimeEvent) {
								Ok(()) => {
									tracing::info!("🔥 RuntimeEvent SENT");
								}
								Err(err) => {
									tracing::error!(?err, "🔥 RuntimeEvent FAILED");
								}
							}
					}
				}
			}
		});
		Ok(handle)
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn start_cargo_watcher(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		tracing::debug!("cargo: entered");

		let watcher = CargoWatcher::new().map_err(|error| {
			tracing::error!("Failed to create Cargo watcher: {error}");
			error
		})?;
		tracing::debug!("cargo: watcher created");

		tracing::debug!("Watching Cargo.toml: {}", watcher.path().display());
		let worker = self.worker();
		tracing::debug!("cargo: watcher started");

		Ok(worker.run_background_blocking(move |cancel| {
			let (tx, rx) = std::sync::mpsc::channel();

			let mut fs_watcher = match RecommendedWatcher::new(tx, Config::default()) {
				Ok(watcher) => watcher,
				Err(error) => {
					tracing::error!("Failed to create Cargo watcher: {error}");
					return;
				}
			};
			if let Err(error) = fs_watcher.watch(watcher.path(), RecursiveMode::NonRecursive) {
				tracing::error!("Failed to watch Cargo.toml: {error}");
				return;
			}
			loop {
				if cancel.is_cancelled() {
					break;
				}

				match rx.recv_timeout(std::time::Duration::from_millis(100)) {
					Ok(Ok(event)) => {
						if matches!(
							event.kind,
							notify::EventKind::Modify(_) | notify::EventKind::Create(_)
						) {
							tracing::debug!("Cargo.toml changed");

							if let Err(error) = watcher.run_once_sync() {
								tracing::error!("Failed to process Cargo.toml: {error}");
							}
						}
					}

					Ok(Err(error)) => {
						tracing::error!("Cargo watcher error: {error}");
					}

					Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
						// Allows us to check cancellation.
					}

					Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
						break;
					}
				}
			}
			tracing::debug!("Cargo watcher stopped");
		}))
	}

	fn start_clock(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let clock = self.host.clock();
		let msg = String::from("App.start_clock.clock.run_background(Duration::from_secs(1));");
		Ok(clock.run_background(Duration::from_secs(1), msg))
	}

	fn start_clock_wasm(&mut self) {
		let clock = self.host.clock();
		// clock.inherent_background_tick(String::from(
		// 	"let clock = self.host.clock(); clock.inherent_background_tick",
		// ));
		// let msg = String::from("start_clock_wasm clock.run_background(Duration::from_secs(1));");
		// clock.run_background(Duration::from_secs(1), msg.clone());
		// let handle = Clock::run_background(clock, Duration::from_secs(1), msg.clone());
	}
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn start_cursor_watcher_from_app(
		&mut self,
	) -> anyhow::Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let sink = AppCursorSink {
			tx: self.cursor_event_tx.clone(),
		};

		Ok(self.worker().run_background_blocking(move |cancel| {
			if let Err(error) = CursorDaemon::new(sink, cancel).run() {
				tracing::error!("Cursor daemon failed: {error}");
			}
		}))
	}

	fn worker(&self) -> &HostWorker<C> {
		self.host.worker()
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

impl<C, S> Renderer<C, S> {
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

pub struct App<C: Ctx> {
	state: structs::S<structs::C>,
	pub host: Host<C>,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,
}

pub struct Renderer<C, S> {
	phantom: PhantomData<C>,
	pub state: S,
	pub view: ViewType,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub windows: Vec<AppWindow>,
	pub cancel: CancellationToken,
}
