use crate::prelude::{traits::Ctx, *};

impl<C> App<C>
where
	C: Ctx + 'static,
{
	pub fn context(self) -> Arc<C> {
		self.host.context()
	}

	fn init_services(&mut self) -> Result<()> {
		tracing::debug!("App init services");
		let handle = self.start_clock()?;
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
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
		self.init_services()?;
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

	#[cfg(all(feature = "web", target_arch = "wasm32"))]
	fn run_gui(&mut self) -> Result<()> {
		Ok(())
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn run_gui(&mut self) -> Result<()> {
		let cancel = CancellationToken::new();
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.build()
			.expect("failed to build GUI event loop");
		let proxy = event_loop.create_proxy();
		self.start_app_events(proxy.clone());
		let mut renderer =
			structs::Renderer::<NativeContext, structs::S<structs::C>>::new(self.state.clone(), cancel);
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
		let msg = String::from("start_clock_wasm clock.run_background(Duration::from_secs(1));");
		// clock.run_background(Duration::from_secs(1), msg.clone());
		// clock.run_background::<C, ()>(Duration::from_secs(1), msg.clone())
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
}

pub struct App<C: Ctx> {
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,

	pub host: Host<C>,
	state: structs::S<structs::C>,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
}
