use crate::theme::palette;

pub(crate) mod chart;
pub(crate) mod components;

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const DEFAULT_CONFIG: VeConfig = VeConfig {
	bg: palette::BG,
	surface: palette::SURFACE,
	activity_bar: PanelConfig::new(true, 48.0),
	primary_bar: PanelConfig::new(true, 40.0),
	secondary_bar: PanelConfig::new(true, 48.0),
	bottom_panel: PanelConfig::new(false, 0.0),
	status_bar: PanelConfig::new(false, 24.0),
	dock_left: PanelConfig::new(false, 280.0),
	dock_right: PanelConfig::new(false, 320.0),
};

pub struct VeConfig {
	pub bg: egui::Color32,
	pub surface: egui::Color32,

	pub activity_bar: PanelConfig,
	pub dock_left: PanelConfig,
	pub primary_bar: PanelConfig,
	pub secondary_bar: PanelConfig,
	pub status_bar: PanelConfig,
	pub dock_right: PanelConfig,
	pub bottom_panel: PanelConfig,
}

pub struct PanelConfig {
	pub active: bool,
	pub size: f32,
	pub resizable: bool,
	pub docked: bool,
}
impl PanelConfig {
	pub const fn new(active: bool, size: f32) -> Self {
		Self {
			active,
			size,
			resizable: true,
			docked: true,
		}
	}
}

#[derive(Clone, Copy)]
pub enum ResizeEdge {
	Left,
	Right,
	Top,
	Bottom,
}
