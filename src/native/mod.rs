pub(crate) mod agent;
pub(crate) mod backend;
pub(crate) mod constants_native;
pub(crate) mod core;
pub(crate) mod daemon;
pub(crate) mod job;
pub(crate) mod monitor;
pub(crate) mod poc;
pub(crate) mod prelude;
pub(crate) mod resolver;
pub(crate) mod router;
pub(crate) mod runtime;
pub(crate) mod screens;
pub(crate) mod state;
pub(crate) mod task;
// pub(crate) mod task_manager;
pub(crate) mod ui;
pub(crate) mod ve;
pub(crate) mod window;

pub(crate) use crate::native::task::*;
// pub(crate) use crate::native::task_manager::*;
pub(crate) use runtime::*;
pub(crate) use screens::*;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(target_os = "windows")]
pub(crate) mod windows;

#[cfg(target_os = "macos")]
pub(crate) mod macos;
