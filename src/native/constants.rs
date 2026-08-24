use std::sync::{
	Mutex, OnceLock,
	atomic::{AtomicBool, AtomicU64},
};

use crate::{
	native::prelude::{PanelConfig, ScrollRedirectState},
	shared::prelude::*,
};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub static HOTKEY_INITIALIZED: AtomicBool = AtomicBool::new(false);
pub static REDIRECTING_SCROLL: AtomicBool = AtomicBool::new(false);
pub static SCROLL_STATE: OnceLock<Mutex<ScrollRedirectState>> = OnceLock::new();
pub static SHIFT_HELD: AtomicBool = AtomicBool::new(false);

pub mod palette {
	use egui::Color32;
	pub const BG: Color32 = Color32::from_rgb(18, 20, 24);
	pub const SURFACE: Color32 = Color32::from_rgb(27, 30, 36);
	pub const SURFACE_HOVER: Color32 = Color32::from_rgb(34, 38, 46);
	pub const BORDER: Color32 = Color32::from_rgb(52, 57, 68);
	pub const TEXT: Color32 = Color32::from_rgb(232, 235, 240);
	pub const TEXT_MUTED: Color32 = Color32::from_rgb(145, 152, 165);
	pub const PRIMARY: Color32 = Color32::from_rgb(100, 160, 255);
	pub const SUCCESS: Color32 = Color32::from_rgb(82, 190, 125);
	pub const WARNING: Color32 = Color32::from_rgb(235, 180, 70);
	pub const DANGER: Color32 = Color32::from_rgb(230, 90, 95);
	pub const GRID: Color32 = Color32::from_rgb(45, 49, 58);
}

pub const DEFAULT_CONFIG: VeConfig = VeConfig {
	bg: palette::BG,
	surface: palette::SURFACE,
	activity_bar: PanelConfig::new(true, 48.0),
	primary_bar: PanelConfig::new(true, 40.0),
	secondary_bar: PanelConfig::new(true, 48.0),
	bottom_panel: PanelConfig::new(true, 240.0),
	status_bar: PanelConfig::new(true, 24.0),
	dock_left: PanelConfig::new(true, 280.0),
	dock_right: PanelConfig::new(true, 320.0),
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
pub const PROBES_MINIMAL: ProbeSet = &[
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
];
pub const PROBES_NODE: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "vscode",
		name: ".vscode",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "npm",
		name: "package.json",
		kind: ProbeKind::File,
	},
	Probe {
		id: "lockfile",
		name: "package-lock.json",
		kind: ProbeKind::File,
	},
	Probe {
		id: "prettier",
		name: ".prettierrc",
		kind: ProbeKind::File,
	},
];
pub const PROBES_RUST_ZED: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "zed",
		name: ".zed",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "rust_lock",
		name: "Cargo.lock",
		kind: ProbeKind::File,
	},
];
pub const PROBES_PERSONAL: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "bookmarks",
		name: "bookmarks",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "knowledge",
		name: "knowledge",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "commands",
		name: "commands",
		kind: ProbeKind::Directory,
	},
];
pub const REACT_CONFIGS: ProbeSet = &[
	Probe {
		id: "js",
		name: "next.config.js",
		kind: ProbeKind::File,
	},
	Probe {
		id: "ts",
		name: "next.config.ts",
		kind: ProbeKind::File,
	},
];
pub const PROBES_MONOREPO: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "workspace",
		name: "pnpm-workspace.yaml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "npm",
		name: "package.json",
		kind: ProbeKind::File,
	},
];
pub const PROBES: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "vscode",
		name: ".vscode",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "zed",
		name: ".zed",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
];
