use std::sync::Arc;

use crate::prelude::*;
use egui::{Context as EguiContext, PopupAnchor::Position, TexturesDelta, Ui};
use egui_wgpu::{
	Renderer, SurfaceConfig,
	wgpu::{self, hal::InstanceDescriptor},
};
use egui_winit::State as EguiState;
use tokio::runtime::Runtime;
use tracing::instrument::WithSubscriber;
use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, Submenu},
};
use wgpu::{ExperimentalFeatures, SurfaceColorSpace, SurfaceConfiguration};
use winit::{
	application::ApplicationHandler,
	dpi::{LogicalPosition, LogicalSize, PhysicalSize},
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::{Window, WindowAttributes, WindowId},
};

pub struct App {
	tray: TrayIcon,
	menu: TrayMenu,
	context: Context,
	engine: EstateEngine,
	daemon_tx: Option<mpsc::Sender<DaemonCommand>>,
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

		// TODO

		Ok(())
	}

	#[tracing::instrument(target = "estate::analysis", skip(self))]
	pub async fn analyze(&mut self) -> anyhow::Result<()> {
		tracing::debug!("analyzing workspace");

		// TODO

		Ok(())
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
	fn show_tasks(&self) {
		println!("Estate Tasks");
	}
	fn new_task(&mut self) {
		println!("Creating task...");
	}
	fn clear_tasks(&mut self) {
		println!("Clearing tasks...");
	}
}
impl ApplicationHandler for App {
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			eprintln!(">>> MENU EVENT: {:?}", event.id());
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
		if let Some(dev_window) = &mut self.dev_window {
			if dev_window.window.id() != window_id {
				return;
			}
			match event {
				WindowEvent::RedrawRequested => {
					if dev_window.occluded {
						tracing::info!("DEV >>> Redraw skipped: occluded");
						return;
					}

					tracing::info!("DEV >>> RedrawRequested");

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
						tracing::info!("DEV >>> WINDOW IS NOW DRAWABLE — requesting redraw");
						dev_window.window.request_redraw();
					}
				}
				WindowEvent::Resized(size) => {
					tracing::info!("DEV >>> Resized({}x{})", size.width, size.height);
				}

				WindowEvent::CloseRequested => {
					tracing::info!("DEV >>> CloseRequested");
				}

				other => {
					tracing::debug!("DEV >>> {:?}", other);
				}
			}

			return;
		}
	}
}

// #[derive(Debug)]
pub struct DevWindow {
	window: Arc<Window>,
	egui_ctx: egui::Context,
	egui_state: EguiState,
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	renderer: egui_wgpu::Renderer,
	config: wgpu::SurfaceConfiguration,
	top_tab: DevTopTab,
	side_tab: DevSideTab,
	state: DaemonState,
	occluded: bool,
	pending_textures: egui::TexturesDelta,
}

impl DevWindow {
	pub fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
		let monitor = event_loop
			.primary_monitor()
			.ok_or_else(|| anyhow::anyhow!("no primary monitor found"))?;
		let monitor_size = monitor.size();
		let width = 900;
		let height = 600;
		let x = (monitor_size.width.saturating_sub(width)) / 2;
		let y = (monitor_size.height.saturating_sub(height)) / 2;
		let window = Arc::new(
			event_loop.create_window(
				WindowAttributes::default()
					.with_title("Estate — Dev")
					.with_inner_size(PhysicalSize::new(900, 600))
					.with_resizable(true),
			)?,
		);
		window.set_visible(true);
		window.focus_window();
		// window.focus_window();
		// // let window = Arc::new(
		// // 	event_loop.create_window(
		// // 		WindowAttributes::default()
		// // 			.with_title("Estate — Dev")
		// // 			.with_inner_size(PhysicalSize::new(900, 600))
		// // 			.with_resizable(true)
		// // 			.with_visible(true),
		// // 	)?,
		// // );
		// window.set_visible(true);
		// window.focus_window();
		// window.request_redraw();
		// window.with_current_subscriber();
		// window.with_subscriber(subscriber);
		// window.request_redraw();
		// dev_window.draw()?;
		tracing::info!(
			"window after setup: visible={} focused={} minimized={} size={:?}",
			window.is_visible().unwrap_or(false),
			window.has_focus(),
			window.is_minimized().unwrap_or(false),
			window.inner_size(),
		);
		let egui_ctx = egui::Context::default();
		let egui_state = EguiState::new(
			egui_ctx.clone(),
			egui::ViewportId::ROOT,
			event_loop,
			None,
			None,
			None,
		);
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
		let surface = unsafe { instance.create_surface(window.clone())? };
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
		let size = window.inner_size();
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
			color_space: SurfaceColorSpace::Auto,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			width: size.width.max(1),
			height: size.height.max(1),
			present_mode,
			alpha_mode,
			view_formats: vec![],
			desired_maximum_frame_latency: 2,
		};
		surface.configure(&device, &config);
		let renderer = Renderer::new(&device, format, egui_wgpu::RendererOptions::default());
		Ok(Self {
			window,
			egui_ctx,
			egui_state,
			config,
			surface,
			device,
			queue,
			renderer,
			occluded: true,
			pending_textures: TexturesDelta::default(),
			top_tab: DevTopTab::Status,
			side_tab: DevSideTab::Overview,
			state: DaemonState::load(),
		})
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
	pub fn draw(&mut self) -> anyhow::Result<()> {
		let raw_input = self.egui_state.take_egui_input(&self.window);

		self.egui_ctx.begin_pass(raw_input);

		let mut ui = egui::Ui::new(
			self.egui_ctx.clone(),
			egui::Id::new("dev_root"),
			egui::UiBuilder::new(),
		);

		egui::CentralPanel::default().show(&mut ui, |ui| {
			ui.heading("🔥 ESTATE DEV WINDOW");
			ui.label("If you see this, egui + wgpu is working.");

			ui.separator();

			ui.horizontal(|ui| {
				ui.label("Status:");
				ui.colored_label(egui::Color32::GREEN, "CONNECTED");
			});

			ui.add_space(20.0);

			if ui.button("TEST BUTTON").clicked() {
				tracing::info!("TEST BUTTON CLICKED");
			}

			ui.add_space(20.0);

			let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 100.0), egui::Sense::hover());

			ui.painter()
				.rect_filled(rect, 10.0, egui::Color32::from_rgb(255, 0, 0));

			ui.painter().text(
				rect.center(),
				egui::Align2::CENTER_CENTER,
				"HELLO GPU",
				egui::FontId::proportional(24.0),
				egui::Color32::WHITE,
			);
		});

		let egui::FullOutput {
			platform_output: _,
			textures_delta,
			shapes,
			pixels_per_point,
			viewport_output: _,
			..
		} = self.egui_ctx.end_pass();

		self.pending_textures.append(textures_delta);

		tracing::info!(
			"egui output: shapes={} texture_sets={} texture_frees={} pixels_per_point={}",
			shapes.len(),
			self.pending_textures.set.len(),
			self.pending_textures.free.len(),
			pixels_per_point,
		);


		let surface_texture = match self.surface.get_current_texture() {
			wgpu::CurrentSurfaceTexture::Success(texture)
			| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
				tracing::info!("SURFACE ACQUIRED");
				texture
			}
			wgpu::CurrentSurfaceTexture::Occluded => {
				tracing::warn!("SURFACE OCCLUDED");
				return Ok(());
			}
			wgpu::CurrentSurfaceTexture::Timeout => {
				return Ok(());
			}
			wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
				self.reconfigure_surface();
				return Ok(());
			}
			wgpu::CurrentSurfaceTexture::Validation => {
				return Err(anyhow::anyhow!("surface validation error"));
			}
		};

		let view = surface_texture
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);
		tracing::info!("EGUI: {} clipped primitives", clipped_primitives.len());
		for (i, primitive) in clipped_primitives.iter().enumerate() {
			match &primitive.primitive {
				egui::epaint::Primitive::Mesh(mesh) => {
					tracing::info!(
						"EGUI primitive {}: vertices={} indices={} clip_rect={:?}",
						i,
						mesh.vertices.len(),
						mesh.indices.len(),
						primitive.clip_rect,
					);
				}
				egui::epaint::Primitive::Callback(_) => {
					tracing::info!("EGUI primitive {}: CALLBACK", i);
				}
			}
		}
		let screen_descriptor = egui_wgpu::ScreenDescriptor {
			size_in_pixels: [
				self.window.inner_size().width,
				self.window.inner_size().height,
			],
			pixels_per_point,
		};
		for (id, image_deltas) in &self.pending_textures.set {
			for image_delta in image_deltas {
				self
					.renderer
					.update_texture(&self.device, &self.queue, *id, image_delta);
			}
		}

		self.pending_textures.clear();

		// Create ONE encoder.
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("egui-render"),
			});

		// Prepare egui buffers.
		self.renderer.update_buffers(
			&self.device,
			&self.queue,
			&mut encoder,
			&clipped_primitives,
			&screen_descriptor,
		);

		tracing::info!("EGUI: {} clipped primitives", clipped_primitives.len());

		{
			let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("egui-render-pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
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

			tracing::info!("EGUI: BEFORE renderer.render");

			self
				.renderer
				.render(&mut render_pass, &clipped_primitives, &screen_descriptor);

			tracing::info!("EGUI: AFTER renderer.render");
		}
		// Submit GPU commands.
		self.queue.submit(Some(encoder.finish()));
		// Present THIS surface texture.
		self.queue.present(surface_texture);

		Ok(())
	}

	// pub fn draw2(&mut self) -> anyhow::Result<()> {
	// 	tracing::info!("draw: START");

	// 	// ------------------------------------------------------------
	// 	// Input
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: taking egui input");
	// 	let raw_input = self.egui_state.take_egui_input(&self.window);
	// 	tracing::info!("draw: got egui input");

	// 	// ------------------------------------------------------------
	// 	// Begin egui pass
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: begin_pass");
	// 	self.egui_ctx.begin_pass(raw_input);
	// 	tracing::info!("draw: begin_pass complete");

	// 	// ------------------------------------------------------------
	// 	// Build UI
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: creating root UI");
	// 	let mut ui = egui::Ui::new(
	// 		self.egui_ctx.clone(),
	// 		egui::Id::new("dev_root"),
	// 		egui::UiBuilder::new(),
	// 	);

	// 	tracing::info!("draw: building UI");

	// 	egui::CentralPanel::default().show(&mut ui, |ui| {
	// 		ui.heading("🔥 ESTATE DEV WINDOW");
	// 		ui.label("If you see this, egui + wgpu is working.");

	// 		ui.separator();

	// 		ui.horizontal(|ui| {
	// 			ui.label("Status:");
	// 			ui.colored_label(egui::Color32::GREEN, "CONNECTED");
	// 		});

	// 		ui.add_space(20.0);

	// 		if ui.button("TEST BUTTON").clicked() {
	// 			tracing::info!("TEST BUTTON CLICKED");
	// 		}

	// 		ui.add_space(20.0);

	// 		let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 100.0), egui::Sense::hover());

	// 		ui.painter()
	// 			.rect_filled(rect, 10.0, egui::Color32::from_rgb(255, 0, 0));

	// 		ui.painter().text(
	// 			rect.center(),
	// 			egui::Align2::CENTER_CENTER,
	// 			"HELLO GPU",
	// 			egui::FontId::proportional(24.0),
	// 			egui::Color32::WHITE,
	// 		);
	// 	});

	// 	tracing::info!("draw: UI built");
	// 	// self.ui_counter(&mut ui, &mut 42i32);
	// 	// self.draw_ui(&mut ui);

	// 	tracing::info!("draw: draw_ui complete");

	// 	// ------------------------------------------------------------
	// 	// End egui pass
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: end_pass");

	// 	let mut full_output = self.egui_ctx.end_pass();

	// 	tracing::info!(
	// 		"draw: end_pass complete shapes={} texture_sets={} texture_frees={}",
	// 		full_output.shapes.len(),
	// 		full_output.textures_delta.set.len(),
	// 		full_output.textures_delta.free.len(),
	// 	);

	// 	// ------------------------------------------------------------
	// 	// Tessellate
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: tessellating");

	// 	let clipped_primitives = self
	// 		.egui_ctx
	// 		.tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

	// 	tracing::info!(
	// 		"draw: tessellate complete primitives={}",
	// 		clipped_primitives.len()
	// 	);

	// 	// ------------------------------------------------------------
	// 	// Texture uploads
	// 	// ------------------------------------------------------------

	// 	tracing::info!(
	// 		"draw: uploading {} texture sets",
	// 		full_output.textures_delta.set.len()
	// 	);

	// 	for (id, image_deltas) in &full_output.textures_delta.set {
	// 		tracing::info!("draw: texture id={:?}, deltas={}", id, image_deltas.len());

	// 		for (index, image_delta) in image_deltas.iter().enumerate() {
	// 			tracing::info!("draw: updating texture id={:?}, delta={}", id, index);

	// 			self
	// 				.renderer
	// 				.update_texture(&self.device, &self.queue, *id, image_delta);

	// 			tracing::info!("draw: updated texture id={:?}, delta={}", id, index);
	// 		}
	// 	}

	// 	tracing::info!("draw: texture uploads complete");

	// 	// ------------------------------------------------------------
	// 	// Surface
	// 	// ----------------------------
	// 	tracing::info!("draw: BEFORE get_current_texture");
	// 	// let surface_texture = match self.surface.get_current_texture() {
	// 	// 	wgpu::CurrentSurfaceTexture::Success(texture)
	// 	// 	| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,

	// 	// 	wgpu::CurrentSurfaceTexture::Timeout => {
	// 	// 		tracing::info!("draw: surface = Timeout");
	// 	// 		full_output.textures_delta.clear();
	// 	// 		return Ok(());
	// 	// 	}

	// 	// 	wgpu::CurrentSurfaceTexture::Occluded => {
	// 	// 		tracing::info!("draw: surface = Occluded");
	// 	// 		full_output.textures_delta.clear();
	// 	// 		return Ok(());
	// 	// 	}

	// 	// 	wgpu::CurrentSurfaceTexture::Outdated => {
	// 	// 		tracing::info!("draw: surface = Outdated");
	// 	// 		full_output.textures_delta.clear();
	// 	// 		self.reconfigure_surface();
	// 	// 		return Ok(());
	// 	// 	}

	// 	// 	wgpu::CurrentSurfaceTexture::Lost => {
	// 	// 		tracing::info!("draw: surface = Lost");
	// 	// 		full_output.textures_delta.clear();
	// 	// 		self.reconfigure_surface();
	// 	// 		return Ok(());
	// 	// 	}

	// 	// 	wgpu::CurrentSurfaceTexture::Validation => {
	// 	// 		tracing::error!("draw: surface = Validation");
	// 	// 		full_output.textures_delta.clear();
	// 	// 		return Err(anyhow::anyhow!("surface validation error"));
	// 	// 	}
	// 	// };
	// 	let surface_texture = match self.surface.get_current_texture() {
	// 		wgpu::CurrentSurfaceTexture::Success(texture)
	// 		| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,

	// 		wgpu::CurrentSurfaceTexture::Timeout => {
	// 			return Ok(());
	// 		}

	// 		wgpu::CurrentSurfaceTexture::Occluded => {
	// 			tracing::warn!("draw skipped: surface is occluded");
	// 			return Ok(());
	// 		}

	// 		wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
	// 			self.reconfigure_surface();
	// 			return Ok(());
	// 		}

	// 		wgpu::CurrentSurfaceTexture::Validation => {
	// 			return Err(anyhow::anyhow!("surface validation error"));
	// 		}
	// 	};
	// 	tracing::info!("draw: SURFACE TEXTURE ACQUIRED");

	// 	// ------------------------------------------------------------
	// 	// Texture view
	// 	// ------------------------------------------------------------

	// 	tracing::info!("draw: creating texture view");
	// 	let view = surface_texture
	// 		.texture
	// 		.create_view(&wgpu::TextureViewDescriptor::default());

	// 	let view = surface_texture
	// 		.texture
	// 		.create_view(&wgpu::TextureViewDescriptor::default());
	// 	// ------------------------------------------------------------
	// 	// Command encoder
	// 	// ------------------------------------------------------------
	// 	tracing::info!("draw: creating command encoder");

	// 	let mut encoder = self
	// 		.device
	// 		.create_command_encoder(&wgpu::CommandEncoderDescriptor {
	// 			label: Some("estate-dev-egui-encoder"),
	// 		});

	// 	tracing::info!("draw: command encoder created");

	// 	tracing::info!("draw: update_buffers");
	// 	let screen_descriptor = egui_wgpu::ScreenDescriptor {
	// 		size_in_pixels: [
	// 			self.window.inner_size().width,
	// 			self.window.inner_size().height,
	// 		],
	// 		pixels_per_point: full_output.pixels_per_point,
	// 	};
	// 	self.renderer.update_buffers(
	// 		&self.device,
	// 		&self.queue,
	// 		&mut encoder,
	// 		&clipped_primitives,
	// 		&screen_descriptor,
	// 	);

	// 	tracing::info!("draw: update_buffers complete");

	// 	tracing::info!("draw: creating render pass");

	// 	let color_attachments = [Some(wgpu::RenderPassColorAttachment {
	// 		view: &view,
	// 		depth_slice: None,
	// 		resolve_target: None,
	// 		ops: wgpu::Operations {
	// 			load: wgpu::LoadOp::Clear(wgpu::Color {
	// 				r: 0.05,
	// 				g: 0.05,
	// 				b: 0.05,
	// 				a: 1.0,
	// 			}),
	// 			store: wgpu::StoreOp::Store,
	// 		},
	// 	})];

	// 	let render_pass_descriptor = wgpu::RenderPassDescriptor {
	// 		label: Some("estate-dev-egui-render-pass"),
	// 		color_attachments: &color_attachments,
	// 		depth_stencil_attachment: None,
	// 		timestamp_writes: None,
	// 		occlusion_query_set: None,
	// 		multiview_mask: None,
	// 	};

	// 	let render_pass = encoder.begin_render_pass(&render_pass_descriptor);
	// 	let mut render_pass = render_pass.forget_lifetime();

	// 	tracing::info!("draw: render pass created");

	// 	tracing::info!("draw: calling renderer.render");

	// 	self
	// 		.renderer
	// 		.render(&mut render_pass, &clipped_primitives, &screen_descriptor);

	// 	tracing::info!("draw: renderer.render complete");

	// 	drop(render_pass);
	// 	tracing::info!("draw: finishing encoder");
	// 	let command_buffer = encoder.finish();
	// 	tracing::info!("draw: encoder finished");
	// 	self.queue.submit(Some(command_buffer));
	// 	tracing::info!("draw: queue submitted");
	// 	self.window.request_redraw();
	// 	tracing::info!("draw: redraw requested");
	// 	Ok(())
	// }

	fn draw_ui(&mut self, ui: &mut egui::Ui) {
		// Top tab bar
		ui.horizontal(|ui| {
			ui.selectable_value(&mut self.top_tab, DevTopTab::Status, "Status");

			ui.selectable_value(&mut self.top_tab, DevTopTab::Tasks, "Tasks");

			ui.selectable_value(&mut self.top_tab, DevTopTab::Logs, "Logs");

			ui.selectable_value(&mut self.top_tab, DevTopTab::Config, "Config");
		});

		ui.separator();

		// Main area
		ui.horizontal(|ui| {
			// Sidebar
			ui.allocate_ui_with_layout(
				egui::vec2(180.0, ui.available_height()),
				egui::Layout::top_down(egui::Align::LEFT),
				|ui| {
					ui.heading("Estate");
					ui.separator();

					ui.selectable_value(&mut self.side_tab, DevSideTab::Overview, "Overview");

					ui.selectable_value(&mut self.side_tab, DevSideTab::Daemon, "Daemon");

					ui.selectable_value(&mut self.side_tab, DevSideTab::Engine, "Engine");

					ui.selectable_value(&mut self.side_tab, DevSideTab::Workspace, "Workspace");

					ui.selectable_value(&mut self.side_tab, DevSideTab::Runtime, "Runtime");
				},
			);

			ui.separator();

			// Main content
			ui.vertical(|ui| {
				self.draw_content(ui);
			});
		});
	}
	fn ui_counter(&mut self, ui: &mut egui::Ui, counter: &mut i32) {
		ui.horizontal(|ui| {
			if ui.button("−").clicked() {
				*counter -= 1;
			}

			ui.label(counter.to_string());

			if ui.button("+").clicked() {
				*counter += 1;
			}
		});
	}
	fn draw_content(&mut self, ui: &mut egui::Ui) {
		match self.top_tab {
			DevTopTab::Status => self.draw_status(ui),
			DevTopTab::Tasks => self.draw_tasks(ui),
			DevTopTab::Logs => self.draw_logs(ui),
			DevTopTab::Config => self.draw_config(ui),
		}
	}
	fn resize(&mut self, width: u32, height: u32) {
		if width == 0 || height == 0 {
			return;
		}

		tracing::info!("DevWindow resize: {}x{}", width, height);

		self.config.width = width;
		self.config.height = height;

		self.surface.configure(&self.device, &self.config);
	}
	fn draw_status(&self, ui: &mut egui::Ui) {
		ui.heading("Status");
		ui.separator();

		match self.side_tab {
			DevSideTab::Overview => {
				ui.label("Estate Daemon");
				ui.label("Status: Running");

				ui.separator();

				ui.label(format!("Starts: {}", self.state.starts));
				ui.label(format!("Status checks: {}", self.state.status_checks));
				ui.label(format!("Started at: {}", self.state.started_at));
				ui.label(format!("Longest run: {}s", self.state.longest_run));
			}

			DevSideTab::Daemon => {
				ui.heading("Daemon");
				ui.label("Running");
				ui.label(format!("Starts: {}", self.state.starts));
			}

			DevSideTab::Engine => {
				ui.heading("Engine");
				ui.label("Engine information placeholder");
			}

			DevSideTab::Workspace => {
				ui.heading("Workspace");
				ui.label("Workspace information placeholder");
			}

			DevSideTab::Runtime => {
				ui.heading("Runtime");
				ui.label("Runtime information placeholder");
			}
		}
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
// Command = "What does this executable invocation mean?"
// DaemonCommand = "What should happen to the running daemon?"
// ActionRequest = "What work should the daemon perform?"
//
// Things that can happen to the daemon itself
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevSideTab {
	Overview,
	Daemon,
	Engine,
	Workspace,
	Runtime,
}

// pub struct WorkspaceContext {
// 	pub root: PathBuf,
// 	pub estate: Option<PathBuf>,
// }

// pub struct RuntimeContext {
// 	pub engine_dir: PathBuf,
// 	pub connected: bool,
// }
