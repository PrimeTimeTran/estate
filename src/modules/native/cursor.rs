use crate::prelude::*;

pub use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};

#[derive(Debug, Clone, Copy)]
pub enum CursorEvent {
	CursorPosition { x: f64, y: f64 },
	ModifiersChanged(Modifiers),
}
