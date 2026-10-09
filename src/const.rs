use crate::{prelude::*, ui::PanelState};

// Estate daemon process identity / control socket.
pub static PATH_PID: &str = "/tmp/estate-daemon.pid";
pub static PATH_SOCKET: &str = "/tmp/estate-daemon.sock";

// Estate application IPC socket.
pub static ESTATE_SOCKET: &str = "/tmp/estate.sock";

// Native macOS HID/OS-observer IPC socket.
pub static HID_SOCKET: &str = "/tmp/estate-hid.sock";
pub static ESTATE_HID_SMOKE_LOG: &str = "estate-hid-smoke.log";

pub static SCHEMA_VERSION: u32 = 1;
pub static NEXT_PROBLEM_ID: AtomicI64 = AtomicI64::new(1);
pub static START_APP_CLOCK: bool = true;
