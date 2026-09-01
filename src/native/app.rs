use crate::{
	AppEvent, DaemonCommand,
	app::{self, App, Runtime, model::EstateEngine},
	e,
	leetcode::{
		problem_service_client::ProblemServiceClient,
		submission_service_client::SubmissionServiceClient, types::Problem,
	},
	native::{self, runtime::NativeRuntime, screens::*, *},
	prelude::*,
	spawn_global_cursor_daemon,
	ui::{View, rendermd::MarkdownView},
};
use tonic::transport::Channel;
use tray_icon::{
	TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
};
use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::WindowId,
};

pub struct NativeApp {
	pub app: Option<App<NativeRuntime>>,
	pub windows: Vec<AppWindow>,
	clock_running: Arc<AtomicBool>,
	daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
	daemon_tx: mpsc::Sender<DaemonCommand>,
	engine: EstateEngine<NativeRuntime>,
	hotkey_manager: GlobalHotkeys,
	menu: Option<TrayMenu>,
	menu_bar: Option<Menu>,
	monitor: NativeMonitor,
	scroll_tray: Option<TrayIcon>,
	tray: Option<TrayIcon>,
	view_type: ViewType,
}
impl NativeApp {
	pub fn new() -> Result<Self> {
		let (daemon_tx, daemon_rx) = mpsc::channel(100);
		let tokio = tokio::runtime::Runtime::new()?;
		let handle = tokio.handle().clone();
		let runtime = NativeRuntime::new(handle)?;
		let engine = EstateEngine::new(runtime)?;
		Ok(Self {
			app: None,
			clock_running: Arc::new(AtomicBool::new(true)),
			daemon_rx: Some(daemon_rx),
			daemon_tx,
			engine,
			hotkey_manager: GlobalHotkeys::new().unwrap(),
			menu: None,
			menu_bar: None,
			monitor: NativeMonitor::new()?,
			scroll_tray: None,
			tray: None,
			view_type: crate::START_VIEW,
			windows: vec![],
		})
	}
	pub fn run(&mut self, cli: Cli) -> Result<()> {
		tracing::debug!(">>> NativeApp::run entered");
		let result = match cli.command {
			None | Some(Command::Start { .. }) | Some(Command::Tray) => self.start_runtime(),
			Some(_) => {
				let runtime = tokio::runtime::Runtime::new()?;
				runtime.block_on(async {
					let ctx = cli::context::Context::new();
					router::execute(cli, ctx, self.engine.clone()).await
				})
			}
		};
		tracing::debug!(">>> NativeApp::run returning");
		result
	}
	fn start_runtime(&mut self) -> Result<()> {
		let daemon_rx = self.daemon_rx.take().expect("daemon already started");
		let (ready_tx, ready_rx) =
			std::sync::mpsc::sync_channel::<Result<(tokio::runtime::Handle, Arc<ApiClient>)>>(1);
		Self::spawn_daemon(daemon_rx, Arc::clone(&self.engine.runtime), ready_tx);
		let (handle, api) = ready_rx.recv().expect("daemon failed to initialize")?;
		let instance = App::new(self.engine.clone(), api);
		instance
			.runtime()
			.emit(e::Event::app(e::EventKind::SessionStart));
		self.app = Some(instance);
		self.spawn_global_hotkey_daemon()?;
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.with_activation_policy(ActivationPolicy::Regular)
			.build()?;
		if let Some(menu) = &self.menu_bar {
			menu.init_for_nsapp();
		}
		let proxy = event_loop.create_proxy();
		self.spawn_clock(proxy.clone());
		self.spawn_cursor_daemon(proxy.clone());
		self.spawn_signal_handler(proxy.clone());
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::SessionStart));
		event_loop.run_app(self)?;
		tracing::info!(">>> NativeApp::start_runtime returning");
		Ok(())
	}
	fn spawn_clock(&mut self, proxy: EventLoopProxy<AppEvent>) {
		let running = Arc::clone(&self.clock_running);
		let runtime = self.engine.runtime();
		std::thread::spawn(move || {
			let mut current_time = 3;
			let mut view_index = 0;
			while running.load(Ordering::Relaxed) {
				let views = [
					ViewType::MarkdownView,
					ViewType::ProblemsScreen,
					ViewType::WaterfallChart,
					ViewType::ProblemsScreen,
					ViewType::TaskManager,
					ViewType::ProblemsScreen,
				];
				let _ = proxy.send_event(AppEvent::TickClock(format!(" {}s", current_time)));
				tracing::info!("tick {}", current_time);
				if current_time == 0 {
					current_time = 5;
					view_index = (view_index + 1) % views.len();
					let view = views[view_index];
					tracing::info!("⏩ Clock navigation → {:?}", view);
					runtime.emit(e::Event::app(e::EventKind::Navigate(view)));
					let _ = proxy.send_event(AppEvent::RuntimeEvent);
				} else {
					current_time -= 1;
				}
				std::thread::sleep(std::time::Duration::from_secs(1));
			}
		});
	}
	fn spawn_cursor_daemon(&mut self, proxy: EventLoopProxy<AppEvent>) {
		spawn_global_cursor_daemon(proxy)
	}
	fn spawn_global_hotkey_daemon(&mut self) -> Result<()> {
		self.hotkey_manager.start();
		Ok(())
	}
	fn spawn_daemon(
		mut rx: mpsc::Receiver<DaemonCommand>,
		runtime: Arc<NativeRuntime>,
		ready_tx: std::sync::mpsc::SyncSender<Result<(tokio::runtime::Handle, Arc<ApiClient>)>>,
	) {
		std::thread::spawn(move || {
			let tokio_runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
			tokio_runtime.block_on(async move {
				runtime.start_dispatcher();
				let api = match ApiClient::connect().await {
					Ok(api) => Arc::new(api),
					Err(error) => {
						let _ = ready_tx.send(Err(error));
						return;
					}
				};
				let handle = tokio::runtime::Handle::current();
				let _ = ready_tx.send(Ok((handle, api)));
				let daemon: Daemon<NativeRuntime> = Daemon::new(runtime.clone());
				let shutdown_token = daemon.shutdown_token.clone();
				let daemon_task = tokio::spawn(async move {
					let mut daemon = daemon;
					daemon.run_foreground().await
				});
				match rx.recv().await {
					Some(DaemonCommand::Stop) => {
						tracing::info!("daemon stop requested");
						shutdown_token.cancel();
						match daemon_task.await {
							Ok(Ok(())) => {
								tracing::info!("daemon stopped cleanly");
							}
							Ok(Err(error)) => {
								tracing::error!(%error, "daemon exited with error");
							}
							Err(error) => {
								tracing::error!(%error, "daemon task panicked");
							}
						}
					}

					None => {
						tracing::info!("daemon command channel closed");

						shutdown_token.cancel();
						let _ = daemon_task.await;
					}
				}
			});
		});
	}
	fn spawn_signal_handler(&mut self, proxy: EventLoopProxy<AppEvent>) {
		std::thread::spawn(move || {
			tracing::info!("SIGNAL: thread started");
			let mut signals = Signals::new([SIGINT]).expect("failed to register SIGINT");
			tracing::info!("SIGNAL: waiting");
			if signals.forever().next().is_some() {
				tracing::info!("SIGNAL: received");
				let _ = proxy.send_event(AppEvent::Shutdown);
				tracing::info!("SIGNAL: event sent");
			}
			tracing::info!("SIGNAL: thread exiting");
		});
	}
	fn shutdown(&mut self) {
		tracing::info!(">>> shutting down runtime");
		self.clock_running.store(false, Ordering::Relaxed);
		self.hotkey_manager.shutdown();
		match self.daemon_tx.try_send(DaemonCommand::Stop) {
			Ok(()) => tracing::info!(">>> daemon stop sent"),
			Err(error) => tracing::error!(%error, ">>> daemon stop failed"),
		}
		tracing::info!(">>> runtime shutdown complete");
	}
}
impl ApplicationHandler<AppEvent> for NativeApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.menu_bar.is_none() {
			let menu = Self::menu_bar(true);
			menu.init_for_nsapp();
			self.menu_bar = Some(menu);
		}
		if self.windows.is_empty() {
			self.open_window(event_loop, crate::INITIAL_WINDOW);
		}
		if self.tray.is_none() {
			let (menu, tray) = match Self::bootstrap() {
				Ok(value) => value,
				Err(error) => {
					tracing::error!(%error, "failed to bootstrap tray");
					return;
				}
			};
			self.menu = Some(menu);
			self.tray = Some(tray);
			tracing::debug!("🔥 main tray initialized");
		}
		if self.scroll_tray.is_none() {
			match TrayIconBuilder::new()
				.with_icon(scroll_tray_icon())
				.with_tooltip("Estate Scroll Controller")
				.build()
			{
				Ok(tray) => {
					self.scroll_tray = Some(tray);
					tracing::debug!("🔥 scroll tray initialized");
				}
				Err(error) => {
					tracing::error!(%error, "failed to create scroll tray");
				}
			}
		}
	}
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if let Some(app) = &mut self.app {
			app.update();
			// self
			// 	.app
			// 	.as_mut()
			// 	.expect("app must be initialized before the event loop starts")
			// 	.update();
		}
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			self.handle_event(event, event_loop);
		}
	}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(window) = self
			.windows
			.iter_mut()
			.find(|window| window.window.instance.id() == window_id)
		else {
			return;
		};
		let response = window
			.window
			.gui_state
			.on_window_event(&window.window.instance, &event);
		if response.repaint {
			window.window.instance.request_redraw();
		}
		match event {
			WindowEvent::CloseRequested => {
				tracing::info!("🛑 Window close requested for id: {:?}", window_id);
				self
					.windows
					.retain(|window| window.window.instance.id() != window_id);
				return;
			}
			WindowEvent::RedrawRequested => {
				if window.window.occluded {
					return;
				}
				let menu = {
					let event_rx = self.engine.runtime.subscribe();
					if let Some(app) = self.app.as_mut() {
						let event_rx = self.engine.runtime.subscribe();

						let mut ctx = AppContext {
							app,
							input: VeInputState::default(),
							event_rx,
							last_revision: 0,
						};

						if let Err(e) = window.window.draw(&mut ctx) {
							tracing::error!("DEV >>> draw failed: {e:#}");
						}
					}
				};
			}
			WindowEvent::Focused(true) => {
				window.window.instance.request_redraw();
			}
			WindowEvent::Occluded(occluded) => {
				window.window.occluded = occluded;
				if !occluded {
					window.window.instance.request_redraw();
				}
			}
			WindowEvent::Resized(size) => {
				if size.width == 0 || size.height == 0 {
					return;
				}
				window.window.config.width = size.width;
				window.window.config.height = size.height;
				window
					.window
					.surface
					.configure(&window.window.device, &window.window.config);
				window.window.needs_resize = false;
				window.window.instance.request_redraw();
			}
			_ => {}
		}
	}
	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		match event {
			AppEvent::RuntimeEvent => {
				tracing::info!("🔄 Runtime event");
				if let Some(app) = &mut self.app {
					app.update();
				}
			}

			AppEvent::Navigate(view) => {
				self
					.engine
					.runtime
					.emit(e::Event::app(e::EventKind::Navigate(view)));
				if let Some(app) = &mut self.app {
					app.update();
				}
				for window in &mut self.windows {
					window.window.instance.request_redraw();
				}
			}
			AppEvent::Shutdown => {
				tracing::info!(">>> shutdown event received");
				self.shutdown();
				event_loop.exit();
				tracing::info!(">>> event_loop.exit() called");
			}
			AppEvent::CursorPosition { x, y } => {
				// let text = format!("↖ {:.0}  {:.0}", x, y);
				// let text = format!("← {:.0}  {:.0}", x, y);
				// let text = format!("→ {:.0}  {:.0}", x, y);
				// let text = format!("↑ {:.0}  {:.0}", x, y);
				// let text = format!("● {:.0}, {:.0}", x, y);
				// let text = format!("◉ {:.0}, {:.0}", x, y);
				let text = format!("⌖ {:.0}, {:.0}", x, y);
				// let text = format!("🟢 {:.0}, {:.0}", x, y);
				// let text = format!("🔵 {:.0}, {:.0}", x, y);
				// let text = format!("🟡 {:.0}, {:.0}", x, y);
				// let text = format!("🔴 {:.0}, {:.0}", x, y);
				// let region = if x < 960.0 { "← LEFT" } else { "RIGHT →" };
				if let Some(tray) = &self.scroll_tray {
					let _ = tray.set_title(Some(text));
				}
			}
			AppEvent::TickClock(text) => {
				if let Some(tray) = &self.tray {
					let _ = tray.set_title(Some(text));
				}
				self
					.app
					.as_mut()
					.expect("app must be initialized before the event loop starts")
					.update();
				self.sync_views();
			}
			AppEvent::ModifiersChanged {
				alt,
				command,
				ctrl,
				shift,
			} => {}
			_ => {}
		}
	}
}
impl NativeApp {
	fn handle_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		let Some(menu) = self.menu.as_ref() else {
			return;
		};
		let id = event.id();
		if id == menu.quit.id() {
			tracing::debug!(">>> tray quit requested");
			self.shutdown();
			event_loop.exit();
			tracing::debug!(">>> event_loop.exit() called");
		} else if id == menu.dev.id() {
			self.open_window(event_loop, WindowType::Dashboard);
		} else if id == menu.telemetry.id() {
			self.open_window(event_loop, WindowType::TelemetryInspector);
		} else if id == menu.task_manager.id() {
			self.open_window(event_loop, WindowType::TaskManager);
		} else if id == menu.new_task.id() {
			self.new_task();
		} else if id == menu.list_tasks.id() {
			self.show_tasks();
		} else if id == menu.clear_tasks.id() {
			self.clear_tasks();
		} else if id == menu.problem_screen.id() {
			tracing::info!("🧭 Menu → ProblemsScreen");
			self
				.engine
				.runtime
				.emit(e::Event::app(e::EventKind::Navigate(
					ViewType::ProblemsScreen,
				)));
			self.open_window(event_loop, WindowType::ProblemsScreen);
		}
	}
	fn bootstrap() -> Result<(TrayMenu, TrayIcon)> {
		bootstrap()
	}
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		if self.window_by_type(kind).is_some() {
			return;
		}
		if let Some(app) = self.app.as_mut() {
			let api = app.api();
			match Window::new(event_loop, self.view_type.clone(), api) {
				Ok(window) => {
					window.instance.set_title("Hi there Loi");
					self.windows.push(AppWindow {
						kind: WindowType::from(WindowType::Markdown),
						window,
					});
				}

				Err(error) => {
					tracing::error!(
							?self.view_type,
							%error,
							"failed to create window"
					);
				}
			}
		}
	}
}
impl NativeApp {
	fn new_task(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			}));
	}
	fn show_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_list".into(),
			}));
	}
	fn clear_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	#[tracing::instrument(
		target = "estate::discovery",
		name = "scan_workspace",
		skip(self),
		fields(flow_id = %Uuid::new_v4())
	)]
	async fn _scan_workspace(&mut self, path: &Path) -> Result<()> {
		tracing::info!("starting workspace scan");
		self._discover(path).await?;
		tracing::debug!("discovery complete");
		self._analyze().await?;
		tracing::debug!("analysis complete");
		self._build_graph().await?;
		tracing::info!("workspace scan complete");
		Ok(())
	}
	#[tracing::instrument(target = "estate::discovery", skip(self, path))]
	async fn _discover(&mut self, path: &Path) -> Result<()> {
		tracing::debug!(path = %path.display(), "discovering workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::analysis", skip(self))]
	async fn _analyze(&mut self) -> Result<()> {
		tracing::debug!("analyzing workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::graph", skip(self))]
	async fn _build_graph(&mut self) -> Result<()> {
		tracing::debug!("building semantic graph");
		Ok(())
	}
}

impl NativeApp {
	fn sync_views(&mut self) {
		for window in &mut self.windows {
			if let Some(app) = self.app.as_mut() {
				let view_type = app.view();
				let api = app.api();
				window.window.sync_view(view_type, api);
				window.window.instance.request_redraw();
			}
		}
	}
	fn set_view(&mut self, view_type: ViewType) {
		self.view_type = view_type;
	}
	fn change_view(&self, view_type: ViewType) -> Ve<NativeRuntime> {
		let app = self
			.app
			.as_ref()
			.expect("app must be initialized before changing views");
		match view_type {
			ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::MARKDOWN)),
			ViewType::TaskManager => Ve::new(TaskManager::new()),
			ViewType::WaterfallChart => Ve::new(OracleView::new()),
			ViewType::ProblemsScreen => Ve::new(ProblemsScreen::new()),
			_ => app.default_view(),
		}
	}
	fn set_menu_bar(&mut self, new_menu: Menu) {
		self.menu_bar = Some(new_menu);
		tracing::info!("setting menu bar");
		if let Some(menu) = &self.menu_bar {
			menu.init_for_nsapp();
		}
	}
	fn menu_bar(has_document: bool) -> Menu {
		let menu = Menu::new();
		menu.append(&Self::file_menu(has_document)).unwrap();
		menu.append(&Self::edit_menu()).unwrap();

		menu
	}
	fn file_menu(has_document: bool) -> Submenu {
		let menu = Submenu::new("File", true);
		menu.append(&MenuItem::new("New", true, None));
		menu.append(&MenuItem::new("Open…", true, None));
		menu.append(&PredefinedMenuItem::separator());
		menu.append(&MenuItem::new("Close", has_document, None));

		menu
	}
	fn edit_menu() -> Submenu {
		let menu = Submenu::new("Edit", true);
		menu.append(&PredefinedMenuItem::undo(None));
		menu.append(&PredefinedMenuItem::redo(None));
		menu.append(&PredefinedMenuItem::separator());
		menu.append(&PredefinedMenuItem::cut(None));
		menu.append(&PredefinedMenuItem::copy(None));
		menu.append(&PredefinedMenuItem::paste(None));
		menu
	}
	fn create_menu() -> Menu {
		let menu = Menu::new();
		let file = Submenu::new("File", true);
		file.append_items(&[
			&MenuItem::new("New", true, None),
			&MenuItem::new("Open…", true, None),
			&PredefinedMenuItem::separator(),
			&MenuItem::new("Close", true, None),
		]);
		let edit = Submenu::new("Edit", true);
		edit.append_items(&[
			&PredefinedMenuItem::undo(None),
			&PredefinedMenuItem::redo(None),
			&PredefinedMenuItem::separator(),
			&PredefinedMenuItem::cut(None),
			&PredefinedMenuItem::copy(None),
			&PredefinedMenuItem::paste(None),
		]);
		let view = Submenu::new("View", true);
		view.append_items(&[
			&MenuItem::new("Toggle Sidebar", true, None),
			&MenuItem::new("Fullscreen", true, None),
		]);
		menu.append_items(&[&file, &edit, &view]);
		menu
	}
}
