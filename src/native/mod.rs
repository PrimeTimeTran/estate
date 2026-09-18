pub mod agent;
pub mod app;
pub mod backend;
pub mod daemon;
pub mod discovery;
pub mod job;
pub mod lint;
pub mod monitor;
pub mod poc;
#[path = "[prelude].rs"]
pub mod prelude_native;
pub mod resolver;
pub mod router;
pub mod runtime;
pub mod screens;
pub mod scroll;
pub mod state;
pub mod task;
pub mod ui;
pub mod util;
pub mod window;

use crate::ui::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;
