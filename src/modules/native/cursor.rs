use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum CursorEvent {
	CursorPosition { x: f64, y: f64 },
	ModifiersChanged(global_hotkey::hotkey::Modifiers),
}
