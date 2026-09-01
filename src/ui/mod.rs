pub(crate) mod chart;
pub(crate) mod components;
pub(crate) mod screen;
pub(crate) mod view;

pub mod ve;

pub use crate::ui::{screen::*, ve::*, view::*};
pub use crate::{palette::*, prelude::*};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));

pub struct PanelConfig {
	/// Is the panel "open"? Think sidebar.
	pub active: bool,
	pub size: f32,
	/// Drablable? Think left sidebar vs status bar
	pub resizable: bool,
	pub docked: bool,
}
impl PanelConfig {
	/// New Panel
	pub const fn new(active: bool, size: f32) -> Self {
		Self {
			active,
			size,
			resizable: true,
			docked: true,
		}
	}
}

#[derive(Clone, Copy)]
pub enum ResizeEdge {
	Left,
	Right,
	Top,
	Bottom,
}
