use crate::{prelude::*, ui::PanelState};

// Estate daemon process identity / control socket.
pub static ESTATE_PID: &str = "/tmp/estate-daemon.pid";
pub static ESTATE_DAEMON_SOCKET: &str = "/tmp/estate-daemon.sock";

// Estate application IPC socket.
pub static ESTATE_IPC_SOCKET: &str = "/tmp/estate.sock";

// Native macOS HID/OS-observer IPC socket.
pub static ESTATE_HID_SOCKET: &str = "/tmp/estate-hid.sock";
pub static ESTATE_HID_SMOKE_LOG: &str = "estate-hid-smoke.log";

pub static ESTATE_HOME_DIR: &str = ".config/estate";
pub static ESTATE_INDEX_PATH: &str = ".config/estate/master.json";

pub static SCHEMA_VERSION: u32 = 1;
pub static NEXT_PROBLEM_ID: AtomicI64 = AtomicI64::new(1);
pub static START_APP_CLOCK: bool = true;

pub static NEXT_RECEIVER_ID: AtomicU64 = AtomicU64::new(0);
