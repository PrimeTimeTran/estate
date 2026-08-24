//! Static definitions shared by estate dependencies
//! # Description
use std::sync::{Mutex, OnceLock, atomic::AtomicBool};

use crate::{
	core::{Probe, ProbeKind, ProbeSet},
	ve::ScrollRedirectState,
};

pub const FILE_EXTENSIONS: &[&str] = &[
	"rs", "loi", "estate", "html", "htm", "css", "js", "jsx", "ts", "tsx", "json", "jsonc", "md",
	"mdx", "txt", "toml", "yaml", "yml", "ini", "conf", "sh", "bash", "zsh", "c", "h", "cpp", "hpp",
	"py", "go", "java", "kt", "png", "jpg", "jpeg", "svg", "webp", "ico", "csv", "xml", "sql",
];
pub const FILE_NAMES: &[&str] = &[
	"Dockerfile",
	"Makefile",
	"LICENSE",
	"README",
	"README.md",
	"Cargo.toml",
	"package.json",
];
pub const TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";
pub const SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub const PID_PATH: &str = "/tmp/estate-daemon.pid";
pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SCHEMA_VERSION: u32 = 1;
pub const PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub const PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";

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

pub static SCROLL_STATE: OnceLock<Mutex<ScrollRedirectState>> = OnceLock::new();
pub static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
pub static HOTKEY_INITIALIZED: AtomicBool = AtomicBool::new(false);
