pub use crate::app::*;
use crate::{e, theme::palette};

// #[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "native")]
pub use crate::native::prelude::*;

pub trait Veable<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);
	fn update(&mut self, _ctx: &mut AppContext<'_, R>) {}
	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, R>) {}
}

pub struct Ve<R: Runtime> {
	///      A type-erased container for any concrete `Veable`.
	///
	///      `Box<dyn Veable>` stores the concrete implementation on the heap while
	///      exposing only the `Veable` interface to callers. This allows different
	///      concrete implementations to be substituted without changing the code
	///      which consumes them.
	// Top left to bottom right ordering for mental model.
	// Top left to bottom right ordering for mental model.
	pub activity_bar: Region<R>,
	pub dock_left: Panel<R>,
	pub main: Region<R>,
	pub primary_bar: Region<R>,
	pub secondary_bar: Region<R>,
	pub bottom_panel: Panel<R>,
	pub status_bar: Region<R>,
	pub dock_right: Panel<R>,
}
// impl<R: Runtime> Veable<R> for Ve<R> {
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
// 		self.main.draw(ui, ctx);
// 		// self.activity_bar.draw(ui, ctx);
// 		// self.dock_left.draw(ui, ctx);
// 		// self.primary_bar.draw(ui, ctx);
// 		// self.secondary_bar.draw(ui, ctx);
// 		// self.bottom_panel.draw(ui, ctx);
// 		// self.status_bar.draw(ui, ctx);
// 		// self.dock_right.draw(ui, ctx);
// 	}
// }
// pub struct Ve<R: Runtime> {
// 	///      A type-erased container for any concrete `Veable`.
// 	///
// 	///      `Box<dyn Veable>` stores the concrete implementation on the heap while
// 	///      exposing only the `Veable` interface to callers. This allows different
// 	///      concrete implementations to be substituted without changing the code
// 	///      which consumes them.
// 	// Top left to bottom right ordering for mental model.
// 	// Top left to bottom right ordering for mental model.
// 	// pub activity_bar: Region<R>,
// 	// pub dock_left: Panel<R>,
// 	pub main: Region<R>,
// 	// pub primary_bar: Region<R>,
// 	// pub secondary_bar: Region<R>,
// 	// pub bottom_panel: Panel<R>,
// 	// pub status_bar: Region<R>,
// 	// pub dock_right: Panel<R>,
// }
// impl<R: Runtime> Ve<R> {
// 	pub fn new(view: impl Veable<R> + 'static) -> Self {
// 		Self {
// 			main: Region::content(view),
// 		}
// 	}
// }
pub struct Region<R: Runtime> {
	pub content: Box<dyn Veable<R>>,
	// Layout
	pub size: f32,
	pub min_size: f32,
	pub max_size: f32,
	pub resizable: bool,
	// Presentation
	pub padding: egui::Margin,
	pub fill: Option<egui::Color32>,
	pub is_docked: bool,
	pub top_border: bool,
}
impl<R: Runtime> Region<R> {
	pub fn new(view: impl Veable<R> + 'static, size: f32) -> Self {
		Self {
			content: Box::new(view),
			fill: None,
			is_docked: false,
			max_size: size,
			min_size: size,
			padding: egui::Margin::ZERO,
			resizable: false,
			size,
			top_border: false,
		}
	}
	pub fn fixed(view: impl Veable<R> + 'static, size: f32) -> Self {
		Self::new(view, size)
	}
	pub fn resizable(
		view: impl Veable<R> + 'static,
		size: f32,
		min_size: f32,
		max_size: f32,
	) -> Self {
		let mut region = Self::new(view, size);
		region.min_size = min_size;
		region.max_size = max_size;
		region.resizable = true;
		region.fill = Some(palette::SURFACE);
		region.is_docked = true;
		region
	}
	pub fn content(view: impl Veable<R> + 'static) -> Self {
		Self {
			content: Box::new(view),
			size: 0.0,
			min_size: 0.0,
			max_size: f32::MAX,
			fill: None,
			padding: egui::Margin::ZERO,
			resizable: false,
			is_docked: false,
			top_border: false,
		}
	}
	pub fn with_fill(mut self, fill: egui::Color32) -> Self {
		self.fill = Some(fill);
		self
	}
	pub fn set_size(&mut self, size: f32) {
		self.size = size.clamp(self.min_size, self.max_size);
	}
	pub fn resize(&mut self, delta: f32) {
		self.set_size(self.size + delta);
	}
	pub fn with_padding(mut self, padding: i32) -> Self {
		self.padding = egui::Margin::same(padding as i8);
		self
	}
	pub fn with_top_border(mut self, enabled: bool) -> Self {
		self.top_border = enabled;
		self
	}
	pub fn content_rect(&self, rect: egui::Rect) -> egui::Rect {
		egui::Rect::from_min_max(
			egui::pos2(
				rect.left() + (self.padding.left as f32),
				rect.top() + (self.padding.top as f32),
			),
			egui::pos2(
				rect.right() - (self.padding.right as f32),
				rect.bottom() - (self.padding.bottom as f32),
			),
		)
	}
}
// impl<R: Runtime> Veable<R> for Region<R> {
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
// 		// Region itself controls layout/presentation.
// 		//
// 		// For now, simply delegate to the contained view.
// 		self.content.draw(ui, ctx);
// 	}
// 	fn update(&mut self, ctx: &mut AppContext<'_, R>) {
// 		self.content.update(ctx);
// 	}
// 	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {
// 		self.content.event(event, ctx);
// 	}
// }
/// A named, interactive view that occupies a region.
///
/// Panels add interaction and lifecycle behavior to a Region.
/// They may be opened, closed, overlaid, auto-hidden, moved,
/// or potentially detached from their parent layout.

pub struct Panel<R: Runtime> {
	pub region: Region<R>,
	pub open: bool,
	pub overlay: bool,
	pub auto_hide: bool,
}

impl<R: Runtime> Veable<R> for Panel<R> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		self.region.content.draw(ui, ctx);
	}
}

impl<R: Runtime> Panel<R> {
	pub fn new(region: Region<R>) -> Self {
		Self {
			region,
			open: true,
			overlay: false,
			auto_hide: false,
		}
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
}

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

impl<R: Runtime> Veable<R> for DebugPanel {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
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
}

pub struct Size {
	pub value: f32,
	pub min: f32,
	pub max: f32,
	pub resizable: bool,
}
impl Size {
	pub fn new(value: f32, min: f32, max: f32) -> Self {
		Self {
			value: value.clamp(min, max),
			min,
			max,
			resizable: true,
		}
	}
	pub fn set(&mut self, value: f32) {
		self.value = value.clamp(self.min, self.max);
	}
	pub fn resize(&mut self, delta: f32) {
		self.set(self.value + delta);
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	#[default]
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
}
