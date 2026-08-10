pub const TEMPLATE_PATH: &str = "/Users/future/KB/project/crates/estate/template";
pub const SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub const PID_PATH: &str = "/tmp/estate-daemon.pid";
pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SCHEMA_VERSION: u32 = 1;
pub const PIPELINE_DIAGRAM: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-diagram.md";
pub const PIPELINE_ESTATE_WORKSPACE: &str =
	"/Users/future/KB/project/crates/estate/estate/1-estate-workspace-with-persona.md";
