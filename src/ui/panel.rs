use crate::{e, prelude::*};

// A named, interactive view that occupies a region.
// Panels add interaction and lifecycle behavior to a Region.
// They may be opened, closed, overlaid, auto-hidden, moved,
// or potentially detached from their parent layout.
pub struct DebugPanel {
	pub title: String,
}
impl DebugPanel {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
		}
	}
}
impl<R: Runtime, E: Executor> ViewTrait<R, E> for DebugPanel {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		ui.vertical_centered(|ui| {
			ui.heading(&self.title);
			ui.separator();
			ui.label(format!(
				"{} × {}",
				ui.available_width(),
				ui.available_height()
			));
		});
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}
