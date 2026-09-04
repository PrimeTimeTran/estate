use crate::prelude::*;

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
	Probe {
		id: "estate",
		name: "settings.json",
		kind: ProbeKind::File,
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
pub static PROBES_WORKSPACE: &[Probe] = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "estate-settings",
		name: ".estate/settings.json",
		kind: ProbeKind::File,
	},
	Probe {
		id: "estate-keymap",
		name: ".estate/key-map.json",
		kind: ProbeKind::File,
	},
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
