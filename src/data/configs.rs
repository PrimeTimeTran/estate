use std::sync::{Mutex, OnceLock, atomic::AtomicBool};

use core_graphics::display::{CGPoint, CGRect};
use egui::Color32;

use crate::{
	native::prelude::{ScrollRedirectState, *},
	ui::PanelConfig,
};
pub static ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static HMR_CHART_JSON: &str = "/Users/future/kb/project/crates/estate/src/data/chart.json";
pub static HOME_DIR: &str = ".config/estate";
pub static INDEX_PATH: &str = ".config/estate/master.json";
pub static INTRINSIC_FILES: [&str; 3] = ["default.settings.json", "settings.json", "key-map.json"];
pub static PID_PATH: &str = "/tmp/estate-daemon.pid";
pub static PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub static PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";
pub static SCHEMA_VERSION: u32 = 1;
pub static SERVER_URL: &str = "http://localhost:50051";
pub static SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub static STATE_PATH: &str = "/Users/future/Library/Application Support/estate/state.json";
pub static TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";
pub static WORKSPACE_SETTINGS: &str = ".estate/settings.json";

// pub static INITIAL_WINDOW: WindowType = WindowType::Dashboard;
// pub static INITIAL_WINDOW: WindowType = WindowType::TelemetryInspector;
pub static INITIAL_WINDOW: WindowType = WindowType::TaskManager;
// pub static INITIAL_WINDOW: WindowType = WindowType::EguiVeable;
// pub static INITIAL_WINDOW: WindowType = WindowType::WaterfallChart;
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
use std::sync::atomic::AtomicU64;

pub static EVENT_ID: AtomicU64 = AtomicU64::new(1);
pub static FILE_EXTENSIONS: &[&str] = &[
	"rs", "loi", "estate", "html", "htm", "css", "js", "jsx", "ts", "tsx", "json", "jsonc", "md",
	"mdx", "txt", "toml", "yaml", "yml", "ini", "conf", "sh", "bash", "zsh", "c", "h", "cpp", "hpp",
	"py", "go", "java", "kt", "png", "jpg", "jpeg", "svg", "webp", "ico", "csv", "xml", "sql",
];
pub static FILE_NAMES: &[&str] = &[
	"Dockerfile",
	"Makefile",
	"LICENSE",
	"README",
	"README.md",
	"Cargo.toml",
	"package.json",
];
