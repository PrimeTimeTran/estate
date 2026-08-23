use crate::{prelude::*, router, ve};

use egui::{Context as EguiContext, PopupAnchor::Position, TexturesDelta, Ui};
use egui_wgpu::{
	Renderer, SurfaceConfig,
	wgpu::{self, hal::InstanceDescriptor},
};
use egui_winit::State as EguiState;
use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use tracing::instrument::WithSubscriber;
use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, Submenu},
};
use wgpu::{Adapter, Device, ExperimentalFeatures, SurfaceColorSpace, SurfaceConfiguration};
use winit::{
	application::ApplicationHandler,
	dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::{self as EguiWindow, WindowAttributes, WindowId, WindowLevel},
};
/// Top-level application state.
///
/// Owns the system tray integration, application context, Estate engine,
/// daemon communication channel, and optional development window.
pub struct App {
	/// The system tray icon owned by the application.
	tray: Option<TrayIcon>,
	/// The system tray menu and its associated menu items.
	menu: Option<TrayMenu>,
	/// Shared application context and runtime state.
	context: Context,
	/// The Estate engine responsible for the application's core functionality.
	engine: EstateEngine,
	/// Channel used to send commands to the Estate daemon, when available.
	// daemon_tx: Option<mpsc::Sender<DaemonCommand>>,
	/// The development window, when it has been opened.
	// daemon: Option<DaemonHandle>,
	daemon: Option<Daemon>,
	daemon_tx: mpsc::Sender<DaemonCommand>,
	/// Use an `Option` if **another thread or runtime task needs to take permanent ownership** of the receiver to run a loop, and you won't be polling it directly inside `Oracle`'s `draw()` method.
	/// * **How it works:** You create the channel inside `Oracle::new()`, keep the `Sender` inside `Oracle` (or pass it to your file watcher), and **take** the `Receiver` out once via `.take()` to hand it off to a worker thread or your application's main event pump.
	/// * **Why `Option`?** Because in Rust, you cannot move a field out of a struct by value if the struct itself is behind a mutable reference or doesn't implement `Default`. `.take()` replaces the field with `None` so you can move the `Receiver` out cleanly.
	/// ### 2. When to NOT use `Option` (Direct Polling)
	/// If you are polling the receiver *directly inside* `Oracle`'s own methods (like calling `self.rx.try_recv()` inside your `draw()` frame tick), **you do not need an `Option**`.
	/// * **How it works:** The receiver stays embedded in `Oracle`, and you just access it mutably via `&mut self.rx`.
	/// * **Why skip `Option`?** It avoids unnecessary `.unwrap()` calls, keeps the struct fields clean, and prevents runtime panics if something tries to access a receiver that has already been taken.
	/// ### Summary Rule of Thumb:
	/// * **Poll inside `Oracle` methods (like your current setup):** Keep it as a plain `mpsc::Receiver<T>` (no `Option`).
	/// * **Handoff to an external worker loop/thread:** Use `Option<mpsc::Receiver<T>>` so you can `.take()` it once upon startup.
	daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
	window: Option<Window>,
	hotkey_manager: Option<GlobalHotKeyManager>,
	pub windows: Vec<Window>,
	pub dashboard_window: Option<Window>,
	pub telemetry_window: Option<Window>,
	pub clock: Option<Window>,
	clock_tray: Option<TrayIcon>,
	scroll_tray: Option<TrayIcon>,
}
use std::sync::atomic::{AtomicBool, Ordering};
static HOTKEY_INITIALIZED: AtomicBool = AtomicBool::new(false);
impl App {
	pub fn new() -> anyhow::Result<Self> {
		let engine = EstateEngine::new()?;
		let context = Context::default();
		let (daemon_tx, daemon_rx) = mpsc::channel(100);

		Ok(Self {
			hotkey_manager: None,
			context,
			clock: None,
			daemon: None,
			daemon_rx: Some(daemon_rx),
			daemon_tx,
			engine,
			menu: None,
			tray: None,
			window: None,
			windows: vec![],
			dashboard_window: None,
			telemetry_window: None,
			clock_tray: None,
			scroll_tray: None,
		})
	}
	pub fn run(&mut self, cli: Cli) -> anyhow::Result<()> {
		match cli.command {
			None | Some(Command::Start { .. }) | Some(Command::Tray) => self.start_runtime(),
			Some(_) => {
				let runtime = tokio::runtime::Runtime::new()?;
				runtime.block_on(async {
					let ctx = cli::context::Context::new();
					router::execute(cli, ctx, self.engine.clone()).await
				})
			}
		}
	}
	fn start_runtime(&mut self) -> anyhow::Result<()> {
		tracing::info!(">>> start_runtime: entering");
		self.register_global_hotkeys()?;
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.with_activation_policy(ActivationPolicy::Accessory)
			.build()?;
		let proxy = event_loop.create_proxy();
		start_global_scroll_daemon(proxy.clone());
		self.spawn_signal_handler(proxy.clone());
		self.span_clock(proxy);
		tracing::info!(">>> start_runtime: entering event loop");
		event_loop.run_app(self)?;
		tracing::info!(">>> start_runtime: event loop returned");
		let rx = self
			.daemon_rx
			.take()
			.expect("daemon receiver already consumed");
		let engine = self.engine.clone();
		Self::spawn_daemon(engine, rx);
		Ok(())
	}
	fn spawn_daemon(engine: EstateEngine, mut rx: mpsc::Receiver<DaemonCommand>) {
		std::thread::spawn(move || {
			let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
			runtime.block_on(async move {
				let mut daemon = Daemon::new(engine);
				tokio::select! {
					result = daemon.run_foreground() => {
						if let Err(error) = result {
							tracing::error!(%error, "daemon exited");
						}
					}

					command = rx.recv() => {
						match command {
							Some(DaemonCommand::Stop) => {
								if let Err(error) = daemon.shutdown().await {
									tracing::error!(
										%error,
										"daemon shutdown failed"
									);
								}
							}

							None => {
								tracing::info!("daemon command channel closed");
								let _ = daemon.shutdown().await;
							}
						}
					}
				}
			});
		});
	}
	fn spawn_signal_handler(&mut self, proxy: EventLoopProxy<AppEvent>) {
		std::thread::spawn(move || {
			tracing::info!("Ctrl+C handler started");
			let runtime = tokio::runtime::Runtime::new().expect("failed to create Tokio runtime");
			runtime.block_on(async move {
				tracing::info!("Waiting for Ctrl+C...");

				match tokio::signal::ctrl_c().await {
					Ok(()) => {
						tracing::info!("🛑 Ctrl+C received");

						if let Err(error) = proxy.send_event(AppEvent::Shutdown) {
							tracing::error!(
								%error,
								"failed to send Shutdown event"
							);
						}
					}

					Err(error) => {
						tracing::error!(
							%error,
							"failed to listen for Ctrl+C"
						);
					}
				}
			});

			tracing::info!("Ctrl+C handler exiting");
		});
	}
	fn span_clock(&mut self, proxy: EventLoopProxy<AppEvent>) {
		let proxy_ticker = proxy.clone();
		std::thread::spawn(move || {
			let mut current_time = 30;
			loop {
				let text = format!(" {}s", current_time);
				let _ = proxy_ticker.send_event(AppEvent::TickClock(text));
				if current_time == 0 {
					current_time = 30;
				} else {
					current_time -= 1;
				}
				std::thread::sleep(std::time::Duration::from_secs(1));
			}
		});
	}
	fn register_global_hotkeys(&mut self) -> anyhow::Result<()> {
		let manager = GlobalHotKeyManager::new()?;

		let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);

		let hotkey_id = hotkey.id();

		manager.register(hotkey)?;

		self.hotkey_manager = Some(manager);

		std::thread::spawn(move || {
			let receiver = GlobalHotKeyEvent::receiver();

			while let Ok(event) = receiver.recv() {
				if event.id == hotkey_id && event.state == global_hotkey::HotKeyState::Pressed {
					move_cursor_to(ScreenPosition::Left);
				}
			}
		});

		Ok(())
	}
	fn bootstrap() -> anyhow::Result<(TrayMenu, TrayIcon)> {
		let menu = Menu::new();
		let clock_item = MenuItem::new("Clock: 30s", true, None);
		let scroll_item = MenuItem::new("Scroll: Idle", true, None);
		let status = MenuItem::new("● Estate Daemon Running", false, None);
		let dev = MenuItem::new("Open Dashboard", true, None);
		let telemetry = MenuItem::new("Open Telemetry Inspector", true, None);
		let task_manager = MenuItem::new("Open Task Manager", true, None);
		let new_task = MenuItem::new("New Task", true, None);
		let list_tasks = MenuItem::new("List Tasks", true, None);
		let clear_tasks = MenuItem::new("Clear Tasks", true, None);
		let tasks = Submenu::new("Tasks", true);
		tasks.append(&new_task)?;
		tasks.append(&list_tasks)?;
		tasks.append(&clear_tasks)?;
		let quit = MenuItem::new("Quit", true, None);
		menu.append(&clock_item)?;
		menu.append(&scroll_item)?;
		menu.append(&status)?;
		menu.append(&dev)?;
		menu.append(&telemetry)?;
		menu.append(&task_manager)?;
		menu.append(&tasks)?;
		menu.append(&quit)?;
		let tray = TrayIconBuilder::new()
			.with_icon(Self::tray_icon())
			.with_menu(Box::new(menu))
			.with_tooltip("Estate Daemon — Running")
			.build()
			.map_err(|e| anyhow::anyhow!("failed to create tray icon: {e}"))?;
		let clock_tray = Some(
			TrayIconBuilder::new()
				.with_icon(Self::tray_icon())
				.with_tooltip("Estate Clock")
				.build()?,
		);
		let scroll_tray = Some(
			TrayIconBuilder::new()
				.with_icon(Self::tray_icon())
				.with_tooltip("Scroll Redirect")
				.build()?,
		);
		Ok((
			TrayMenu {
				clear_tasks,
				dev,
				list_tasks,
				new_task,
				quit,
				status,
				// scroll_item,
				tasks,
				telemetry,
				task_manager,
			},
			tray,
		))
	}
	fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("🛑 Shutting down application");

		let _ = self.daemon_tx.try_send(DaemonCommand::Stop);

		tracing::info!("🛑 Calling event_loop.exit()");
		event_loop.exit();
		tracing::info!("🛑 event_loop.exit() returned");
	}
	fn handle_menu_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		let Some(menu) = self.menu.as_ref() else {
			return;
		};
		let id = event.id();
		if id == menu.quit.id() {
			let _ = self.daemon_tx.try_send(DaemonCommand::Stop);
			event_loop.exit();
		} else if id == menu.dev.id() {
			self.open_named_window(event_loop, AppWindowType::Dashboard);
		} else if id == menu.telemetry.id() {
			self.open_named_window(event_loop, AppWindowType::TelemetryInspector);
		} else if id == menu.task_manager.id() {
			self.open_named_window(event_loop, AppWindowType::TaskManager);
		} else if id == menu.new_task.id() {
			self.new_task();
		} else if id == menu.list_tasks.id() {
			self.show_tasks();
		} else if id == menu.clear_tasks.id() {
			self.clear_tasks();
		}
	}
	fn open_named_window(&mut self, event_loop: &ActiveEventLoop, window_type: AppWindowType) {
		match window_type {
			AppWindowType::Dashboard => {
				tracing::info!("AppWindowType::Dashboard with let ve = Ve::new(Graphics::new());");
				if self.dashboard_window.is_some() {
					return;
				}
				let ve = Ve::new(Graphics::new());
				match Window::new(event_loop, ve) {
					Ok(mut window) => {
						window.window.set_title("Estate Dashboard");
						tracing::info!(">>> Dashboard Window created: {:?}", window.window.id());
						self.dashboard_window = Some(window);
					}
					Err(e) => tracing::error!(">>> Dashboard creation failed: {e:#}"),
				}
			}
			AppWindowType::TelemetryInspector => {
				tracing::info!("AppWindowType::TelemetryInspector");
				if self.telemetry_window.is_some() {
					return;
				}
				let ve = Ve::new(Oracle::new());
				match Window::new(event_loop, ve) {
					Ok(mut window) => {
						window.window.set_title("Telemetry Inspector");
						tracing::info!(">>> Telemetry Window created: {:?}", window.window.id());
						self.telemetry_window = Some(window);
					}
					Err(e) => tracing::error!(">>> Telemetry creation failed: {e:#}"),
				}
			}
			AppWindowType::TaskManager => {
				tracing::info!("AppWindowType::TaskManagerView");
				let ve = Ve::new(TaskManager::new());
				match Window::new(event_loop, ve) {
					Ok(mut window) => {
						window.window.set_title("TaskManagerView");
						self.telemetry_window = Some(window);
					}
					Err(e) => tracing::error!(">>> TaskManagerView creation failed: {e:#}"),
				}
			}
		}
	}
}

impl App {
	fn show_tasks(&mut self) {
		println!("Requesting task/status refresh...");
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::CommandExecuted {
				command: "task_list".into(),
			}));
	}
	fn new_task(&mut self) {
		println!("Creating task...");
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::CommandExecuted {
				command: "task_create".into(),
			}));
	}
	fn clear_tasks(&mut self) {
		println!("Clearing tasks...");
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	fn tray_icon() -> Icon {
		let image = image::load_from_memory(constants::TRAY_ICON)
			.expect("failed to load generated tray icon")
			.into_rgba8();
		let (width, height) = image.dimensions();
		Icon::from_rgba(image.into_raw(), width, height).expect("failed to create tray icon")
	}
	fn scroll_tray_icon() -> tray_icon::Icon {
		let image = image::load_from_memory(constants::TRAY_SCROLL_ICON)
			.expect("failed to load scroll tray icon")
			.into_rgba8();
		let (width, height) = image.dimensions();
		tray_icon::Icon::from_rgba(image.into_raw(), width, height)
			.expect("failed to create scroll tray icon")
	}
	#[tracing::instrument(
		target = "estate::discovery",
		name = "scan_workspace",
		skip(self),
		fields(flow_id = %Uuid::now_v7())
	)]
	async fn scan_workspace(&mut self, path: &Path) -> anyhow::Result<()> {
		tracing::info!("starting workspace scan");
		self.discover(path).await?;
		tracing::debug!("discovery complete");
		self.analyze().await?;
		tracing::debug!("analysis complete");
		self.build_graph().await?;
		tracing::info!("workspace scan complete");
		Ok(())
	}
	#[tracing::instrument(target = "estate::discovery", skip(self, path))]
	async fn discover(&mut self, path: &Path) -> anyhow::Result<()> {
		tracing::debug!(path = %path.display(), "discovering workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::analysis", skip(self))]
	async fn analyze(&mut self) -> anyhow::Result<()> {
		tracing::debug!("analyzing workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::graph", skip(self))]
	async fn build_graph(&mut self) -> anyhow::Result<()> {
		tracing::debug!("building semantic graph");
		Ok(())
	}
}
impl ApplicationHandler<AppEvent> for App {
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			self.handle_menu_event(event, event_loop);
		}
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			self.open_named_window(event_loop, AppWindowType::TaskManager);
		}

		// Main Estate tray.
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

			tracing::info!("🔥 main tray initialized");
		}

		// Scroll controller tray.
		if self.scroll_tray.is_none() {
			match TrayIconBuilder::new()
				.with_icon(Self::scroll_tray_icon())
				.with_tooltip("Estate Scroll Controller")
				.build()
			{
				Ok(tray) => {
					self.scroll_tray = Some(tray);
					tracing::info!("🔥 scroll tray initialized");
				}
				Err(error) => {
					tracing::error!(%error, "failed to create scroll tray");
				}
			}
		}
	}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let target_window = if let Some(ref mut win) = self.dashboard_window {
			if win.window.id() == window_id {
				Some(win)
			} else {
				None
			}
		} else {
			None
		}
		.or_else(|| {
			if let Some(ref mut win) = self.telemetry_window {
				if win.window.id() == window_id {
					Some(win)
				} else {
					None
				}
			} else {
				None
			}
		});
		let Some(window) = target_window else {
			return;
		};
		let response = window.egui_state.on_window_event(&window.window, &event);
		if response.repaint {
			window.window.request_redraw();
		}
		match event {
			WindowEvent::CloseRequested => {
				tracing::info!("🛑 Window close requested for id: {:?}", window_id);

				// Clean up the correct slot based on matching ID
				if let Some(ref win) = self.dashboard_window {
					if win.window.id() == window_id {
						self.dashboard_window = None;
						return;
					}
				}
				if let Some(ref win) = self.telemetry_window {
					if win.window.id() == window_id {
						self.telemetry_window = None;
						return;
					}
				}
			}
			WindowEvent::RedrawRequested => {
				if window.occluded {
					return;
				}

				if let Err(e) = window.draw() {
					tracing::error!("DEV >>> draw failed: {e:#}");
				}
			}
			WindowEvent::Focused(true) => {
				window.window.request_redraw();
			}
			WindowEvent::Occluded(occluded) => {
				window.occluded = occluded;
				if !occluded {
					window.window.request_redraw();
				}
			}
			WindowEvent::Resized(size) => {
				if size.width == 0 || size.height == 0 {
					return;
				}

				window.config.width = size.width;
				window.config.height = size.height;
				window.surface.configure(&window.device, &window.config);
				window.needs_resize = false;
				window.window.request_redraw();
			}
			_ => {}
		}
	}
	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		match event {
			AppEvent::Shutdown => {
				self.shutdown(event_loop);
			}
			AppEvent::CursorPosition { x, y } => {
				let text = format!("↖ {:.0}  {:.0}", x, y);
				let text = format!("← {:.0}  {:.0}", x, y);
				let text = format!("→ {:.0}  {:.0}", x, y);
				let text = format!("↑ {:.0}  {:.0}", x, y);
				let text = format!("● {:.0}, {:.0}", x, y);
				let text = format!("◉ {:.0}, {:.0}", x, y);
				let text = format!("⌖ {:.0}, {:.0}", x, y);
				let text = format!("🟢 {:.0}, {:.0}", x, y);
				let text = format!("🔵 {:.0}, {:.0}", x, y);
				let text = format!("🟡 {:.0}, {:.0}", x, y);
				let text = format!("🔴 {:.0}, {:.0}", x, y);
				let region = if x < 960.0 { "← LEFT" } else { "RIGHT →" };
				if let Some(tray) = &self.scroll_tray {
					let _ = tray.set_title(Some(region));
				}
			}
			AppEvent::TickClock(text) => {
				if let Some(tray) = &self.tray {
					let _ = tray.set_title(Some(text));
				}
			}
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct Context {
	pub source: RuntimeMode,
	pub workspace: PathBuf,
	pub estate_root: PathBuf,
	pub engine_root: PathBuf,
}
#[derive(Clone, Debug, Default)]
pub enum RuntimeMode {
	#[default]
	App,
	Cli,
	Daemon,
	Lsp,
	Tray,
	// ZedEditor,
	// CompilerPipeline,
	// KnowledgeBase,
}
impl Context {
	pub fn new(source: RuntimeMode) -> std::io::Result<Self> {
		Ok(Self {
			source,
			workspace: std::env::current_dir()?,
			estate_root: crate::daemon::resolver::global_estate_dir()?,
			engine_root: crate::daemon::resolver::engine_data_dir()?,
		})
	}
}
struct TrayMenu {
	clear_tasks: MenuItem,
	dev: MenuItem,
	list_tasks: MenuItem,
	new_task: MenuItem,
	quit: MenuItem,
	status: MenuItem,
	task_manager: MenuItem,
	tasks: Submenu,
	telemetry: MenuItem,
	// scroll_item: MenuItem,
}

/// Estate UI Container
///
/// Owns the native window, egui state, wgpu rendering resources, and the
/// application state required to render and interact with the development UI.
/// CPU / Rust
///    │
///    │ create resources + record commands
///    ▼
/// wgpu::Device
///    │
///    │ command encoder
///    ▼
/// wgpu::Queue
///    │
///    │ submit
///    ▼
/// ┌─────────────────────────────────────────────┐
/// │                 GPU PIPELINE                │
/// │                                             │
/// │ Vertex Input                                │
/// │      ↓                                      │
/// │ Vertex Shader                                │
/// │      ↓                                      │
/// │ Primitive Assembly                          │
/// │      ↓                                      │
/// │ Rasterization                               │
/// │      ↓                                      │
/// │ Fragment Shader                              │
/// │      ↓                                      │
/// │ Depth / Stencil / Blending                  │
/// │      ↓                                      │
/// │ Render Target                               │
/// └─────────────────────────────────────────────┘
///    │
///    ▼
/// Surface Texture
///    │
///    ▼
/// Window
/// Vertex data
///     ↓
/// Vertex Shader
///     ↓
/// Primitive assembly
///     ↓
/// Rasterization
///     ↓
/// Fragment Shader
///     ↓
/// Depth / Stencil / Blending
///     ↓
/// Color attachment
pub struct Window {
	pub window: Arc<winit::window::Window>,
	egui_ctx: egui::Context,
	egui_state: EguiState,
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	renderer: egui_wgpu::Renderer,
	config: wgpu::SurfaceConfiguration,
	occluded: bool,
	pending_textures: TexturesDelta,
	needs_resize: bool,
	view: Ve,
	// 1. input assembler
	// 2.vertex shader
	// 3.hull shader
	//4. tesselator
	// 5.domain shader
	// 6.geometry shader
	// 7.rasterizer
	// 8.pixel shader
	// 9.output merger
}

impl Window {
	pub fn new(event_loop: &ActiveEventLoop, view: Ve) -> anyhow::Result<Self> {
		let (egui_ctx, egui_state) = build_egui(event_loop);
		let (window, instance, surface) = create_gpu_surface(event_loop)?;
		let (adapter, device, queue) = initialize_gpu(&instance, &surface)?;
		let size = window.inner_size();
		let (config, renderer) = build_renderer(&surface, adapter, &device, size)?;
		Ok(Self {
			config,
			device,
			egui_ctx,
			egui_state,
			needs_resize: false,
			occluded: true,
			pending_textures: TexturesDelta::default(),
			queue,
			renderer,
			surface,
			view,
			window,
		})
	}
	pub fn draw(&mut self) -> anyhow::Result<()> {
		self.begin_egui();
		let output = self.build_ui();
		let Some(surface_texture) = self.acquire_surface()? else {
			return Ok(());
		};
		self.render_egui(surface_texture, output)?;
		Ok(())
	}
	fn begin_egui(&mut self) {
		let input = self.egui_state.take_egui_input(&self.window);
		self.egui_ctx.begin_pass(input);
	}
	fn build_ui(&mut self) -> egui::FullOutput {
		let mut ui = egui::Ui::new(
			self.egui_ctx.clone(),
			egui::Id::new("window_root"),
			egui::UiBuilder::new(),
		);
		egui::Frame::NONE
			.inner_margin(egui::Margin::same(16))
			.show(&mut ui, |ui| {
				self.view.draw(ui);
			});

		self.egui_ctx.end_pass()
	}
	fn acquire_surface(&mut self) -> anyhow::Result<Option<wgpu::SurfaceTexture>> {
		match self.surface.get_current_texture() {
			wgpu::CurrentSurfaceTexture::Success(texture)
			| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
				// tracing::info!("SURFACE ACQUIRED");
				Ok(Some(texture))
			}
			wgpu::CurrentSurfaceTexture::Occluded => {
				tracing::warn!("SURFACE OCCLUDED");
				Ok(None)
			}
			wgpu::CurrentSurfaceTexture::Timeout => Ok(None),
			wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
				self.reconfigure_surface();
				Ok(None)
			}
			wgpu::CurrentSurfaceTexture::Validation => Err(anyhow::anyhow!("surface validation error")),
		}
	}
	fn reconfigure_surface(&mut self) {
		let size = self.window.inner_size();
		if size.width == 0 || size.height == 0 {
			return;
		}
		self.config.width = size.width;
		self.config.height = size.height;
		self.surface.configure(&self.device, &self.config);
	}
	fn render_egui(
		&mut self,
		surface_texture: wgpu::SurfaceTexture,
		output: egui::FullOutput,
	) -> anyhow::Result<()> {
		let egui::FullOutput {
			pixels_per_point,
			platform_output: _,
			shapes,
			textures_delta,
			viewport_output: _,
			..
		} = output;
		self.pending_textures.append(textures_delta);
		let view = surface_texture
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);
		let screen_descriptor = egui_wgpu::ScreenDescriptor {
			size_in_pixels: [
				self.window.inner_size().width,
				self.window.inner_size().height,
			],
			pixels_per_point,
		};
		self.upload_textures();
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("egui-render"),
			});
		self.renderer.update_buffers(
			&self.device,
			&self.queue,
			&mut encoder,
			&clipped_primitives,
			&screen_descriptor,
		);
		self.render_pass(&mut encoder, &view, &clipped_primitives, &screen_descriptor);
		self.queue.submit(Some(encoder.finish()));
		self.queue.present(surface_texture);
		Ok(())
	}
	fn upload_textures(&mut self) {
		for (id, image_deltas) in &self.pending_textures.set {
			for image_delta in image_deltas {
				self
					.renderer
					.update_texture(&self.device, &self.queue, *id, image_delta);
			}
		}
		self.pending_textures.clear();
	}
	fn render_pass(
		&mut self,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
		primitives: &[egui::ClippedPrimitive],
		screen_descriptor: &egui_wgpu::ScreenDescriptor,
	) {
		let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("egui-render-pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.08,
						g: 0.08,
						b: 0.08,
						a: 1.0,
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});
		let mut render_pass = render_pass.forget_lifetime();
		self
			.renderer
			.render(&mut render_pass, primitives, screen_descriptor);
	}
}
fn initialize_gpu(
	instance: &wgpu::Instance,
	surface: &wgpu::Surface<'_>,
) -> anyhow::Result<(Adapter, Device, wgpu::Queue)> {
	let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
		apply_limit_buckets: true,
		power_preference: wgpu::PowerPreference::HighPerformance,
		compatible_surface: Some(&surface),
		force_fallback_adapter: false,
	}))
	.map_err(|e| anyhow::anyhow!("failed to find suitable GPU adapter: {e}"))?;
	let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
		experimental_features: wgpu::ExperimentalFeatures::disabled(),
		label: Some("estate-dev-device"),
		required_features: wgpu::Features::empty(),
		required_limits: wgpu::Limits::default(),
		memory_hints: wgpu::MemoryHints::Performance,
		trace: wgpu::Trace::Off,
	}))?;
	Ok((adapter, device, queue))
}
fn create_gpu_surface(
	event_loop: &ActiveEventLoop,
) -> anyhow::Result<(
	Arc<winit::window::Window>,
	wgpu::Instance,
	wgpu::Surface<'static>,
)> {
	let window = build_window(event_loop)?;
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
	let surface = unsafe { instance.create_surface(window.clone())? };
	Ok((window, instance, surface))
}
fn build_egui(event_loop: &ActiveEventLoop) -> (EguiContext, EguiState) {
	let egui_ctx = egui::Context::default();
	let egui_state = EguiState::new(
		egui_ctx.clone(),
		egui::ViewportId::ROOT,
		event_loop,
		None,
		None,
		None,
	);
	(egui_ctx, egui_state)
}
fn build_window(event_loop: &ActiveEventLoop) -> anyhow::Result<Arc<winit::window::Window>> {
	let width = 1920;
	let height = 1280;
	let icon = {
		let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
			.expect("failed to load icon")
			.into_rgba8();
		let (width, height) = image.dimensions();
		winit::window::Icon::from_rgba(image.into_raw(), width, height)?
	};
	let mut attrs = winit::window::Window::default_attributes()
		.with_title("Estate Dev")
		.with_inner_size(PhysicalSize::new(width, height))
		.with_window_icon(Some(icon));
	// .with_window_level(WindowLevel::AlwaysOnTop);

	// Calculate bottom-right screen coordinates if a monitor is available
	if let Some(monitor) = event_loop
		.primary_monitor()
		.or_else(|| event_loop.available_monitors().next())
	{
		let screen_size = monitor.size();
		let scale_factor = monitor.scale_factor();

		// Optional: leave a small margin (e.g., 40 pixels) away from the edge/dock
		let margin_x = (40.0 * scale_factor) as i32;
		let margin_y = (60.0 * scale_factor) as i32;

		let x = screen_size.width as i32 - width as i32 - margin_x;
		let y = screen_size.height as i32 - height as i32 - margin_y;

		attrs = attrs.with_position(PhysicalPosition::new(x.max(0), y.max(0)));
	} else {
		// Fallback position if no monitor info is found
		attrs = attrs.with_position(PhysicalPosition::new(100, 100));
	}
	// use eframe::egui;
	let window = event_loop.create_window(attrs)?;
	// Force macOS to show a Dock icon and participate in Cmd + Tab
	#[cfg(target_os = "macos")]
	{
		unsafe {
			let mtm = MainThreadMarker::new().expect("must be on the main thread");
			let app = NSApplication::sharedApplication(mtm);
			app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
			app.activateIgnoringOtherApps(true);
		}
	}
	Ok(Arc::new(window))
}
fn build_renderer(
	surface: &wgpu::Surface<'_>,
	adapter: wgpu::Adapter,
	device: &wgpu::Device,
	size: PhysicalSize<u32>,
) -> Result<
	(
		wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
		Renderer,
	),
	Error,
> {
	let caps = surface.get_capabilities(&adapter);
	let format = caps
		.formats
		.iter()
		.copied()
		.find(|format| {
			matches!(
				format,
				wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
			)
		})
		.or_else(|| caps.formats.first().copied())
		.ok_or_else(|| anyhow::anyhow!("GPU surface has no supported formats"))?;
	let present_mode = caps
		.present_modes
		.iter()
		.copied()
		.find(|mode| *mode == wgpu::PresentMode::Fifo)
		.unwrap_or(wgpu::PresentMode::Fifo);
	let alpha_mode = caps
		.alpha_modes
		.first()
		.copied()
		.ok_or_else(|| anyhow::anyhow!("GPU surface has no alpha modes"))?;
	let config = wgpu::SurfaceConfiguration {
		format,
		alpha_mode,
		present_mode,
		view_formats: vec![],
		width: size.width.max(1),
		height: size.height.max(1),
		desired_maximum_frame_latency: 2,
		color_space: SurfaceColorSpace::Auto,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
	};
	surface.configure(device, &config);
	let renderer = Renderer::new(device, format, egui_wgpu::RendererOptions::default());
	Ok((config, renderer))
}

// 80/20 curriculum. The goal isn't to build every subsystem completely; it's to learn where the boundaries are.

// ### 1. Hot reloading

// * **Goal:** Change a JSON/config file on disk → Estate notices it → reloads state → UI reflects it.
// * **Stage 1 — File model**

//   * Define the file being watched.
//   * Define the deserialized config/state type.
//   * Define defaults and validation.
// * **Stage 2 — Watcher**

//   * Use a filesystem watcher (`notify` is the obvious Rust choice).
//   * Boundary: watcher only reports **"this path changed."**
//   * It should not know anything about egui or Estate business logic.
// * **Stage 3 — Reload boundary**

//   * `reload_config(path) -> Result<Config>`.
//   * Read → parse → validate.
//   * Keep this synchronous/simple initially.
// * **Stage 4 — Event propagation**

//   * Watcher sends something like `AppEvent::ConfigChanged`.
//   * App/engine receives it.
//   * Engine reloads the configuration.
// * **Stage 5 — UI update**

//   * DevWindow observes the updated state.
//   * Request redraw.
// * **Parameters worth learning**

//   * Debouncing.
//   * Atomic writes/temp files.
//   * Invalid JSON.
//   * File deleted/recreated.
//   * Watcher lifetime.
// * **Boundary to aim for:**

//   * `filesystem → watcher → application event → state → UI`
//   * Not `filesystem → egui`.

// ---

// ### 2. Tab state / navigation

// * **Goal:** Top tabs and sidebar tabs actually control what is rendered.
// * **Stage 1 — State**

//   * `DevTopTab`
//   * `DevSideTab`
//   * Store selected values in `DevWindow`.
// * **Stage 2 — Input**

//   * Render each tab as an egui `selectable_label`, button, etc.
//   * On click, mutate the enum.
// * **Stage 3 — Rendering**

//   * `draw_top_bar()`
//   * `draw_sidebar()`
//   * `draw_content()`
// * **Stage 4 — Dispatch**

//   * `draw_content()` matches on `self.side_tab`.
//   * Each variant delegates to its own renderer.
// * **Stage 5 — Persistence, optionally**

//   * Remember the last selected tab.
//   * This is a good tiny exercise in state persistence.
// * **Parameters**

//   * Enum vs string IDs.
//   * Selection state.
//   * `egui::Id`.
//   * Redraw/invalidation.
// * **Boundary:**

//   * **State determines UI; UI does not own the state.**

// This one is probably your most important egui exercise.

// ---

// ### 3. OS app switcher / `Cmd+Tab`

// * **Goal:** Estate behaves like a real macOS application/window.
// * **Stage 1 — Window identity**

//   * Give the window a proper title.
//   * Set application/window metadata.
// * **Stage 2 — macOS application identity**

//   * Ensure the `.app` bundle has the correct:

//     * bundle identifier
//     * executable
//     * application name
//     * icon
// * **Stage 3 — Icon**

//   * Create `.icns`.
//   * Add it to the application bundle.
//   * Set the appropriate bundle metadata.
// * **Stage 4 — Activation**

//   * Verify clicking Estate in `Cmd+Tab` activates the existing window.
//   * Don't accidentally create a second window.
// * **Stage 5 — Multi-window behavior**

//   * Understand:

//     * application
//     * window
//     * process
//     * activation
// * **Parameters**

//   * macOS `Info.plist`
//   * bundle ID
//   * application icon
//   * window activation
// * **Boundary:**

//   * This is mostly **OS/application packaging**, not egui/wgpu.

// This is a particularly useful thing to learn because it separates *"I can render a window"* from *"I built an actual desktop application."*

// ---

// ### 4. Real Estate Engine metrics

// * **Goal:** Turn your existing engine state into a useful dashboard.
// * **Stage 1 — Define metrics**

//   * Start with things you already have:

//     * uptime
//     * starts
//     * status checks
//     * longest run
//     * resources
//     * nodes
//     * registry size
// * **Stage 2 — Metrics model**

//   * Create something like:

//   `EngineMetrics`

//   rather than making the UI inspect random engine internals.
// * **Stage 3 — Collection**

//   * Engine exposes a snapshot:

//     * `engine.metrics() -> EngineMetrics`
// * **Stage 4 — UI**

//   * Cards/stat rows:

//     * `Uptime`
//     * `Resources`
//     * `Nodes`
//     * `Status`
//     * etc.
// * **Stage 5 — Refresh**

//   * Periodically update metrics.
//   * Learn egui repaint timing.
// * **Stage 6 — Historical metrics**

//   * Optional:

//     * requests/sec
//     * index operations
//     * average latency
//     * memory
// * **Parameters**

//   * Snapshot vs live references.
//   * Counters vs gauges.
//   * Refresh frequency.
//   * Thread safety.
// * **Boundary:**

//   `Engine → MetricsSnapshot → DevWindow`

//   The UI shouldn't reach deep into the engine.

// ---

// ### 5. Logs UI

// * **Goal:** Make the DevWindow itself useful for debugging Estate.
// * **Stage 1 — Log capture**

//   * Create an application-owned logging layer.
//   * Capture tracing events.
// * **Stage 2 — Log model**

//   * Something like:

//   `LogEntry { timestamp, level, target, message }`
// * **Stage 3 — Buffer**

//   * Store recent entries in a bounded buffer.
//   * e.g. last 1,000/5,000 messages.
// * **Stage 4 — UI**

//   * `ScrollArea::vertical()`
//   * timestamp
//   * level
//   * target
//   * message
// * **Stage 5 — Filtering**

//   * `TRACE`
//   * `DEBUG`
//   * `INFO`
//   * `WARN`
//   * `ERROR`
// * **Stage 6 — Search**

//   * Filter by text.
// * **Stage 7 — Auto-scroll**

//   * Follow newest messages unless the user scrolls upward.
// * **Stage 8 — Clear/export**

//   * Clear buffer.
//   * Optionally save logs.
// * **Parameters**

//   * `tracing_subscriber`
//   * channel/buffer architecture
//   * bounded memory
//   * UI repaint on incoming logs
// * **Boundary:**

//   `tracing event → log collector → LogEntry buffer → egui`

// This one teaches you a **very real desktop-app pattern**: background events feeding a UI.

// ---

// ### 6. Embedded WebViews

// This is the most interesting one because it connects directly to the stuff you've already been experimenting with.
// * **Goal:** Render HTML/CSS/JS inside your Estate desktop application.
// * **Stage 1 — WebView**

//   * Embed a native WebView.
//   * Get a static HTML page rendering.
// * **Stage 2 — Local content**

//   * Load HTML/CSS/JS from disk or memory.
// * **Stage 3 — Communication**

//   * Rust → JavaScript
//   * JavaScript → Rust
// * **Stage 4 — Estate bridge**

//   * Expose carefully selected operations:

//     * `get_status()`
//     * `get_resources()`
//     * `resolve_node()`
//     * etc.
// * **Stage 5 — UI integration**

//   * Decide whether WebView is:

//     * a separate window
//     * a panel
//     * a tab
//     * a full application view
// * **Stage 6 — Asset loading**

//   * CSS
//   * JS
//   * images
//   * fonts
//   * local resources
// * **Stage 7 — Security boundary**

//   * Decide exactly what JavaScript is allowed to call.
//   * Don't expose arbitrary filesystem/process access.
// * **Parameters**

//   * WebView lifecycle.
//   * navigation.
//   * IPC.
//   * origin/security.
//   * local asset resolution.
//   * native ↔ JS serialization.
// * **Boundary:**

//   `egui/native UI ↔ WebView ↔ JS application`

//   with a deliberately tiny:

//   `Rust ↔ JS API`

// ---

// ## The order I'd actually do them
// I'd slightly reorder your list:
// 1. **Tab state**

//    * Learn application state → rendering.
// 2. **Real engine metrics**

//    * Learn backend state → UI state.
// 3. **Logs**

//    * Learn asynchronous events → UI.
// 4. **Hot reload**

//    * Learn filesystem events → application state.
// 5. **Cmd+Tab / app icon**

//    * Learn OS/application packaging.
// 6. **WebView**

//    * Learn native UI ↔ another rendering runtime.
// That gives you a pretty nice progression:
// ```text
//                 ┌──────────────┐
//                 │   Estate     │
//                 │    Engine    │
//                 └──────┬───────┘
//                        │
//                  state / events
//                        │
//                        ▼
//               ┌─────────────────┐
//               │   App / Model   │
//               └───────┬─────────┘
//                       │
//           ┌───────────┼────────────┐
//           ▼           ▼            ▼
//        egui UI      logs        WebView
//           │
//           ▼
//        wgpu
//           │
//           ▼
//      native window
//           │
//           ▼
//        macOS
// ```
impl Window {
	fn foo_layout_sidebar_top(&mut self, ui: &mut egui::Ui) {
		// ┌─────────────────────────────────────────────┐
		// │                    TOP                      │
		// ├──────────────┬──────────────────────────────┤
		// │              │                              │
		// │   SIDEBAR    │             MAIN             │
		// │              │                              │
		// └──────────────┴──────────────────────────────┘

		egui::containers::Panel::top(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::LeftToRight,
				main_wrap: false,
				main_align: egui::Align::Center,
				main_justify: true,
				cross_align: egui::Align::Center,
				cross_justify: false,
			};

			let mut top = ui.new_child(egui::UiBuilder::new().layout(layout));

			top.label("ESTATE");
			top.label("STATUS");
			top.label("REGISTRY");
			top.label("RUNTIME");
		});

		egui::containers::Panel::left(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::TopDown,
				main_wrap: false,
				main_align: egui::Align::Min,
				main_justify: false,
				cross_align: egui::Align::Min,
				cross_justify: true,
			};

			let mut sidebar = ui.new_child(egui::UiBuilder::new().layout(layout));

			sidebar.label("Overview");
			sidebar.label("Registry");
			sidebar.label("Daemon");
			sidebar.label("Engine");
			sidebar.label("Workspace");
		});

		let layout = egui::Layout {
			main_dir: egui::Direction::TopDown,
			main_wrap: false,
			main_align: egui::Align::Min,
			main_justify: false,
			cross_align: egui::Align::Min,
			cross_justify: true,
		};

		let mut main = ui.new_child(egui::UiBuilder::new().layout(layout));
		main.heading("MAIN");
		main.label("This area consumes the remaining space.");
	}
	fn foo_layout_three_columns(&mut self, ui: &mut egui::Ui) {
		// ┌─────────────────────────────────────────────┐
		// │                    TOP                      │
		// ├──────────┬─────────────────────┬────────────┤
		// │          │                     │            │
		// │ SIDEBAR  │        MAIN         │   ASIDE    │
		// │          │                     │            │
		// ├──────────┴─────────────────────┴────────────┤
		// │                   FOOTER                    │
		// └─────────────────────────────────────────────┘

		egui::containers::Panel::top(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::LeftToRight,
				main_wrap: false,
				main_align: egui::Align::Center,
				main_justify: true,
				cross_align: egui::Align::Center,
				cross_justify: false,
			};

			let mut top = ui.new_child(egui::UiBuilder::new().layout(layout));

			top.label("ESTATE");
			top.label("PROJECT");
			top.label("COMMANDS");
		});
		egui::containers::Panel::bottom(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::LeftToRight,
				main_wrap: false,
				main_align: egui::Align::Center,
				main_justify: true,
				cross_align: egui::Align::Center,
				cross_justify: false,
			};

			let mut footer = ui.new_child(egui::UiBuilder::new().layout(layout));

			footer.label("Connected");
			footer.label("v0.1.0");
		});
		egui::containers::Panel::left(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::TopDown,
				main_wrap: false,
				main_align: egui::Align::Min,
				main_justify: false,
				cross_align: egui::Align::Min,
				cross_justify: true,
			};

			let mut sidebar = ui.new_child(egui::UiBuilder::new().layout(layout));

			sidebar.label("Files");
			sidebar.label("Registry");
			sidebar.label("Resources");
		});
		egui::containers::Panel::right(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::TopDown,
				main_wrap: false,
				main_align: egui::Align::Min,
				main_justify: false,
				cross_align: egui::Align::Min,
				cross_justify: true,
			};

			let mut aside = ui.new_child(egui::UiBuilder::new().layout(layout));

			aside.label("Inspector");
			aside.label("Properties");
			aside.label("Details");
		});
		let layout = egui::Layout {
			main_dir: egui::Direction::TopDown,
			main_wrap: false,
			main_align: egui::Align::Min,
			main_justify: false,
			cross_align: egui::Align::Min,
			cross_justify: true,
		};
		let mut main = ui.new_child(egui::UiBuilder::new().layout(layout));
		main.heading("MAIN");
		main.label("The remaining space belongs to the main content.");
	}
	fn foo_layout_infinite_scroll(&mut self, ui: &mut egui::Ui) {
		// ┌─────────────────────────────────────────────┐
		// │                    TOP                      │
		// ├─────────────────────────────────────────────┤
		// │                                             │
		// │                  SCROLLER                   │
		// │                  ↓                          │
		// │               content 1                     │
		// │               content 2                     │
		// │               content 3                     │
		// │                  ...                        │
		// │                                             │
		// ├─────────────────────────────────────────────┤
		// │                   FOOTER                    │
		// └─────────────────────────────────────────────┘

		egui::containers::Panel::top(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::LeftToRight,
				main_wrap: false,
				main_align: egui::Align::Center,
				main_justify: true,
				cross_align: egui::Align::Center,
				cross_justify: false,
			};

			let mut top = ui.new_child(egui::UiBuilder::new().layout(layout));

			top.label("ESTATE");
			top.label("SEARCH");
			top.label("FILTER");
		});

		egui::containers::Panel::bottom(ui.id()).show(ui, |ui| {
			let layout = egui::Layout {
				main_dir: egui::Direction::LeftToRight,
				main_wrap: false,
				main_align: egui::Align::Center,
				main_justify: true,
				cross_align: egui::Align::Center,
				cross_justify: false,
			};

			let mut footer = ui.new_child(egui::UiBuilder::new().layout(layout));

			footer.label("Status");
			footer.label("Connected");
		});

		egui::ScrollArea::vertical()
			.id_salt("infinite_content")
			.auto_shrink([false, false])
			.show(ui, |ui| {
				let layout = egui::Layout {
					main_dir: egui::Direction::TopDown,
					main_wrap: false,
					main_align: egui::Align::Min,
					main_justify: false,
					cross_align: egui::Align::Min,
					cross_justify: true,
				};

				let mut content = ui.new_child(egui::UiBuilder::new().layout(layout));

				for i in 0..1000 {
					content.horizontal(|ui| {
						ui.label(format!("{:04}", i));
						ui.separator();
						ui.label(format!("Resource {}", i));
					});
				}
			});
	}
}

struct AppState {
	tray_icon: TrayIcon,
}
// impl AppState {
// 	/// Updates the tray menu title text based on screen region thresholds
// 	pub fn update_tray_indicator(&mut self, mouse_x: f32, total_width: f32) {
// 		if total_width <= 0.0 {
// 			return;
// 		}

// 		let ratio = mouse_x / total_width;

// 		let indicator_text = if ratio < 0.25 {
// 			"[ ◀︎ ]  ·   · " // Left zone
// 		} else if ratio > 0.75 {
// 			" ·   ·  [ ▶︎ ]" // Right zone
// 		} else {
// 			" ·   [ ▲ ]  · " // Center zone
// 		};

// 		// If your TrayIcon instance is stored in your app state:
// 		if let Some(tray) = &self.tray_icon {
// 			let _ = tray.set_title(Some(indicator_text));
// 		}
// 	}
// }
#[derive(Debug)]
pub enum AppEvent {
	Shutdown,
	CursorPosition { x: f64, y: f64 },
	TickClock(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppWindowType {
	Dashboard,
	TelemetryInspector,
	TaskManager,
}

pub struct NamedWindow {
	pub window_type: AppWindowType,
	pub window_handle: Window,
}

pub struct GlobalHotkeys {
	manager: GlobalHotKeyManager,
	shutdown: Arc<AtomicBool>,
}

impl GlobalHotkeys {
	pub fn new() -> anyhow::Result<Self> {
		let manager = GlobalHotKeyManager::new()?;

		let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);

		manager.register(hotkey)?;

		Ok(Self {
			manager,
			shutdown: Arc::new(AtomicBool::new(false)),
		})
	}
}

// struct Icon {}
// impl Icon {}
