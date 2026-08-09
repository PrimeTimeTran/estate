pub const SOCKET_PATH: &str = "/tmp/estate-daemon.sock";
pub const PID_PATH: &str = "/tmp/estate-daemon.pid";

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const ESTATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SCHEMA_VERSION: u32 = 1;
