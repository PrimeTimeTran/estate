use crate::prelude::*;
use egui::{Context as EguiContext, PopupAnchor::Position, TexturesDelta, Ui};
use egui_wgpu::{
	Renderer, SurfaceConfig,
	wgpu::{self, hal::InstanceDescriptor},
};
use egui_winit::State as EguiState;
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
	event_loop::{ActiveEventLoop, EventLoop},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::{Window, WindowAttributes, WindowId, WindowLevel},
};

/// Top-level application state.
///
/// Owns the system tray integration, application context, Estate engine,
/// daemon communication channel, and optional development window.
pub struct App {
	/// The system tray icon owned by the application.
	tray: TrayIcon,

	/// The system tray menu and its associated menu items.
	menu: TrayMenu,

	/// Shared application context and runtime state.
	context: Context,

	/// The Estate engine responsible for the application's core functionality.
	engine: EstateEngine,

	/// Channel used to send commands to the Estate daemon, when available.
	daemon_tx: Option<mpsc::Sender<DaemonCommand>>,

	/// The development window, when it has been opened.
	dev_window: Option<DevWindow>,
}
impl App {
	fn new(context: Context, engine: EstateEngine) -> anyhow::Result<Self> {
		let (menu, tray) = Self::bootstrap()?;

		Ok(Self {
			context,
			engine,
			tray,
			menu,
			dev_window: None,
			daemon_tx: None,
		})
	}
	fn bootstrap() -> anyhow::Result<(TrayMenu, TrayIcon)> {
		let menu = Menu::new();
		let status = MenuItem::new("● Estate Daemon Running", false, None);
		let dev = MenuItem::new("Dev Info", true, None);

		// ─────────────────────────────────────────────
		// Tasks submenu
		// ─────────────────────────────────────────────

		let new_task = MenuItem::new("New Task", true, None);
		let list_tasks = MenuItem::new("List Tasks", true, None);
		let clear_tasks = MenuItem::new("Clear Tasks", true, None);

		let tasks = Submenu::new("Tasks", true);

		tasks.append(&new_task)?;
		tasks.append(&list_tasks)?;
		tasks.append(&clear_tasks)?;

		// ─────────────────────────────────────────────
		// Root menu
		// ─────────────────────────────────────────────

		let quit = MenuItem::new("Quit", true, None);

		menu.append(&status)?;
		menu.append(&dev)?;
		menu.append(&tasks)?;
		menu.append(&quit)?;

		let tray = TrayIconBuilder::new()
			.with_icon(Self::tray_icon())
			.with_menu(Box::new(menu))
			.with_tooltip("Estate Daemon — Running")
			.build()
			.map_err(|e| anyhow::anyhow!("failed to create tray icon: {e}"))?;

		Ok((
			TrayMenu {
				status,
				dev,
				tasks,
				new_task,
				list_tasks,
				clear_tasks,
				quit,
			},
			tray,
		))
	}
	fn tray_icon() -> Icon {
		let image = image::load_from_memory(constants::TRAY_ICON)
			.expect("failed to load generated tray icon")
			.into_rgba8();

		let (width, height) = image.dimensions();

		Icon::from_rgba(image.into_raw(), width, height).expect("failed to create tray icon")
	}
}
impl App {
	#[tracing::instrument(
		target = "estate::discovery",
		name = "scan_workspace",
		skip(self),
		fields(flow_id = %Uuid::now_v7())
	)]
	pub async fn scan_workspace(&mut self, path: &Path) -> anyhow::Result<()> {
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
	pub async fn discover(&mut self, path: &Path) -> anyhow::Result<()> {
		tracing::debug!(path = %path.display(), "discovering workspace");
		Ok(())
	}

	#[tracing::instrument(target = "estate::analysis", skip(self))]
	pub async fn analyze(&mut self) -> anyhow::Result<()> {
		tracing::debug!("analyzing workspace");
		Ok(())
	}
	fn show_tasks(&self) {
		println!("Estate Tasks");
	}
	fn new_task(&mut self) {
		println!("Creating task...");
	}
	fn clear_tasks(&mut self) {
		println!("Clearing tasks...");
	}

	#[tracing::instrument(target = "estate::graph", skip(self))]
	pub async fn build_graph(&mut self) -> anyhow::Result<()> {
		tracing::debug!("building semantic graph");

		// TODO

		Ok(())
	}
	pub async fn spawn_tray_process() -> anyhow::Result<()> {
		eprintln!(">>> spawn_tray_process");
		if Daemon::is_running().await {
			return Ok(());
		}
		let exe = std::env::current_exe()?;
		eprintln!(">>> spawning tray: {}", exe.display());
		std::process::Command::new(exe)
			.arg("tray")
			.stdin(std::process::Stdio::inherit())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.spawn()?;
		eprintln!(">>> tray process spawned");
		Ok(())
	}
	/// Run the tray application in the current process.
	pub fn run_tray_daemon(engine: EstateEngine) -> anyhow::Result<()> {
		let context = Context::new(RuntimeMode::Cli)?;

		let event_loop = EventLoop::builder()
			.with_activation_policy(ActivationPolicy::Accessory)
			.build()?;

		let mut app = Self::new(context, engine)?;

		event_loop.run_app(&mut app)?;

		Ok(())
	}
	fn run_daemon(engine: EstateEngine, mut rx: mpsc::Receiver<DaemonCommand>) {
		let runtime = Runtime::new().unwrap();

		runtime.block_on(async move {
			let mut daemon = Daemon::new(engine);

			let daemon_task = tokio::spawn(async move {
				if let Err(e) = daemon.run_foreground().await {
					eprintln!("Daemon error: {e}");
				}
			});

			tokio::select! {
					result = daemon_task => {
							match result {
									Ok(()) => eprintln!("Daemon exited"),
									Err(e) => eprintln!("Daemon task failed: {e}"),
							}
					}

					command = rx.recv() => {
							match command {
									Some(DaemonCommand::Stop) => {
											eprintln!("Stopping daemon...");
									}
									None => {
											eprintln!("Daemon command channel closed");
									}
							}
					}
			}
		});
	}
	fn handle_menu_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		if event.id() == self.menu.quit.id() {
			if let Some(tx) = &self.daemon_tx {
				let _ = tx.send(DaemonCommand::Stop);
			}
			event_loop.exit();
		} else if event.id() == self.menu.dev.id() {
			tracing::info!("processing loop started");
			eprintln!(">>> DEV INFO CLICKED");
			self.show_dev_info(event_loop);
		} else if event.id() == self.menu.new_task.id() {
			self.new_task();
		} else if event.id() == self.menu.list_tasks.id() {
			self.show_tasks();
		} else if event.id() == self.menu.clear_tasks.id() {
			self.clear_tasks();
		}
	}
	fn show_dev_info(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("show_dev_info");
		match DevWindow::new(event_loop) {
			Ok(window) => {
				tracing::info!(">>> DevWindow created: {:?}", window.window.id());
				self.dev_window = Some(window);
			}
			Err(e) => {
				tracing::error!(">>> DevWindow creation failed: {e:#}");
			}
		}
	}
}
impl ApplicationHandler for App {
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			self.handle_menu_event(event, event_loop);
		}
	}
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		let (tx, rx) = mpsc::channel(100);
		self.daemon_tx = Some(tx);
		let engine = self.engine.clone();
		thread::spawn(move || {
			Self::run_daemon(engine, rx);
		});
	}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(dev_window) = &mut self.dev_window else {
			return;
		};

		if dev_window.window.id() != window_id {
			return;
		}

		let response = dev_window
			.egui_state
			.on_window_event(&dev_window.window, &event);

		// tracing::info!(
		// 	"EGUI EVENT: {:?}, consumed={}, repaint={}",
		// 	event,
		// 	response.consumed,
		// 	response.repaint
		// );

		if response.repaint {
			dev_window.window.request_redraw();
		}
		match event {
			WindowEvent::RedrawRequested => {
				if dev_window.occluded {
					// Very noisy on loop render
					// tracing::info!("DEV Redraw skipped: occluded");
					return;
				}

				// tracing::info!("DEV >>> RedrawRequested");

				if let Err(e) = dev_window.draw() {
					tracing::error!("DEV >>> draw failed: {e:#}");
				}
			}
			WindowEvent::Focused(true) => {
				dev_window.window.request_redraw();
			}
			WindowEvent::Occluded(occluded) => {
				tracing::info!("DEV >>> Occluded({occluded})");
				dev_window.occluded = occluded;

				if !occluded {
					dev_window.window.request_redraw();
				}
			}
			WindowEvent::Resized(size) => {
				tracing::info!("DEV >>> Resized({}x{})", size.width, size.height);

				if size.width == 0 || size.height == 0 {
					return;
				}

				dev_window.config.width = size.width;
				dev_window.config.height = size.height;

				dev_window
					.surface
					.configure(&dev_window.device, &dev_window.config);

				dev_window.needs_resize = false;
				dev_window.window.request_redraw();
			}
			WindowEvent::CloseRequested => {
				tracing::info!("DEV >>> CloseRequested");
			}
			_ => {}
		}
	}
}

/// The development/debug window for the Estate application.
///
/// Owns the native window, egui state, wgpu rendering resources, and the
/// application state required to render and interact with the development UI.
pub struct DevWindow {
	/// The native window handle.
	window: Arc<Window>,

	/// The egui context used to build and render the UI.
	egui_ctx: egui::Context,

	/// Translates native window events and input into egui input.
	egui_state: EguiState,

	/// The wgpu surface associated with the native window.
	surface: wgpu::Surface<'static>,

	/// The GPU device used to create rendering resources.
	device: wgpu::Device,

	/// The GPU queue used to submit rendering commands.
	queue: wgpu::Queue,

	/// The egui renderer backed by wgpu.
	renderer: egui_wgpu::Renderer,

	/// Configuration used to configure the wgpu surface.
	config: wgpu::SurfaceConfiguration,

	/// The currently selected top-level development view.
	top_tab: DevTopTab,

	/// The currently selected side-panel development view.
	side_tab: DevSideTab,

	/// Current Estate daemon/application state displayed by the development UI.
	state: EstateState,

	/// Whether the window's surface is currently occluded and therefore cannot
	/// be rendered to.
	occluded: bool,

	/// Texture updates produced by egui that have not yet been uploaded to the
	/// GPU.
	pending_textures: egui::TexturesDelta,

	needs_resize: bool,
}
impl DevWindow {
	pub fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
		let (egui_ctx, egui_state) = build_egui(event_loop);
		let (window, instance, surface) = create_gpu_surface(&event_loop)?;
		let (adapter, device, queue) = initialize_gpu(&instance, &surface)?;
		let size = window.inner_size();
		let (config, renderer) = build_renderer(&surface, adapter, &device, size)?;
		Ok(Self {
			queue,
			config,
			device,
			surface,
			renderer,
			egui_ctx,
			egui_state,
			occluded: true,
			needs_resize: false,
			window: window.clone(),
			state: EstateState::load(),
			top_tab: DevTopTab::Status,
			side_tab: DevSideTab::Overview,
			pending_textures: TexturesDelta::default(),
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
			egui::Id::new("dev_root"),
			egui::UiBuilder::new(),
		);
		// self.foo_layout_infinite_scroll(&mut ui);
		// self.foo_layout_sidebar_top(&mut ui);
		// self.foo_layout_three_columns(&mut ui);
		self.draw_ui(&mut ui);
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
			platform_output: _,
			textures_delta,
			shapes,
			pixels_per_point,
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
	fn draw_content(&mut self, ui: &mut egui::Ui) {
		match self.top_tab {
			DevTopTab::Status => self.draw_status(ui),
			DevTopTab::Tasks => self.draw_tasks(ui),
			DevTopTab::Logs => self.draw_logs(ui),
			DevTopTab::Config => self.draw_config(ui),
		}
	}
	fn draw_ui(&mut self, ui: &mut egui::Ui) {
		self.draw_side_tabs(ui);

		egui::CentralPanel::default().show_inside(ui, |ui| {
			self.draw_status(ui);
		});
	}
	fn draw_side_tabs(&mut self, ui: &mut egui::Ui) {
		ui.heading("Estate");
		ui.separator();
		for &tab in DevSideTab::ALL {
			let response = ui.selectable_label(self.side_tab == tab, tab.label());

			// tracing::info!(
			// 	"TAB {} rect={:?} hovered={} clicked={} contains_pointer={}",
			// 	tab.label(),
			// 	response.rect,
			// 	response.hovered(),
			// 	response.clicked(),
			// 	response.contains_pointer(),
			// );

			if response.clicked() {
				tracing::info!(">>> TAB CLICKED: {:?}", tab);
				self.side_tab = tab;
			}
		}
	}
	fn draw_status(&self, ui: &mut egui::Ui) {
		match self.side_tab {
			DevSideTab::Overview => self.draw_overview(ui),
			DevSideTab::Registry => self.draw_registry(ui),
			DevSideTab::Daemon => self.draw_daemon(ui),
			DevSideTab::Engine => self.draw_engine(ui),
			DevSideTab::Workspace => self.draw_workspace(ui),
			DevSideTab::Runtime => self.draw_runtime(ui),
		}
	}
	fn draw_overview(&self, ui: &mut egui::Ui) {
		ui.heading("Overview");
		ui.separator();
		ui.label(format!("Starts: {}", self.state.starts));
		ui.label(format!("Status checks: {}", self.state.status_checks));
		ui.label(format!("Started at: {}", self.state.started_at));
		ui.label(format!("Longest run: {}s", self.state.longest_run));
	}
	fn draw_registry(&self, ui: &mut egui::Ui) {
		ui.heading("Registry");
		ui.separator();
		ui.label("Registry view");
	}

	fn draw_daemon(&self, ui: &mut egui::Ui) {
		ui.heading("Daemon");
		ui.separator();
		ui.label("Daemon view");
	}

	fn draw_engine(&self, ui: &mut egui::Ui) {
		ui.heading("Engine");
		ui.separator();
		ui.label("Engine view");
	}

	fn draw_workspace(&self, ui: &mut egui::Ui) {
		ui.heading("Workspace");
		ui.separator();
		ui.label("Workspace view");
	}

	fn draw_runtime(&self, ui: &mut egui::Ui) {
		todo!("Error!");
		ui.heading("Runtime");
		ui.separator();
		ui.label("Runtime view");
	}
	fn draw_tasks(&self, ui: &mut egui::Ui) {
		ui.heading("Tasks");
		ui.separator();
		ui.label("Task manager coming soon.");
	}
	fn draw_logs(&self, ui: &mut egui::Ui) {
		ui.heading("Logs");
		ui.separator();
		ui.label("Logs coming soon.");
	}
	fn draw_config(&self, ui: &mut egui::Ui) {
		ui.heading("Configuration");
		ui.separator();
		ui.label("Configuration coming soon.");
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
) -> anyhow::Result<(Arc<Window>, wgpu::Instance, wgpu::Surface<'static>)> {
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
fn build_egui2(event_loop: &ActiveEventLoop) -> (EguiContext, EguiState) {
	// let monitor = event_loop
	// 	.primary_monitor()
	// 	.ok_or_else(|| anyhow::anyhow!("no primary monitor found"))?;
	// let monitor_size = monitor.size();
	// let width = 900;
	// let height = 600;
	// let x = (monitor_size.width.saturating_sub(width)) / 2;
	// let y = (monitor_size.height.saturating_sub(height)) / 2;
	// let attrs = Window::default_attributes()
	// 	.with_title("Estate Dev")
	// 	.with_inner_size(PhysicalSize::new(900, 600))
	// 	.with_position(PhysicalPosition::new(x, y))
	// 	.with_window_level(WindowLevel::AlwaysOnTop);

	// let x = (monitor_size.width.saturating_sub(width)) / 2;
	// let y = (monitor_size.height.saturating_sub(height)) / 2;

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
fn build_window(event_loop: &ActiveEventLoop) -> anyhow::Result<Arc<Window>> {
	let attrs = Window::default_attributes()
		.with_title("Estate Dev")
		.with_inner_size(PhysicalSize::new(900, 600))
		.with_window_level(WindowLevel::AlwaysOnTop)
		.with_position(PhysicalPosition::new(0, 0));

	let window = event_loop.create_window(attrs)?;

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

#[derive(Clone, Debug)]
pub struct Context {
	pub source: RuntimeMode,
	// Where the user is operating
	pub workspace: PathBuf,
	// Global user estate (~/.estate)
	pub estate_root: PathBuf,
	// Engine internals (cache, daemon state, registry)
	pub engine_root: PathBuf,
}
#[derive(Clone, Debug)]
pub enum RuntimeMode {
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
#[derive(Clone, Debug)]
enum DaemonCommand {
	Stop,
	// Metrics,
	// Restart,
	// Refresh,
	// Enable,
	// Disable,
	// Status,
}
struct TrayMenu {
	status: MenuItem,
	dev: MenuItem,
	tasks: Submenu,
	new_task: MenuItem,
	list_tasks: MenuItem,
	clear_tasks: MenuItem,
	quit: MenuItem,
}
#[derive(Clone, Default)]
pub struct TaskManager {
	tasks: HashMap<TaskId, Task>,
}
pub type TaskId = Uuid;

#[derive(Debug, Clone)]
pub struct Task {
	pub id: TaskId,
	pub name: String,
	pub status: TaskStatus,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
	Pending,
	Running,
	Completed,
	Failed,
	Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevTopTab {
	Status,
	Tasks,
	Logs,
	Config,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DevSideTab {
	#[default]
	Overview,
	Registry,
	Daemon,
	Engine,
	Workspace,
	Runtime,
}

impl DevSideTab {
	const ALL: &[Self] = &[
		Self::Overview,
		Self::Registry,
		Self::Daemon,
		Self::Engine,
		Self::Workspace,
		Self::Runtime,
	];

	fn label(self) -> &'static str {
		match self {
			Self::Overview => "Overview",
			Self::Registry => "Registry",
			Self::Daemon => "Daemon",
			Self::Engine => "Engine",
			Self::Workspace => "Workspace",
			Self::Runtime => "Runtime",
		}
	}
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
impl DevWindow {
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
