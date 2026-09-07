use crate::prelude::*;

use tokio_util::sync::CancellationToken;

#[cfg(not(feature = "web"))]
pub async fn test_main_2() {
	let clock = HostClock;
	let worker = HostWorker::new();

	let handle = worker.run_background(move |cancel| async move {
		loop {
			tokio::select! {
					_ = cancel.cancelled() => break,

					_ = tokio::time::sleep(Duration::from_secs(1)) => {
							println!("background tick: {}", clock.now());
					}
			}
		}
	});

	tokio::time::sleep(Duration::from_secs(5)).await;

	handle.stop();
}

#[cfg(not(feature = "web"))]
#[tokio::main]
pub async fn tokio_main() {
	let clock = HostClock;
	let worker = HostWorker::new();
	let handle = worker.run_background(move |cancel| async move {
		loop {
			tokio::select! {
					_ = cancel.cancelled() => {
							break;
					}
					_ = tokio::time::sleep(Duration::from_secs(1)) => {
							println!("background tick: {}", clock.now());
					}
			}
		}
	});
	tokio::time::sleep(Duration::from_secs(5)).await;
	handle.stop();
}

impl<C> App<C>
where
	C: AppCtx,
{
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::info!("App New");
		Ok(Self {
			host,
			handle_clock: None,
			handle_egui: None,
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
			self.host.run()?;
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

		// Host owns infrastructure.
		self.host.worker().spawn_ctrl_c(proxy);

		// App owns GUI behavior.
		event_loop.run_app(self)?;

		Ok(())
	}
	pub fn shutdown(&mut self) {
		if let Some(handle) = self.handle_clock.take() {
			handle.stop();
		}

		if let Some(handle) = self.handle_egui.take() {
			handle.stop();
		}
	}
	pub fn start(&mut self) -> Result<()> {
		tracing::info!("App start");
		self.start_egui();
		self.start_clock();
		#[cfg(not(target_arch = "wasm32"))]
		self.start_cargo_watcher();
		Ok(())
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn start_cargo_watcher(&mut self) {
		let watcher = match CargoWatcher::new() {
			Ok(watcher) => watcher,
			Err(error) => {
				tracing::error!("Failed to create Cargo watcher: {error}");
				return;
			}
		};
		tracing::info!("Watching Cargo.toml: {}", watcher.path().display());
		self.worker().run_background_blocking(move |cancel| {
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
				match rx.recv() {
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
					Err(_) => break,
				}
			}

			tracing::info!("Cargo watcher stopped");
		});
	}
	fn start_clock(&mut self) {
		let clock = self.host.clock();
		let msg = String::from("App.start_clock.clock.run_background(Duration::from_secs(1));");
		// handle works
		let handle = clock.run_background(Duration::from_secs(1), msg);
		// So does the inherent
		// let msg = String::from("Loi start_clock Clock::run_background(clock, Duration::from_secs(1));");
		// let handle = Clock::run_background(clock, Duration::from_secs(1), msg);
		self.handle_clock = Some(handle);
	}
	/// Inherent works method works
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
	fn start_egui(&mut self) {
		tracing::info!("App start_egui");
		let cancel = CancellationToken::new();
		// Install/register your egui hook here.
		//
		// The hook should retain `cancel.clone()` if it needs
		// to check for shutdown.
		self.handle_egui = Some(EguiHandle { cancel });
	}
	fn worker(&self) -> &impl Worker {
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

impl EguiHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
	}
	// winit different?
	// pub fn stop(self) {
	// 	// unregister hook
	// }
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
impl WorkerHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

pub struct App<C: AppCtx> {
	pub host: Host<C>,
	pub handle_clock: Option<ClockHandle>,
	pub handle_egui: Option<EguiHandle>,
}

pub struct ClockHandle {
	pub cancel: CancellationToken,
}

pub struct EguiHandle {
	pub cancel: CancellationToken,
	// winit diff
	// Whatever is necessary to unregister the egui hook.
}
