use crate::prelude::*;

impl<C> App<C>
where
	C: AppCtx + 'static,
{
	#[cfg(target_arch = "wasm32")]
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::info!("App New");
		Ok(Self {
			host,
			workers: vec![],
		})
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::info!("App New");
		let (cursor_event_tx, cursor_events) = std::sync::mpsc::channel();
		Ok(Self {
			cursor_events,
			cursor_event_tx,
			host,
			workers: vec![],
		})
	}
	pub fn clock(&self) -> &HostClock {
		&self.clock()
	}
	pub fn context(&self) -> &C {
		self.host.context()
	}

	pub fn run(&mut self) -> Result<()> {
		tracing::info!("App run");
		self.start()?;

		#[cfg(not(target_arch = "wasm32"))]
		{
			// self.host.run()?;
			self.worker().wait_for_ctrl_c();
			self.shutdown();
		}

		#[cfg(target_arch = "wasm32")]
		{
			self.host.run()?;
		}

		Ok(())
	}
	fn run_app(&mut self) -> Result<()> {
		todo!("")
	}

	#[cfg(all(not(feature = "web")))]
	fn run_gui(&mut self) -> Result<()> {
		let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
		let proxy = event_loop.create_proxy();
		self.host.worker().spawn_ctrl_c(proxy);
		event_loop.run_app(self)?;
		Ok(())
	}

	pub fn shutdown(&mut self) {
		for worker in &self.workers {
			worker.stop();
		}
		// self.host.shutdown();
	}
	pub fn start(&mut self) -> Result<()> {
		tracing::info!("App start");
		let handle = self.start_clock()?;
		self.workers.push(handle);

		#[cfg(not(target_arch = "wasm32"))]
		{
			// spawn_global_cursor_daemon_new
			let handle = self.start_cargo_watcher()?;
			self.workers.push(handle);
			let handle = self.start_cursor_watcher_from_app()?;
			self.workers.push(handle);
		}
		Ok(())
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn start_cargo_watcher(&mut self) -> Result<WorkHandle<C>> {
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
	fn start_clock(&mut self) -> Result<WorkHandle<C>> {
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
	fn start_egui(&mut self) -> Result<WorkHandle<C>> {
		tracing::info!("App start_egui");

		#[cfg(not(target_arch = "wasm32"))]
		{
			Ok(self.worker().run_background_blocking(move |cancel| {
				// egui work
				// check `cancel`
			}))
		}

		#[cfg(target_arch = "wasm32")]
		{
			Ok(self.worker().run_background(move |cancel| async move {
				// egui work
				// check `cancel`
			}))
		}
	}
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub fn start_cursor_watcher_from_app(&mut self) -> anyhow::Result<WorkHandle<C>> {
		let sink = AppCursorSink {
			tx: self.cursor_event_tx.clone(),
		};

		Ok(self.worker().run_background_blocking(move |cancel| {
			if let Err(error) = CursorDaemon::new(sink, cancel).run() {
				tracing::error!("Cursor daemon failed: {error}");
			}
		}))
	}
	// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	// fn start_cursor_watcher_from_app2(&mut self) -> Result<WorkHandle<C>> {
	// 	let sink = AppCursorSink {
	// 		tx: self.cursor_event_tx.clone(),
	// 	};
	// 	Ok(self.worker().run_background_blocking(move |cancel| {
	// 		if let Err(error) = CursorDaemon::new(sink, cancel).run() {
	// 			tracing::error!("Cursor daemon failed: {error}");
	// 		}
	// 	}))
	// }
	fn worker(&self) -> &HostWorker<C> {
		self.host.worker()
	}
}

#[cfg(feature = "native")]
impl<C> ApplicationHandler<AppEvent> for App<C>
where
	C: AppCtx,
{
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		// self.app.update();
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			// self.handle_event(event, event_loop);
		}
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		todo!("")
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
		todo!("")
		// let Some(window) = self
		// 	.context()
		// 	.windows
		// 	.iter_mut()
		// 	.find(|window| window.window.instance.id() == window_id)
		// else {
		// 	return;
		// };
		// let response = window
		// 	.window
		// 	.gui_state
		// 	.on_window_event(&window.window.instance, &event);
		// if response.repaint {
		// 	window.window.instance.request_redraw();
		// }
		// match event {
		// 	WindowEvent::CloseRequested => {
		// 		tracing::info!("🛑 Window close requested for id: {:?}", window_id);
		// 		self
		// 			.context()
		// 			.windows
		// 			.retain(|window| window.window.instance.id() != window_id);
		// 		return;
		// 	}
		// 	WindowEvent::RedrawRequested => {
		// 		if window.window.occluded {
		// 			return;
		// 		}
		// 		let menu = {
		// 			// let event_rx = self.app.engine.runtime().subscribe();
		// 			// let mut ctx = AppContext {
		// 			// 	app: &mut self.app,
		// 			// 	input: IOState::default(),
		// 			// 	event_rx,
		// 			// 	last_revision: 0,
		// 			// };
		// 			// if let Err(e) = window.window.draw(&mut ctx) {
		// 			// 	tracing::error!("DEV >>> draw failed: {e:#}");
		// 			// }
		// 		};
		// 	}
		// 	WindowEvent::Focused(true) => {
		// 		window.window.instance.request_redraw();
		// 	}
		// 	WindowEvent::Occluded(occluded) => {
		// 		window.window.occluded = occluded;
		// 		if !occluded {
		// 			window.window.instance.request_redraw();
		// 		}
		// 	}
		// 	WindowEvent::Resized(size) => {
		// 		if size.width == 0 || size.height == 0 {
		// 			return;
		// 		}
		// 		window.window.config.width = size.width;
		// 		window.window.config.height = size.height;
		// 		window
		// 			.window
		// 			.surface
		// 			.configure(&window.window.device, &window.window.config);
		// 		window.window.needs_resize = false;
		// 		window.window.instance.request_redraw();
		// 	}
		// 	_ => {}
		// }
	}
	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		todo!("")
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
	pub host: Host<C>,
	pub workers: Vec<WorkHandle<C>>,
	#[cfg(not(target_arch = "wasm32"))]
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	#[cfg(not(target_arch = "wasm32"))]
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,
}

#[derive(Debug, Clone)]
pub struct CursorDaemon<S> {
	pub sink: S,
	pub cancel: CancellationToken,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
	pub x: f64,
	pub y: f64,
}

// "This is a unit of work that I know how to stop."
pub struct WorkHandle<C>
where
	C: AppCtx,
{
	pub cancel: CancellationToken,

	#[cfg(not(target_arch = "wasm32"))]
	pub join: JoinHandle<()>,

	_phantom: PhantomData<C>,
}

impl<C> WorkHandle<C>
where
	C: AppCtx,
{
	#[cfg(target_arch = "wasm32")]
	pub fn new(cancel: CancellationToken) -> Self {
		Self {
			cancel,
			_phantom: PhantomData,
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(cancel: CancellationToken, join: JoinHandle<()>) -> Self {
		Self {
			cancel,
			join,
			_phantom: PhantomData,
		}
	}

	pub fn stop(&self) {
		self.cancel.cancel();
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub async fn join(self) -> Result<(), tokio::task::JoinError> {
		self.join.await
	}
}

// pub struct GuiDriver {
// 	// winit/egui stuff
// }

// impl<C> AppDriver<C> for GuiDriver
// where
// 	C: AppCtx,
// {
// 	fn run(&mut self, app: &mut App<C>) -> Result<()> {
// 		// start winit
// 		// run event loop
// 		// render egui
// 		// receive shutdown
// 		Ok(())
// 	}
// }

// pub struct LspDriver;

// impl<C> AppDriver<C> for LspDriver
// where
// 	C: AppCtx,
// {
// 	fn run(&mut self, app: &mut App<C>) -> Result<()> {
// 		// stdin/stdout JSON-RPC loop
// 		Ok(())
// 	}
// }

// pub struct VsCodeDriver;
