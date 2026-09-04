use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct VeLayout {
	pub activity_bar: egui::Rect,
	pub dock_left: egui::Rect,
	pub main: egui::Rect,
	pub primary_bar: egui::Rect,
	pub secondary_bar: egui::Rect,
	pub bottom_panel: egui::Rect,
	pub status_bar: egui::Rect,
	pub dock_right: egui::Rect,
}

pub static CURSOR_INSET: f64 = 0.125;
pub static REDIRECTING_SCROLL: AtomicBool = AtomicBool::new(false);
pub static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
