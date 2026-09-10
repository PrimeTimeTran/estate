pub mod chart;
pub mod config;
pub mod layout;
pub mod panel;
pub mod primitive;
pub mod region;
pub mod screen;
pub mod theme;
pub mod ui_prelude;
pub mod ui_trait;
pub mod view;

pub use crate::ui::ui_prelude::*;

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));

// use crate::{
// 	doc,
// 	prelude::*,
// 	prelude::{traits::Ctx, *},
// 	ui, ui_prelude as gui,
// };
// use anyhow::anyhow;

// pub struct AppWindow {
// 	// pub runtime: NativeRuntime,
// 	pub kind: WindowType,
// 	pub view: ViewType,
// 	pub window: Window,
// }

// pub struct Window {
// 	#[cfg(all(feature = "web", target_arch = "wasm32"))]
// 	screen: crate::ScreenInstance<WebRuntime, WebExecutor>,
// 	#[cfg(not(target_arch = "wasm32"))]
// 	screen: crate::ScreenInstance<NativeRuntime, NativeExecutor>,
// 	// This Surface contains/borrows something that is guaranteed to be valid for the 'static lifetime.
// 	pub surface: gui::wgpu::Surface<'static>,
// 	pub config: gui::wgpu::SurfaceConfiguration,
// 	pub device: wgpu::Device,
// 	pub gui_ctx: gui::Context,
// 	pub gui_state: egui_winit::State,
// 	pub instance: Arc<winit::window::Window>,
// 	pub kind: WindowType,
// 	pub needs_resize: bool,
// 	pub occluded: bool,

// 	pending_textures: gui::TexturesDelta,
// 	queue: wgpu::Queue,
// 	renderer: gui::Renderer,
// }

// impl Window {
// 	pub fn new(event_loop: &ActiveEventLoop, view: ViewType) -> Result<Self> {
// 		let (gui_ctx, gui_state) = build_egui(event_loop);
// 		let (window, instance, surface) = create_gpu_surface(event_loop)?;
// 		let (adapter, device, queue) = initialize_gpu(&instance, &surface)?;
// 		let size = window.inner_size();
// 		let (config, renderer) = build_renderer(&surface, adapter, &device, size)?;
// 		Ok(Self {
// 			config,
// 			device,
// 			gui_ctx,
// 			gui_state,
// 			instance: window,
// 			kind: WindowType::MarkdownScreen,
// 			needs_resize: false,
// 			occluded: true,
// 			pending_textures: gui::TexturesDelta::default(),
// 			queue,
// 			renderer,
// 			screen: crate::ScreenInstance::new(view),
// 			surface,
// 		})
// 	}

// 	pub fn set_view(&mut self, kind: WindowType)
// 	// V: Veable<NativeRuntime> + 'static,
// 	{
// 		self.kind = kind;
// 		// self.view = Ve::new(view);
// 	}

// 	pub fn draw(&mut self, ctx: &mut AppContext<'_, NativeRuntime, NativeExecutor>) -> Result<()> {
// 		self.begin_egui();
// 		let output = self.build_ui(ctx);
// 		let Some(surface_texture) = self.acquire_surface()? else {
// 			tracing::warn!("NO SURFACE");
// 			return Ok(());
// 		};
// 		self.render_egui(surface_texture, output)?;
// 		Ok(())
// 	}
// 	fn build_ui(
// 		&mut self,
// 		ctx: &mut AppContext<'_, NativeRuntime, NativeExecutor>,
// 	) -> gui::FullOutput {
// 		// tracing::info!("Window::build_ui");
// 		let mut ui = gui::Ui::new(
// 			self.gui_ctx.clone(),
// 			gui::Id::new("window_root"),
// 			gui::UiBuilder::new(),
// 		);

// 		gui::Frame::NONE.show(&mut ui, |ui| {
// 			// tracing::info!("Window → ScreenInstance::draw");
// 			self.screen.draw(ui, ctx);
// 		});

// 		self.gui_ctx.end_pass()
// 	}
// 	fn begin_egui(&mut self) {
// 		let input = self.gui_state.take_egui_input(&self.instance);
// 		self.gui_ctx.begin_pass(input);
// 	}
// 	fn acquire_surface(&mut self) -> Result<Option<wgpu::SurfaceTexture>> {
// 		match self.surface.get_current_texture() {
// 			wgpu::CurrentSurfaceTexture::Success(texture)
// 			| wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Ok(Some(texture)),
// 			wgpu::CurrentSurfaceTexture::Occluded => {
// 				// tracing::warn!("SURFACE OCCLUDED");
// 				Ok(None)
// 			}
// 			wgpu::CurrentSurfaceTexture::Timeout => Ok(None),
// 			wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
// 				self.reconfigure_surface();
// 				Ok(None)
// 			}
// 			wgpu::CurrentSurfaceTexture::Validation => Err(anyhow::anyhow!("surface validation error")),
// 		}
// 	}
// 	fn reconfigure_surface(&mut self) {
// 		let size = self.instance.inner_size();
// 		if size.width == 0 || size.height == 0 {
// 			return;
// 		}
// 		self.config.width = size.width;
// 		self.config.height = size.height;
// 		self.surface.configure(&self.device, &self.config);
// 	}
// 	fn render_egui(
// 		&mut self,
// 		surface_texture: wgpu::SurfaceTexture,
// 		output: gui::FullOutput,
// 	) -> Result<()> {
// 		let gui::FullOutput {
// 			pixels_per_point,
// 			platform_output: _,
// 			shapes,
// 			textures_delta,
// 			viewport_output: _,
// 			..
// 		} = output;
// 		self.pending_textures.append(textures_delta);
// 		let view = surface_texture
// 			.texture
// 			.create_view(&wgpu::TextureViewDescriptor::default());
// 		let clipped_primitives = self.gui_ctx.tessellate(shapes, pixels_per_point);
// 		let screen_descriptor = egui_wgpu::ScreenDescriptor {
// 			size_in_pixels: [
// 				self.instance.inner_size().width,
// 				self.instance.inner_size().height,
// 			],
// 			pixels_per_point,
// 		};
// 		self.upload_textures();
// 		let mut encoder = self.device.create_command_encoder(
// 			&(wgpu::CommandEncoderDescriptor {
// 				label: Some("egui-render"),
// 			}),
// 		);
// 		self.renderer.update_buffers(
// 			&self.device,
// 			&self.queue,
// 			&mut encoder,
// 			&clipped_primitives,
// 			&screen_descriptor,
// 		);
// 		self.render_pass(&mut encoder, &view, &clipped_primitives, &screen_descriptor);
// 		self.queue.submit(Some(encoder.finish()));
// 		self.queue.present(surface_texture);
// 		Ok(())
// 	}
// 	fn upload_textures(&mut self) {
// 		for (id, image_deltas) in &self.pending_textures.set {
// 			for image_delta in image_deltas {
// 				self
// 					.renderer
// 					.update_texture(&self.device, &self.queue, *id, image_delta);
// 			}
// 		}
// 		self.pending_textures.clear();
// 	}
// 	fn render_pass(
// 		&mut self,
// 		encoder: &mut wgpu::CommandEncoder,
// 		view: &wgpu::TextureView,
// 		primitives: &[gui::ClippedPrimitive],
// 		screen_descriptor: &egui_wgpu::ScreenDescriptor,
// 	) {
// 		let render_pass = encoder.begin_render_pass(
// 			&(wgpu::RenderPassDescriptor {
// 				label: Some("egui-render-pass"),
// 				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
// 					view,
// 					depth_slice: None,
// 					resolve_target: None,
// 					ops: wgpu::Operations {
// 						load: wgpu::LoadOp::Clear(wgpu::Color {
// 							r: 0.08,
// 							g: 0.08,
// 							b: 0.08,
// 							a: 1.0,
// 						}),
// 						store: wgpu::StoreOp::Store,
// 					},
// 				})],
// 				depth_stencil_attachment: None,
// 				timestamp_writes: None,
// 				occlusion_query_set: None,
// 				multiview_mask: None,
// 			}),
// 		);
// 		let mut render_pass = render_pass.forget_lifetime();
// 		self
// 			.renderer
// 			.render(&mut render_pass, primitives, screen_descriptor);
// 	}
// 	fn egui_view(&mut self, ctx: &egui::Context) {
// 		doc!(
// 			r#"
//      	Estate UI Container
//      	Owns the native window, egui state, wgpu rendering resources, and the
//      	application state required to render and interact with the development UI.
//      	CPU / Rust
//      	  │
//      	  │ create resources + record commands
//      	  ▼
//      	wgpu::Device
//      	  │
//      	  │ command encoder
//      	  ▼
//      	wgpu::Queue
//      	  │
//      	  │ submit
//      	  ▼
//      	┌─────────────────────────────────────────────┐
//      	│                 GPU PIPELINE                │
//      	│                                             │
//      	│ Vertex Input                                │
//      	│      ↓                                      │
//      	│ Vertex Shader                                │
//      	│      ↓                                      │
//      	│ Primitive Assembly                          │
//      	│      ↓                                      │
//      	│ Rasterization                               │
//      	│      ↓                                      │
//      	│ Fragment Shader                              │
//      	│      ↓                                      │
//      	│ Depth / Stencil / Blending                  │
//      	│      ↓                                      │
//      	│ Render Target                               │
//      	└─────────────────────────────────────────────┘
//      	  │
//      	  ▼
//      	Surface Texture
//      	  │
//      	  ▼
//      	Window
//      	Vertex data
//       	   ↓
//      	Vertex Shader
//       	   ↓
//      	Primitive assembly
//       	   ↓
//      	Rasterization
//       	   ↓
//      	Fragment Shader
//       	   ↓
//      	Depth / Stencil / Blending
//       	   ↓
//      	Color attachment

//      	1. input assembler
//      	2.vertex shader
//      	3.hull shader
//      	4. tesselator
//      	5.domain shader
//      	6.geometry shader
//      	7.rasterizer
//      	8.pixel shader
//      	9.output merger
//    	"#
// 		);
// 	}
// }
// impl Window {
// 	pub fn sync_view(&mut self, view: ViewType, api: Arc<NativeApiClient>) {
// 		if self.screen.kind != view {
// 			tracing::debug!("🖼️ Window view change: {:?} → {:?}", self.screen.kind, view);
// 			self.screen = crate::ScreenInstance::new(view);
// 		}
// 	}
// }
// fn initialize_gpu(
// 	instance: &wgpu::Instance,
// 	surface: &wgpu::Surface<'_>,
// ) -> Result<(gui::Adapter, gui::Device, wgpu::Queue)> {
// 	let adapter = pollster::block_on(instance.request_adapter(
// 		&(wgpu::RequestAdapterOptions {
// 			apply_limit_buckets: true,
// 			power_preference: wgpu::PowerPreference::HighPerformance,
// 			compatible_surface: Some(&surface),
// 			force_fallback_adapter: false,
// 		}),
// 	))
// 	.map_err(|e| anyhow!("failed to find suitable GPU adapter: {e}"))?;
// 	let (device, queue) = pollster::block_on(adapter.request_device(
// 		&(wgpu::DeviceDescriptor {
// 			experimental_features: wgpu::ExperimentalFeatures::disabled(),
// 			label: Some("estate-dev-device"),
// 			required_features: wgpu::Features::empty(),
// 			required_limits: wgpu::Limits::default(),
// 			memory_hints: wgpu::MemoryHints::Performance,
// 			trace: wgpu::Trace::Off,
// 		}),
// 	))?;
// 	Ok((adapter, device, queue))
// }
// fn create_gpu_surface(
// 	event_loop: &ActiveEventLoop,
// ) -> Result<(
// 	Arc<winit::window::Window>,
// 	wgpu::Instance,
// 	wgpu::Surface<'static>,
// )> {
// 	let window = build_window(event_loop)?;
// 	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
// 	let surface = { instance.create_surface(window.clone())? };
// 	Ok((window, instance, surface))
// }
// // WIP: Self Activating Select
// fn build_egui(event_loop: &ActiveEventLoop) -> (gui::Context, egui_winit::State) {
// 	let ctx = gui::Context::default();
// 	ctx.global_style_mut(|style| {
// 		style.interaction.selectable_labels = true;
// 		style.interaction.multi_widget_text_select = true;
// 		style.visuals.widgets.hovered = style.visuals.widgets.inactive.clone();
// 		style.visuals.widgets.active = style.visuals.widgets.inactive.clone();
// 	});
// 	// ctx.memory_mut(|memory| {
// 	// 	memory.surrender_focus();
// 	// });
// 	let state = egui_winit::State::new(
// 		ctx.clone(),
// 		gui::ViewportId::ROOT,
// 		event_loop,
// 		None,
// 		None,
// 		None,
// 	);
// 	(ctx, state)
// }
// fn build_window(event_loop: &ActiveEventLoop) -> Result<Arc<winit::window::Window>> {
// 	let width = 1920;
// 	let height = 1280;
// 	let icon_file = include_bytes!("../../assets/icon.png");
// 	let icon = {
// 		let image = image::load_from_memory(icon_file)
// 			.expect("failed to load icon")
// 			.into_rgba8();
// 		let (width, height) = image.dimensions();
// 		winit::window::Icon::from_rgba(image.into_raw(), width, height)?
// 	};
// 	let mut attrs = winit::window::Window::default_attributes()
// 		.with_title("Estate Dev")
// 		.with_inner_size(PhysicalSize::new(width, height))
// 		.with_window_icon(Some(icon));
// 	// .with_window_level(WindowLevel::AlwaysOnTop);
// 	// Calculate bottom-right screen coordinates if a monitor is available
// 	if let Some(monitor) = event_loop
// 		.primary_monitor()
// 		.or_else(|| event_loop.available_monitors().next())
// 	{
// 		let screen_size = monitor.size();
// 		let scale_factor = monitor.scale_factor();
// 		// Optional: leave a small margin (e.g., 40 pixels) away from the edge/dock
// 		// let margin_x = (40.0 * scale_factor) as i32;
// 		// let margin_y = (60.0 * scale_factor) as i32;
// 		// let x = screen_size.width as i32 - width as i32 - margin_x;
// 		// let y = screen_size.height as i32 - height as i32 - margin_y;
// 		let x = (screen_size.width as i32) - (width as i32);
// 		let y = (screen_size.height as i32) - (height as i32);
// 		attrs = attrs.with_position(PhysicalPosition::new(x.max(0), y.max(0)));
// 	} else {
// 		// Fallback position if no monitor info is found
// 		attrs = attrs.with_position(PhysicalPosition::new(100, 100));
// 	}
// 	let window = event_loop.create_window(attrs)?;
// 	// Force macOS to show a Dock icon and participate in Cmd + Tab
// 	#[cfg(target_os = "macos")]
// 	{
// 		{
// 			let mtm = MainThreadMarker::new().expect("must be on the main thread");
// 			let app = NSApplication::sharedApplication(mtm);
// 			app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
// 			app.activateIgnoringOtherApps(true);
// 		}
// 	}
// 	Ok(Arc::new(window))
// }
// fn build_renderer(
// 	surface: &wgpu::Surface<'_>,
// 	adapter: wgpu::Adapter,
// 	device: &wgpu::Device,
// 	size: PhysicalSize<u32>,
// ) -> Result<
// 	(
// 		wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
// 		gui::Renderer,
// 	),
// 	Error,
// > {
// 	let caps = surface.get_capabilities(&adapter);
// 	let format = caps
// 		.formats
// 		.iter()
// 		.copied()
// 		.find(|format| {
// 			matches!(
// 				format,
// 				wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
// 			)
// 		})
// 		.or_else(|| caps.formats.first().copied())
// 		.ok_or_else(|| anyhow!("GPU surface has no supported formats"))?;
// 	let present_mode = caps
// 		.present_modes
// 		.iter()
// 		.copied()
// 		.find(|mode| *mode == wgpu::PresentMode::Fifo)
// 		.unwrap_or(wgpu::PresentMode::Fifo);
// 	let alpha_mode = caps
// 		.alpha_modes
// 		.first()
// 		.copied()
// 		.ok_or_else(|| anyhow!("GPU surface has no alpha modes"))?;
// 	let config = wgpu::SurfaceConfiguration {
// 		format,
// 		alpha_mode,
// 		present_mode,
// 		view_formats: vec![],
// 		width: size.width.max(1),
// 		height: size.height.max(1),
// 		desired_maximum_frame_latency: 2,
// 		color_space: gui::SurfaceColorSpace::Auto,
// 		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
// 	};
// 	surface.configure(device, &config);
// 	let renderer = gui::Renderer::new(device, format, gui::RendererOptions::default());
// 	Ok((config, renderer))
// }
