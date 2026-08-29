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
pub static ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static PID_PATH: &str = "/tmp/estate-daemon.pid";
pub static PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub static PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";
pub static SCHEMA_VERSION: u32 = 1;
pub static SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub static TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";
pub static STATE_PATH: &str = "/Users/future/Library/Application Support/estate/state.json";
