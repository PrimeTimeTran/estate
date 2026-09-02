pub(crate) mod chart;
pub(crate) mod layout;
pub(crate) mod panel;
mod prelude;
pub(crate) mod primitive;
pub(crate) mod region;
pub(crate) mod screen;
pub(crate) mod theme;
pub(crate) mod r#trait;
pub(crate) mod view;
pub(crate) use crate::ui::{layout::*, primitive::*, screen::*, view::*};
use crate::{
	e,
	ui::r#trait::{LayoutTrait, Screen, ViewTrait},
};
pub use crate::{
	prelude::*,
	ui::{panel::*, region::*, theme::palette::*},
};

pub const TRAY_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
pub const TRAY_SCROLL_ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
