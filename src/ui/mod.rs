use crate::{prelude::*, ui::prelude as gui};

pub mod chart;
pub mod config;
pub mod layout;
pub mod panel;
#[path = "./[prelude].rs"]
pub mod prelude;
pub mod primitive;
pub mod region;
pub mod screen;
pub mod theme;
pub mod view;

pub use crate::ui::prelude::*;

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));

pub struct AppWindow<C, S>
where
	C: Ctx,
{
	// pub runtime: NativeRuntime,
	pub kind: WindowType,
	pub view: ViewType,
	pub window: Window<C, S>,
}

pub struct Window<C, S>
where
	C: Ctx,
{
	pub screen: ScreenInstance<C, S>,
	// This Surface contains/borrows something that is guaranteed to be valid for the 'static lifetime.
	pub surface: gui::wgpu::Surface<'static>,
	pub config: gui::wgpu::SurfaceConfiguration,
	pub device: wgpu::Device,
	pub gui_ctx: gui::Context,
	pub gui_state: egui_winit::State,
	pub instance: Arc<winit::window::Window>,
	pub kind: WindowType,
	pub needs_resize: bool,
	pub occluded: bool,
	pub queue: wgpu::Queue,
	pub renderer: egui_wgpu::Renderer,
}
