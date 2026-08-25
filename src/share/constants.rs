use std::sync::{ Mutex, OnceLock, atomic::{ AtomicBool, AtomicU64 } };

use crate::share::prelude::*;

pub static EVENT_ID: AtomicU64 = AtomicU64::new(1);

pub const FILE_EXTENSIONS: &[&str] = &[
	"rs",
	"loi",
	"estate",
	"html",
	"htm",
	"css",
	"js",
	"jsx",
	"ts",
	"tsx",
	"json",
	"jsonc",
	"md",
	"mdx",
	"txt",
	"toml",
	"yaml",
	"yml",
	"ini",
	"conf",
	"sh",
	"bash",
	"zsh",
	"c",
	"h",
	"cpp",
	"hpp",
	"py",
	"go",
	"java",
	"kt",
	"png",
	"jpg",
	"jpeg",
	"svg",
	"webp",
	"ico",
	"csv",
	"xml",
	"sql",
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
pub const ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PID_PATH: &str = "/tmp/estate-daemon.pid";
pub const PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub const PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";
pub const SCHEMA_VERSION: u32 = 1;
pub const SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub const TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";
