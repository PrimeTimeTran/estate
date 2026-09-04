use crate::{e, prelude::*};

pub struct Panel<R: Runtime, E> {
	pub region: Region,
	pub content: Box<dyn ViewTrait<R, E>>,
	pub open: bool,
	pub overlay: bool,
	pub auto_hide: bool,
}
impl<R: Runtime, E: Executor> Panel<R, E> {
	pub fn draw(&mut self, ui: &mut egui::Ui, rect: egui::Rect, ctx: &mut AppContext<'_, R, E>) {
		if !self.open {
			return;
		}
		let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
		if let Some(fill) = self.region.fill {
			child_ui.painter().rect_filled(rect, 0.0, fill);
		}
		self.content.draw(&mut child_ui, ctx);
	}
}
impl<R: Runtime, E: Executor> Panel<R, E> {
	pub fn new(content: impl ViewTrait<R, E> + 'static, region: Region) -> Self {
		Self {
			region,
			content: Box::new(content),
			open: true,
			overlay: false,
			auto_hide: false,
		}
	}
	pub fn from_config(
		content: impl ViewTrait<R, E> + 'static,
		region: Region,
		config: &PanelState,
	) -> Self {
		Self::new(content, region).with_open(config.active)
	}
	pub fn with_open(mut self, open: bool) -> Self {
		self.open = open;
		self
	}
	pub fn with_overlay(mut self, overlay: bool) -> Self {
		self.overlay = overlay;
		self
	}
	pub fn with_auto_hide(mut self, auto_hide: bool) -> Self {
		self.auto_hide = auto_hide;
		self
	}
	pub fn open(&mut self) {
		self.open = true;
	}
	pub fn close(&mut self) {
		self.open = false;
	}
	pub fn toggle(&mut self) {
		self.open = !self.open;
	}
	pub fn is_open(self) -> bool {
		self.open
	}
}
impl<R: Runtime, E: Executor> ViewTrait<R, E> for Panel<R, E> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		self.content.draw(ui, ctx);
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}

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
