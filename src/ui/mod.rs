pub mod chart;
pub mod config;
pub mod layout;
pub mod panel;
pub mod primitive;
pub mod region;
pub mod screen;
pub mod theme;
pub mod r#trait;
pub mod ui_prelude;
pub mod view;

pub use crate::{
	ui::{
		config::*, layout::*, panel::*, primitive::*, region::*, screen::*, theme::palette::*, view::*,
	},
	ui_prelude::*,
};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
