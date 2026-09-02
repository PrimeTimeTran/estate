use crate::ui::{PanelState, *};

use egui::Color32;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicI64;

pub static ROOT_DIR: &str = "/Users/future/KB/project/crates/estate";
pub static HMR_CHART_JSON: &str = "/Users/future/kb/project/crates/estate/src/data/chart.json";
pub static MARKDOWN: &str = "/Users/future/kb/project/crates/estate/src/data/corpus.md";
pub static PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub static PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";
pub static TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";

use std::sync::LazyLock;

pub static START_APP_CLOCK: bool = true;
pub static START_WINDOW: WindowType = WindowType::ProblemScreen;
pub static START_VIEW: ViewType = ViewType::ProblemScreen;
// Unsafe territory
// pub static mut START_VIEW: ViewType = ...;
pub static DEFAULT_VIEW: ViewType = ViewType::ProblemScreen;
pub static ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GRPC_SOCKET_CLIENT: &str = "http://127.0.0.1:50051";
pub const GRPC_SOCKET: &str = "127.0.0.1:50051";
pub static GRPC_PROBLEMS_PATH: &str = "src/data/problems";
pub static GRPC_SUBMISSIONS_PATH: &str = "src/data/submissions";
pub static HOME_DIR: &str = ".config/estate";
pub static DEFAULT_PROBLEM: &str = "../data/problems/two-sum";
pub static INDEX_PATH: &str = ".config/estate/master.json";
pub static INTRINSIC_FILES: [&str; 3] = ["default.settings.json", "settings.json", "key-map.json"];
pub static PID_PATH: &str = "/tmp/estate-daemon.pid";
pub static NEXT_PROBLEM_ID: AtomicI64 = AtomicI64::new(1);
pub static SCHEMA_VERSION: u32 = 1;
pub static SERVER_URL: &str = "http://localhost:50051";
pub static SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub static STATE_PATH: &str = "/Users/future/Library/Application Support/estate/state.json";
pub static WORKSPACE_SETTINGS: &str = ".estate/settings.json";

// pub static START_WINDOW: WindowType = WindowType::Dashboard;
// pub static START_WINDOW: WindowType = WindowType::TelemetryInspector;
// pub static START_WINDOW: WindowType = WindowType::EguiVeable;
// pub static START_WINDOW: WindowType = WindowType::WaterfallChart;
// pub static START_WINDOW: WindowType = WindowType::MarkdownView;

pub(crate) struct VeConfig {
	pub bg: Color32,
	pub surface: Color32,
	pub activity_bar: PanelState,
	pub primary_bar: PanelState,
	pub secondary_bar: PanelState,
	pub bottom_panel: PanelState,
	pub status_bar: PanelState,
	pub dock_left: PanelState,
	pub dock_right: PanelState,
}
impl VeConfig {
	pub const fn default() -> Self {
		Self {
			bg: palette::BG,
			surface: palette::SURFACE,
			activity_bar: PanelState::new(true, 48.0),
			primary_bar: PanelState::new(true, 40.0),
			secondary_bar: PanelState::new(true, 48.0),
			bottom_panel: PanelState::new(true, 240.0),
			status_bar: PanelState::new(true, 24.0),
			dock_left: PanelState::new(true, 280.0),
			dock_right: PanelState::new(true, 320.0),
		}
	}
	pub const fn zen() -> Self {
		Self {
			activity_bar: PanelState::new(false, 48.0),
			primary_bar: PanelState::new(false, 40.0),
			secondary_bar: PanelState::new(false, 48.0),
			bottom_panel: PanelState::new(false, 240.0),
			status_bar: PanelState::new(false, 24.0),
			dock_left: PanelState::new(false, 280.0),
			dock_right: PanelState::new(false, 320.0),
			..Self::default()
		}
	}
}
impl VeConfig {
	pub const fn new(
		activity_bar: bool,
		primary_bar: bool,
		secondary_bar: bool,
		bottom_panel: bool,
		status_bar: bool,
		dock_left: bool,
		dock_right: bool,
	) -> Self {
		Self {
			bg: palette::BG,
			surface: palette::SURFACE,
			activity_bar: PanelState::new(activity_bar, 48.0),
			primary_bar: PanelState::new(primary_bar, 40.0),
			secondary_bar: PanelState::new(secondary_bar, 48.0),
			bottom_panel: PanelState::new(bottom_panel, 240.0),
			status_bar: PanelState::new(status_bar, 24.0),
			dock_left: PanelState::new(dock_left, 280.0),
			dock_right: PanelState::new(dock_right, 320.0),
		}
	}
}
pub(crate) const LAYOUT: VeConfig = VeConfig::default();

use std::sync::atomic::AtomicU64;

// pub static EVENT_ID: AtomicU64 = AtomicU64::new(1);
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

// pub fn hmr_chart_json() -> PathBuf {
// Path::new(ROOT_DIR).join("src/data/chart.json")
// }
//
// pub fn markdown() -> PathBuf {
// Path::new(ROOT_DIR).join("src/data/corpus.md")
// }
//
// pub fn pipeline_diagram() -> PathBuf {
// Path::new(ROOT_DIR).join("estate/1-estate-diagram.md")
// }
//
// pub fn pipeline_estate_workspace() -> PathBuf {
// Path::new(ROOT_DIR).join("estate/1-estate-workspace-with-persona.md")
// }
//
// pub fn template_path() -> PathBuf {
// Path::new(ROOT_DIR).join("template")
// }
//
// pub static HMR_CHART_JSON: LazyLock<PathBuf> =
// LazyLock::new(|| PathBuf::from(ROOT_DIR).join("src/data/chart.json"));
//
// pub static MARKDOWN: LazyLock<PathBuf> =
// LazyLock::new(|| PathBuf::from(ROOT_DIR).join("src/data/corpus.md"));
//
// pub static PIPELINE_DIAGRAM: LazyLock<PathBuf> =
// LazyLock::new(|| PathBuf::from(ROOT_DIR).join("estate/1-estate-diagram.md"));
//
// pub static TEMPLATE_PATH: LazyLock<PathBuf> =
// LazyLock::new(|| PathBuf::from(ROOT_DIR).join("template"));
