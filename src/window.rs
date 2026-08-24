use crate::prelude::*;
use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};
use tray_icon::menu::{MenuItem, Submenu};
use winit::{
	dpi::{PhysicalPosition, PhysicalSize},
	event_loop::ActiveEventLoop,
};
use egui::{Context as EguiContext, TexturesDelta};
use egui_wgpu::{
	Renderer,
	wgpu::{self},
};
use egui_winit::State as EguiState;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use wgpu::{Adapter, Device, SurfaceColorSpace};
pub struct TrayMenu {
	pub clear_tasks: MenuItem,
	pub dev: MenuItem,
	pub list_tasks: MenuItem,
	pub new_task: MenuItem,
	pub quit: MenuItem,
	pub status: MenuItem,
	pub task_manager: MenuItem,
	pub tasks: Submenu,
	pub telemetry: MenuItem,
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
	pub instance: Arc<winit::window::Window>,
	pub egui_ctx: egui::Context,
	pub egui_state: EguiState,
	pub config: wgpu::SurfaceConfiguration,
	pub occluded: bool,
	pub device: wgpu::Device,
	pub needs_resize: bool,
	pub surface: wgpu::Surface<'static>,
	queue: wgpu::Queue,
	renderer: egui_wgpu::Renderer,
	pending_textures: TexturesDelta,
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
			instance: window,
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
		let input = self.egui_state.take_egui_input(&self.instance);
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
			| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(Some(texture)),
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
		let size = self.instance.inner_size();
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
				self.instance.inner_size().width,
				self.instance.inner_size().height,
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
pub struct AppWindow {
	pub kind: AppWindowType,
	pub window: Window,
}
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
pub struct GlobalHotkeys {
	hotkey_id: u32,
	manager: GlobalHotKeyManager,
	shutdown: Arc<AtomicBool>,
}
impl GlobalHotkeys {
	pub fn new() -> anyhow::Result<Self> {
		let manager = GlobalHotKeyManager::new()?;
		let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);
		let hotkey_id = hotkey.id();
		manager.register(hotkey)?;
		Ok(Self {
			manager,
			hotkey_id,
			shutdown: Arc::new(AtomicBool::new(false)),
		})
	}
	pub fn start(&self) {
		let shutdown = Arc::clone(&self.shutdown);
		let hotkey_id = self.hotkey_id;
		std::thread::spawn(move || {
			let receiver = GlobalHotKeyEvent::receiver();
			while !shutdown.load(Ordering::Relaxed) {
				if let Ok(event) = receiver.recv() {
					if event.id == hotkey_id && event.state == global_hotkey::HotKeyState::Pressed {
						move_cursor_to(ScreenPosition::Left);
					}
				}
			}
		});
	}
	pub fn shutdown(&self) {
		self.shutdown.store(true, Ordering::Relaxed);
	}
}