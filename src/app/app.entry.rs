use crate::prelude::{traits::AppCtx as Ctx, *};

impl<C> App<C>
where
	C: Ctx + 'static,
{
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::info!("App New");
		#[cfg(not(target_arch = "wasm32"))]
		let (cursor_event_tx, cursor_events) = std::sync::mpsc::channel();
		Ok(Self {
			host,
			// ctx: C,
			workers: vec![],
			gui: None,
			#[cfg(not(target_arch = "wasm32"))]
			cursor_events,
			#[cfg(not(target_arch = "wasm32"))]
			cursor_event_tx,
		})
	}
	pub fn clock(&self) -> &HostClock {
		&self.clock()
	}

	pub fn ctx(&self) -> Arc<C> {
		self.host.context()
	}
	fn init_services(&mut self) -> Result<()> {
		tracing::info!("App init services");
		let handle = self.start_clock()?;
		self.workers.push(handle);
		#[cfg(not(target_arch = "wasm32"))]
		{
			let handle = self.start_cargo_watcher()?;
			self.workers.push(handle);
			let handle = self.start_cursor_watcher_from_app()?;
			self.workers.push(handle);
		}

		Ok(())
	}
	pub fn run(&mut self) -> Result<()> {
		tracing::info!("App run");

		self.init_services()?;

		#[cfg(not(target_arch = "wasm32"))]
		{
			self.run_gui()?;
			self.shutdown();
		}

		#[cfg(target_arch = "wasm32")]
		{
			self.host.run()?;
		}

		Ok(())
	}
	fn run_gui(&mut self) -> Result<()> {
		let cancel = CancellationToken::new();
		let mut gui_app = GuiApp {
			state: self.ctx().state().clone(),
			cancel,
		};
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.with_activation_policy(ActivationPolicy::Regular)
			.build()
			.expect("failed to build GUI event loop");
		event_loop
			.run_app(&mut gui_app)
			.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"))
	}
	pub fn shutdown(&mut self) {
		tracing::info!("App shutdown");

		for worker in &self.workers {
			worker.stop();
		}

		if let Some(gui) = &self.gui {
			gui.stop();
		}
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn start_cargo_watcher(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let watcher = CargoWatcher::new().map_err(|error| {
			tracing::error!("Failed to create Cargo watcher: {error}");
			error
		})?;
		tracing::info!("Watching Cargo.toml: {}", watcher.path().display());
		let worker = self.worker();
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
							tracing::info!("Cargo.toml changed");

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
			tracing::info!("Cargo watcher stopped");
		}))
	}
	fn start_clock(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let clock = self.host.clock();
		let msg = String::from("App.start_clock.clock.run_background(Duration::from_secs(1));");
		Ok(clock.run_background(Duration::from_secs(1), msg))
	}
	fn start_clock_wasm(&mut self) {
		// self
		// 	.host
		// 	.clock()
		// 	.inherent_background_tick(String::from("self.host.clock().inherent_background_tick"));
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
		self.worker()
	}
}

impl<S> ApplicationHandler<AppEvent> for GuiApp<S>
where
	S: Send + Sync + 'static,
{
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("about_to_wait");
		// self.app.update();
		#[cfg(not(target_arch = "wasm32"))]
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			// self.handle_event(event, event_loop);
		}
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("Resumed");
		// todo!("")
		// if self.context().menu_bar.is_none() {
		// 	// let menu = Self::menu_bar(true);
		// 	// menu.init_for_nsapp();
		// 	// self.menu_bar = Some(menu);
		// }
		// if self.context().windows.is_empty() {
		// 	// self.open_window(event_loop, crate::START_WINDOW);
		// }
		// if self.context().tray_clock.is_none() {
		// 	// let (menu, tray) = match Self::bootstrap() {
		// 	// 	Ok(value) => value,
		// 	// 	Err(error) => {
		// 	// 		tracing::error!(%error, "failed to bootstrap tray");
		// 	// 		return;
		// 	// 	}
		// 	// };
		// 	// self.menu = Some(menu);
		// 	// self.tray_clock = Some(tray);
		// 	tracing::debug!("🔥 main tray initialized");
		// }
		// if self.context().tray_cursor.is_none() {
		// 	match TrayIconBuilder::new()
		// 		.with_icon(scroll_tray_icon())
		// 		.with_tooltip("Estate Scroll Controller")
		// 		.build()
		// 	{
		// 		Ok(tray) => {
		// 			self.context().tray_cursor = Some(tray);
		// 			tracing::debug!("🔥 scroll tray initialized");
		// 		}
		// 		Err(error) => {
		// 			tracing::error!(%error, "failed to create scroll tray");
		// 		}
		// 	}
		// }
	}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		println!("window_event");
		// let Some(window) = self.state() else {
		// 	return;
		// };
		// let response = window
		// 	.window
		// 	.gui_state
		// 	.on_window_event(&window.window.instance, &event);
		// if response.repaint {
		// 	window.window.instance.request_redraw();
		// }
		match event {
			// WindowEvent::CloseRequested => {
			// 	tracing::info!("🛑 Window close requested for id: {:?}", window_id);
			// 	self
			// 		.windows
			// 		.retain(|window| window.window.instance.id() != window_id);
			// 	return;
			// }
			// WindowEvent::RedrawRequested => {
			// 	if window.window.occluded {
			// 		return;
			// 	}
			// 	let menu = {
			// 		let event_rx = self.app.engine.runtime().subscribe();
			// 		let mut ctx = AppContext {
			// 			app: &mut self.app,
			// 			input: IOState::default(),
			// 			event_rx,
			// 			last_revision: 0,
			// 		};
			// 		if let Err(e) = window.window.draw(&mut ctx) {
			// 			tracing::error!("DEV >>> draw failed: {e:#}");
			// 		}
			// 	};
			// }
			// WindowEvent::Focused(true) => {
			// 	window.window.instance.request_redraw();
			// }
			// WindowEvent::Occluded(occluded) => {
			// 	window.window.occluded = occluded;
			// 	if !occluded {
			// 		window.window.instance.request_redraw();
			// 	}
			// }
			// WindowEvent::Resized(size) => {
			// 	if size.width == 0 || size.height == 0 {
			// 		return;
			// 	}
			// 	// window.window.config.width = size.width;
			// 	// window.window.config.height = size.height;
			// 	// window
			// 	// 	.window
			// 	// 	.surface
			// 	// 	.configure(&window.window.device, &window.window.config);
			// 	// window.window.needs_resize = false;
			// 	// window.window.instance.request_redraw();
			// }
			_ => {}
		}
	}

	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		tracing::info!("user_event");
		// match event {
		// 	AppEvent::RuntimeEvent => {
		// 		// self.app.update();
		// 		// self.sync_views();
		// 	}
		// 	AppEvent::Navigate(view) => {
		// 		// self.host.run();
		// 		// self.host.worker().
		// 		self
		// 			.host
		// 			.worker()
		// 			.runtime
		// 			.spawn(e::Event::app(e::Klass::Navigate(view)));
		// 		// self.runtime().emit(e::Event::app(e::Klass::Navigate(view)));
		// 		// self.app.update();
		// 		// self.sync_views();
		// 	}
		// 	AppEvent::Shutdown => {
		// 		tracing::info!(">>> shutdown event received");
		// 		self.shutdown();

		// 		tracing::info!(">>> event_loop.exit() called");
		// 	}
		// 	AppEvent::CursorPosition { x, y } => {
		// 		// let text = format!("↖ {:.0}  {:.0}", x, y);
		// 		// let text = format!("← {:.0}  {:.0}", x, y);
		// 		// let text = format!("→ {:.0}  {:.0}", x, y);
		// 		// let text = format!("↑ {:.0}  {:.0}", x, y);
		// 		// let text = format!("● {:.0}, {:.0}", x, y);
		// 		// let text = format!("◉ {:.0}, {:.0}", x, y);
		// 		let text = format!("⌖ {:.0}, {:.0}", x, y);
		// 		// let text = format!("🟢 {:.0}, {:.0}", x, y);
		// 		// let text = format!("🔵 {:.0}, {:.0}", x, y);
		// 		// let text = format!("🟡 {:.0}, {:.0}", x, y);
		// 		// let text = format!("🔴 {:.0}, {:.0}", x, y);
		// 		// let region = if x < 960.0 { "← LEFT" } else { "RIGHT →" };
		// 		if let Some(tray) = &self.tray_cursor {
		// 			let _ = tray.set_title(Some(text));
		// 		}
		// 	}
		// 	AppEvent::TickClock(text) => {
		// 		if let Some(tray) = &self.context().tray_clock {
		// 			// let _ = tray.set_title(Some(text));
		// 		}
		// 		// self.sync_views();
		// 	}
		// 	AppEvent::ModifiersChanged {
		// 		alt,
		// 		command,
		// 		ctrl,
		// 		shift,
		// 	} => {}
		// 	_ => {}
		// }
	}
}

impl traits::Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn render(&mut self) {
		// native rendering
	}
}

pub struct App<C: AppCtx> {
	// pub ctx: C,
	pub gui: Option<GuiHandle>,
	pub host: Host<C>,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
	#[cfg(not(target_arch = "wasm32"))]
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	#[cfg(not(target_arch = "wasm32"))]
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,
}

// "This is a unit of work that I know how to stop."
pub struct WorkHandle<C, J>
where
	C: AppCtx,
{
	pub cancel: CancellationToken,

	#[cfg(not(target_arch = "wasm32"))]
	pub join: J,

	_phantom: PhantomData<(C, J)>,
}
impl<C> WorkHandle<C, std::thread::JoinHandle<()>>
where
	C: AppCtx,
{
	pub fn join(self) -> std::thread::Result<()> {
		self.join()
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl<C> WorkHandle<C, tokio::task::JoinHandle<()>>
where
	C: AppCtx,
{
	pub async fn join(self) -> Result<JoinHandle<()>> {
		Ok(self.join)
	}
}
impl<C, J> WorkHandle<C, J>
where
	C: AppCtx,
{
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(cancel: CancellationToken, join: J) -> Self {
		Self {
			cancel,
			join,
			_phantom: PhantomData,
		}
	}

	#[cfg(target_arch = "wasm32")]
	pub fn new(cancel: CancellationToken) -> Self {
		Self {
			cancel,
			_phantom: PhantomData,
		}
	}

	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

pub struct GuiHandle {
	pub proxy: EventLoopProxy<AppEvent>,
}

impl GuiHandle {
	pub fn stop(&self) {
		let _ = self.proxy.send_event(AppEvent::Shutdown);
	}

	pub fn send(&self, event: AppEvent) -> Result<(), EventLoopClosed<AppEvent>> {
		self.proxy.send_event(event)
	}
}
pub struct GuiApp<S> {
	state: S,
	cancel: CancellationToken,
}
impl<S> GuiApp<S> {
	// fn state(&self) -> &C {
	// 	&self.context.state()
	// }
}
