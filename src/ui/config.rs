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
