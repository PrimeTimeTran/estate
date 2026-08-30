use std::sync::{Mutex, OnceLock, atomic::AtomicBool};

use core_graphics::display::{CGPoint, CGRect};
use egui::Color32;

use crate::{
	native::prelude::{ScrollRedirectState, *},
	ui::PanelConfig,
};

pub static INDEX_PATH: &str = ".config/estate/master.json";
pub static HOME_DIR: &str = ".config/estate";
pub const WORKSPACE_SETTINGS: &str = ".estate/settings.json";
pub static HMR_CHART_JSON: &str = "/Users/future/kb/project/crates/estate/src/data/chart.json";
pub const INTRINSIC_FILES: [&str; 3] = ["default.settings.json", "settings.json", "key-map.json"];

// pub static INITIAL_WINDOW: WindowType = WindowType::Dashboard;
// pub static INITIAL_WINDOW: WindowType = WindowType::TelemetryInspector;
// pub static INITIAL_WINDOW: WindowType = WindowType::TaskManager;
pub static INITIAL_WINDOW: WindowType = WindowType::WaterfallChart;
// pub static INITIAL_WINDOW: WindowType = WindowType::MarkdownView;

pub(crate) struct VeConfig {
	pub bg: Color32,
	pub surface: Color32,
	pub activity_bar: PanelConfig,
	pub primary_bar: PanelConfig,
	pub secondary_bar: PanelConfig,
	pub bottom_panel: PanelConfig,
	pub status_bar: PanelConfig,
	pub dock_left: PanelConfig,
	pub dock_right: PanelConfig,
}

pub(crate) const DEFAULT_CONFIG: VeConfig = VeConfig {
	bg: palette::BG,
	surface: palette::SURFACE,
	activity_bar: PanelConfig::new(true, 48.0),
	primary_bar: PanelConfig::new(true, 40.0),
	secondary_bar: PanelConfig::new(true, 48.0),
	bottom_panel: PanelConfig::new(true, 0.0),
	status_bar: PanelConfig::new(true, 24.0),
	dock_left: PanelConfig::new(true, 280.0),
	dock_right: PanelConfig::new(true, 320.0),
};
