pub(crate) mod chart;
pub(crate) mod config;
pub(crate) mod layout;
pub(crate) mod panel;
mod prelude;
pub(crate) mod primitive;
pub(crate) mod region;
pub(crate) mod screen;
pub(crate) mod theme;
pub(crate) mod r#trait;
pub(crate) mod view;

pub use crate::{
	prelude::*,
	ui::{
		config::*, layout::*, panel::*, primitive::*, region::*, screen::*, theme::palette::*, view::*,
	},
};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
