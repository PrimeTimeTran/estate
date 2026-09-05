pub mod chart;
pub mod config;
pub mod layout;
pub mod panel;
pub mod primitive;
pub mod region;
pub mod screen;
pub mod theme;
pub mod ui_prelude;
pub mod ui_trait;
pub mod view;

pub use crate::ui::{
	config::*, layout::*, primitive::*, region::*, screen::*, theme::palette::*, ui_prelude::*,
	view::*,
};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
