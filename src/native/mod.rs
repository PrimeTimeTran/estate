pub mod agent;
pub mod backend;
pub mod daemon;
pub mod discovery;
pub mod lint;
pub mod monitor;
pub mod native_job;
pub mod native_prelude;
pub mod poc;
pub mod resolver;
pub mod router;
pub mod screens;
pub mod scroll;
pub mod state;
pub mod task;
pub mod ui;
pub mod window;

pub use self::{discovery::*, native_prelude::*, scroll::*};
use egui::MenuBar;
pub use screens::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[derive(Debug, Default)]
pub struct NativeState;

#[derive(Default)]
pub struct NativeContext {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	pub tray_cursor: Option<TrayIcon>,
	pub windows: Vec<AppWindow>,
}

impl AppCtx for NativeContext {
	type State = NativeState;
	fn state(&self) -> &Self::State {
		&self.state()
	}
}
