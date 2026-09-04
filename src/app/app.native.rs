use crate::{app::prelude::*, r#trait::Context};

use crate::{
	AppEvent, DaemonCommand, NativeExecutor,
	api::NativeApiClient,
	app::{Runtime, model::EstateEngine},
	e,
	native::router,
	spawn_global_cursor_daemon,
};

use tokio::runtime::Handle;
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
	pub app: AppRuntime<NativeRuntime, NativeExecutor>,
	pub host: NativeHost,
	pub runtime: NativeRuntime,
	// Receiver channel for process/daemon
	pub daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
	// Sender channel for process/daemon
	pub daemon_tx: mpsc::Sender<DaemonCommand>,
	pub hotkey_manager: GlobalHotkeys,
	pub is_clocking: Arc<AtomicBool>,
	pub menu: Option<TrayMenu>,
	pub menu_bar: Option<Menu>,
	pub monitor: NativeMonitor,
	pub tokio: tokio::runtime::Runtime,
	pub tray_clock: Option<TrayIcon>,
	pub tray_cursor: Option<TrayIcon>,
	pub windows: Vec<AppWindow>,
}

impl Context for NativeApp {
	type Host = NativeHost;
	type Runtime = NativeRuntime;
	type Args = Cli;
	fn new() -> Result<Self> {
		NativeApp::new()
	}
	fn host(&self) -> &Self::Host {
		&self.host
	}
	fn runtime(&self) -> &Self::Runtime {
		&self.runtime
	}
	fn run(&mut self, cli: Self::Args) -> Result<()> {
		NativeApp::run(self, cli)
	}
	fn foo(&self, args: String) -> Result<()> {
		NativeApp::foo(&self, args)
	}
	fn bar(&self, args: String) -> Result<()> {
		NativeApp::bar(&self, args)
	}
}

impl NativeApp {
	pub fn new() -> Result<Self> {
		let tokio = tokio::runtime::Runtime::new()?;
		let handle = tokio.handle().clone();
		// Runtime owns all runtime infrastructure:
		// services, executor, state, event bus, session, etc.
		let runtime = tokio.block_on(NativeRuntime::new(handle.clone()))?;

		// Engine owns the domain/application engine and uses Runtime.
		let engine = EstateEngine::new(runtime.clone())?;

		let app = AppRuntime::new(engine.clone(), runtime.executor.clone());
		app.start();
		app.start_services();

		let host = NativeHost::new();
		let (daemon_tx, daemon_rx) = mpsc::channel(100);
		Ok(Self {
			app,
			host,
			runtime,
			is_clocking: Arc::new(AtomicBool::new(true)),
			daemon_rx: Some(daemon_rx),
			daemon_tx,
			hotkey_manager: GlobalHotkeys::new().unwrap(),
			menu: None,
			menu_bar: None,
			monitor: NativeMonitor::new()?,
			tokio,
			tray_clock: None,
			tray_cursor: None,
			windows: vec![],
		})
	}
	fn foo(&self, cli: String) -> Result<()> {
		todo!("run")
	}
	fn bar(&self, cli: String) -> Result<()> {
		todo!("run")
	}
	// Inner NativeApp
	fn runtime_old(&self) -> Arc<NativeRuntime> {
		// [Flexibility]
		// Decide later if theres any bad things that can happen from enabling app runtime
		// access.
		Arc::clone(&self.app.engine.runtime)
	}
	pub fn handle(&self) -> tokio::runtime::Handle {
		self.tokio.handle().clone()
	}
}
impl NativeApp {
	pub fn run(&mut self, cli: Cli) -> Result<()> {
		tracing::debug!(">>> NativeApp::run entered");
		let result = match cli.command {
			None | Some(Command::Start { .. }) | Some(Command::Tray) => self.start_runtime(),
			Some(_) => self.tokio.block_on(async {
				let ctx = cli::context::Context::new();
				router::execute(cli, ctx, self.app.engine.clone()).await
			}),
		};
		tracing::debug!(">>> NativeApp::run returning");
		result
	}
	fn start_runtime(&mut self) -> Result<()> {
		// // App
		// self.engine.runtime.spawn(future);       // ✅
		// // NativeRuntime
		// self.handle.spawn(future);               // ✅
		// // NativeApp
		// self.tokio = Runtime::new()?;             // ✅
		// // App
		// tokio::spawn(future);                     // ❌
		// // App
		// tokio::runtime::Handle::current();        // ❌
		// // AppContext
		// tokio::runtime::Handle;
		// 1. Create Tokio first.
		// AppRuntime
		self.runtime.start_services();
		// EstateEngineRuntime
		self.runtime().start_services();
		let daemon_rx = self.daemon_rx.take().expect("daemon already started");
		let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<Arc<NativeApiClient>>>(1);
		self.spawn_daemon(daemon_rx, ready_tx);
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
		self.runtime_old().attach_event_proxy(proxy);
		self
			.runtime_old()
			.emit(e::Event::app(e::Klass::SessionStart));
		event_loop.run_app(self)?;
		tracing::info!(">>> NativeApp::start_runtime returning");
		Ok(())
	}
	fn spawn_clock(&mut self, proxy: EventLoopProxy<AppEvent>) {
		// EventLoopProxy wakes the winit event loop from this background task.
		// It is needed when the event must be processed by winit itself rather
		// than only through the application's Runtime event bus.
		println!("Spawn Clock Start");

		// Shared shutdown flag. The clock exits when NativeApp shuts down.
		let running = Arc::clone(&self.is_clocking);

		// Tokio handle for spawning work directly onto the native async runtime.
		// This is the native executor behind AppRuntime's Executor abstraction.
		let handle: Handle = self.handle();

		// Runtime access for emitting application events from the background task.
		// let runtime: NativeRuntime = self.runtime.clone();

		// Arc<NativeRuntime> gives the spawned task shared ownership of the
		// runtime so it can outlive this call and safely access runtime services.
		let task_runtime: Arc<NativeRuntime> = self.runtime_old().clone();
		let executor = task_runtime.executor.clone();

		executor.spawn(async move {
			println!("Handle triggered executor");
			let views = [
				ViewType::ProblemScreen,
				ViewType::DashboardScreen,
				ViewType::MarkdownView,
				ViewType::ProblemScreen,
				ViewType::WaterfallScreen,
				ViewType::ProblemScreen,
				ViewType::TaskManagerScreen,
				ViewType::ProblemsScreen,
			];

			let mut current_time = 10;
			let mut view_index = 0;

			while running.load(Ordering::Relaxed) {
				println!("Native App Tick");
				let _ = proxy.send_event(AppEvent::TickClock(format!(" {}s", current_time)));

				tracing::info!("NativeApp clock {}", current_time);

				if current_time == 0 {
					current_time = 10;

					view_index = (view_index + 1) % views.len();
					let view = views[view_index];

					tracing::info!("⏩ Native App Clock navigation → {:?}", view);

					task_runtime.emit(e::Event::app(e::Klass::Navigate(view)));

					let _ = proxy.send_event(AppEvent::RuntimeEvent);
				} else {
					current_time -= 1;
				}
				// IMPORTANT: this must be an async sleep.
				// std::thread::sleep() blocks the Tokio worker thread and can
				// prevent other async tasks from running.
				task_runtime.sleep(std::time::Duration::from_secs(1)).await;
			}
		});
		// std::thread::spawn(move || {
		// 	println!("Handle triggered executor");
		// 	let views = [
		// 		ViewType::ProblemScreen,
		// 		ViewType::DashboardScreen,
		// 		ViewType::MarkdownView,
		// 		ViewType::ProblemScreen,
		// 		ViewType::WaterfallScreen,
		// 		ViewType::ProblemScreen,
		// 		ViewType::TaskManagerScreen,
		// 		ViewType::ProblemsScreen,
		// 	];
		// 	let mut current_time = 10;
		// 	let mut view_index = 0;
		// 	while running.load(Ordering::Relaxed) {
		// 		// runtime.emit(e::Event::app(e::Klass::TickClock(format!(
		// 		// 	" {}s",
		// 		// 	current_time
		// 		// ))));
		// 		let _ = proxy.send_event(AppEvent::TickClock(format!(" {}s", current_time)));
		// 		tracing::info!("NativeApp clock {}", current_time);
		// 		if current_time == 0 {
		// 			current_time = 10;
		// 			view_index = (view_index + 1) % views.len();
		// 			let view = views[view_index];
		// 			tracing::info!("⏩ Native App Clock navigation → {:?}", view);
		// 			runtime.emit(e::Event::app(e::Klass::Navigate(view)));
		// 			let _ = proxy.send_event(AppEvent::RuntimeEvent);
		// 		} else {
		// 			current_time -= 1;
		// 		}
		// 		std::thread::sleep(std::time::Duration::from_secs(1));
		// 	}
		// });
	}
	fn spawn_cursor_daemon(&mut self, proxy: EventLoopProxy<AppEvent>) {
		spawn_global_cursor_daemon(proxy)
	}
	fn spawn_global_hotkey_daemon(&mut self) -> Result<()> {
		self.hotkey_manager.start();
		Ok(())
	}
	fn spawn_daemon(
		&mut self,
		mut rx: mpsc::Receiver<DaemonCommand>,
		ready_tx: std::sync::mpsc::SyncSender<Result<Arc<NativeApiClient>>>,
	) {
		let runtime = self.runtime_old();
		self.handle().spawn(async move {
			runtime.start_dispatcher();
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
	fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!(">>> shutting down runtime");
		let snapshot = {
			let runtime = self.runtime();
			let mut state = runtime.state.write();
			state.session.end();
			state.clone()
		};
		self.runtime().save(&snapshot);
		self.is_clocking.store(false, Ordering::Relaxed);
		self.hotkey_manager.shutdown();
		match self.daemon_tx.try_send(DaemonCommand::Stop) {
			Ok(()) => tracing::info!(">>> daemon stop sent"),
			Err(error) => tracing::error!(%error, ">>> daemon stop failed"),
		}
		tracing::info!(">>> runtime shutdown complete");
		event_loop.exit();
	}
}
impl NativeApp {
	fn bootstrap() -> Result<(TrayMenu, TrayIcon)> {
		bootstrap()
	}
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn handle_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		let Some(menu) = self.menu.as_ref() else {
			return;
		};
		let id = event.id();
		if id == menu.quit.id() {
			tracing::debug!(">>> tray quit requested");
			self.shutdown(event_loop);
			tracing::debug!(">>> event_loop.exit() called");
		} else if id == menu.dev.id() {
			self.open_window(event_loop, WindowType::DashboardScreen);
		} else if id == menu.oracle.id() {
			self.open_window(event_loop, WindowType::OracleScreen);
		} else if id == menu.task_manager.id() {
			self.open_window(event_loop, WindowType::TaskManagerScreen);
		} else if id == menu.new_task.id() {
			self.new_task();
		} else if id == menu.list_tasks.id() {
			self.show_tasks();
		} else if id == menu.clear_tasks.id() {
			self.clear_tasks();
		} else if id == menu.problem_screen.id() {
			tracing::info!("🧭 Menu → ProblemsScreen");
			self
				.runtime()
				.emit(e::Event::app(e::Klass::Navigate(ViewType::ProblemsScreen)));
			self.open_window(event_loop, WindowType::ProblemsScreen);
		}
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		tracing::info!(" open window start");
		if self.window_by_type(kind).is_some() {
			return;
		}
		match Window::new(event_loop, self.app.view()) {
			Ok(window) => {
				tracing::info!(" open window end, new window");
				window.instance.set_title(self.app.view().name().into());
				self.windows.push(AppWindow {
					kind,
					view: self.app.view(),
					window,
				});
			}
			Err(error) => {
				tracing::error!("failed to create window: {error}");
			}
		}
	}
}
impl NativeApp {
	fn new_task(&mut self) {
		self.runtime().emit(e::Event::app(e::Klass::TaskRequested {
			request: TaskRequest::Create(TaskKind::SyncBookmarks),
		}));
	}
	fn show_tasks(&mut self) {
		self
			.runtime()
			.emit(e::Event::app(e::Klass::CommandExecuted {
				command: "task_list".into(),
			}));
	}
	fn clear_tasks(&mut self) {
		self
			.runtime()
			.emit(e::Event::app(e::Klass::CommandExecuted {
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
			// println!("NativeApp {:?}", self.runtime.view);
			tracing::debug!("sync views {:?}", self.app.view.name());
			// window
			// 	.window
			// 	.sync_view(self.runtime.view, self.runtime.api.clone());
			window.window.instance.request_redraw();
			window.window.instance.set_title(self.app.view.name());
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
impl ApplicationHandler<AppEvent> for NativeApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.menu_bar.is_none() {
			let menu = Self::menu_bar(true);
			menu.init_for_nsapp();
			self.menu_bar = Some(menu);
		}
		if self.windows.is_empty() {
			self.open_window(event_loop, crate::START_WINDOW);
		}
		if self.tray_clock.is_none() {
			let (menu, tray) = match Self::bootstrap() {
				Ok(value) => value,
				Err(error) => {
					tracing::error!(%error, "failed to bootstrap tray");
					return;
				}
			};
			self.menu = Some(menu);
			self.tray_clock = Some(tray);
			tracing::debug!("🔥 main tray initialized");
		}
		if self.tray_cursor.is_none() {
			match TrayIconBuilder::new()
				.with_icon(scroll_tray_icon())
				.with_tooltip("Estate Scroll Controller")
				.build()
			{
				Ok(tray) => {
					self.tray_cursor = Some(tray);
					tracing::debug!("🔥 scroll tray initialized");
				}
				Err(error) => {
					tracing::error!(%error, "failed to create scroll tray");
				}
			}
		}
	}
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		self.app.update();
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
					let event_rx = self.app.engine.runtime().subscribe();
					let mut ctx = AppContext {
						app: &mut self.app,
						input: IOState::default(),
						event_rx,
						last_revision: 0,
					};

					if let Err(e) = window.window.draw(&mut ctx) {
						tracing::error!("DEV >>> draw failed: {e:#}");
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
				self.app.update();
				self.sync_views();
			}
			AppEvent::Navigate(view) => {
				self.runtime().emit(e::Event::app(e::Klass::Navigate(view)));
				self.app.update();
				self.sync_views();
			}
			AppEvent::Shutdown => {
				tracing::info!(">>> shutdown event received");
				self.shutdown(event_loop);

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
				if let Some(tray) = &self.tray_cursor {
					let _ = tray.set_title(Some(text));
				}
			}
			AppEvent::TickClock(text) => {
				if let Some(tray) = &self.tray_clock {
					let _ = tray.set_title(Some(text));
				}
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

#[derive(Default)]
pub struct NativeHost {
	window: NativeWindow,
	storage: NativeStorage,
	clock: NativeClock,
}

impl NativeHost {
	fn new() -> Self {
		Self::default()
	}
}
#[derive(Debug, Default, Clone)]
pub struct NativeWindow;
#[derive(Debug, Default, Clone)]
pub struct NativeStorage;
#[derive(Debug, Default, Clone)]
pub struct NativeClock;

impl Clock for NativeClock {
	fn now(&self) -> std::time::Instant {
		todo!("now")
	}
}

impl Host for NativeHost {
	type Window = NativeWindow;
	type Storage = NativeStorage;
	type Clock = NativeClock;
	fn window(&self) -> &Self::Window {
		&self.window
	}
	fn storage(&self) -> &Self::Storage {
		&self.storage
	}
	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
}

#[derive(Debug, Clone)]
pub struct NativeServices {
	persistance: NativePersistance,
	network: NativeNetwork,
	clock: NativeClock,
	api: NativeApiClient,
}

impl NativeServices {
	pub async fn connect() -> anyhow::Result<Self> {
		let api = NativeApiClient::connect().await?;
		Ok(Self {
			persistance: NativePersistance::default(),
			network: NativeNetwork::default(),
			clock: NativeClock::default(),
			api,
		})
	}
}

impl Services for NativeServices {
	type Persistence = NativePersistance;
	type Network = NativeNetwork;
	type Clock = NativeClock;
	type Client = NativeApiClient;

	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn clock(&self) -> &Self::Clock {
		todo!("");
	}
	fn api(&self) -> &Self::Client {
		&self.api
	}
}

#[derive(Debug, Default, Clone)]
pub struct NativePersistance;

impl Persistence for NativePersistance {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
		todo!("")
	}
	fn save(&self, key: &str, value: &[u8]) -> Result<()> {
		todo!("")
	}
}
#[derive(Debug, Default, Clone)]
pub struct NativeNetwork;

impl Network for NativeNetwork {
	fn is_available(&self) -> bool {
		todo!("")
	}
}

impl Drop for NativeApp {
	fn drop(&mut self) {
		tracing::info!("💀 NativeApp DROPPED");
	}
}
