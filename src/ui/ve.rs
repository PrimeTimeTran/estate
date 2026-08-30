use crate::{chart::ChartsFile, data::DEFAULT_CONFIG as CONFIG};

pub use crate::{app::runtime::Runtime, native::prelude::*, prelude::*, theme::palette, ui::*};

use crate::app::AppContext;
use crate::native::runtime::NativeRuntime;

use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points};

pub trait Veable<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);
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

		region.size = size.clamp(min_size, max_size);
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

use ::serde::Deserialize;

// #[derive(Debug)]
// pub struct Graphics<R: Runtime> {
// data: ChartsFile,
// dirty: bool,
// error: Option<String>,
// scroll_x: f32,
// scroll_y: f32,
// last_direction: String,
// pub side_panel_width: f32,
// pub active_focus: FocusedPane,
// pub secondary_scroll_offset: f32,
// }
// impl<R: Runtime> Graphics<R> {
// pub fn new() -> Self {
// let data = Self::load_data();
//
// Self {
// data,
// dirty: false,
// error: None,
// scroll_x: 0.0,
// scroll_y: 0.0,
// last_direction: String::new(),
// side_panel_width: 0.0,
// active_focus: FocusedPane::MainEditor,
// secondary_scroll_offset: 0.0,
// }
// }
// fn load_data() -> ChartsFile {
// serde_json::from_str(include_str!("../data/chart.json")).expect("failed to parse chart.json")
// }
// fn draw_error(&self, ui: &mut egui::Ui, error: &str) {
// ui.heading("Preview Error");
// ui.colored_label(egui::Color32::RED, error);
// ui.separator();
// ui.label("Preview is showing the last valid state.");
// }
// fn draw_ui(&mut self, ui: &mut egui::Ui) {
// if let Some(error) = &self.error {
// self.draw_error(ui, error);
// return;
// }
// let charts = &self.data.charts;
// // Split the available window into a 2x2 grid.
// let available = ui.available_size();
// let cell_width = available.x / 2.0;
// let cell_height = available.y / 2.0;
// for row in 0..2 {
// ui.horizontal(|ui| {
// ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
// for column in 0..2 {
// let index = row * 2 + column;
// ui.allocate_ui(egui::vec2(cell_width - 8.0, cell_height - 8.0), |ui| {
// ui.group(|ui| {
// ui.set_min_size(ui.available_size());
// if let Some(chart) = charts.get(index) {
// chart.ui(ui);
// } else {
// ui.centered_and_justified(|ui| {
// ui.label("No chart");
// });
// }
// });
// });
// }
// });
// }
// }
// }
//
// impl<R: Runtime> Veable<R> for Graphics<R> {
// fn draw(&mut self, ui: &mut egui::Ui) {
// // 1. Poll the channel for file changes on every frame render tick
// // self.check_for_changes(ui.ctx());
// // 2. Split the available space to reserve room for the bottom status bar
// let available_size = ui.available_size();
// let status_bar_height = CONFIG.status_bar.size;
// let main_size = egui::vec2(available_size.x, available_size.y - status_bar_height);
// // Main Content Area
// ui.allocate_ui(main_size, |ui| {
// self.draw_ui(ui);
// });
// ui.separator();
// // Bottom Status Bar
// ui.horizontal(|ui| {
// // Left side: Status or error indicator
// if let Some(error) = &self.error {
// ui.colored_label(egui::Color32::RED, "Status: Error");
// } else if self.dirty {
// ui.colored_label(egui::Color32::YELLOW, "Status: Unsaved / Out of sync");
// } else {
// ui.colored_label(egui::Color32::GREEN, "Status: Connected");
// }
// ui.separator();
// // Right side: Timer / Last Loaded counter
// // if let Some(last_loaded) = self.last_loaded {
// // 	if let Ok(elapsed) = last_loaded.elapsed() {
// // 		let secs = elapsed.as_secs();
// // 		let time_str = if secs < 60 {
// // 			format!("Loaded {secs}s ago")
// // 		} else {
// // 			format!("Loaded {}m {}s ago", secs / 60, secs % 60)
// // 		};
// // 		ui.label(time_str);
// // 	}
// // } else {
// // 	ui.label("Not loaded yet");
// // }
// // Request a continuous repaint so the timer increments live every second
// ui.ctx()
// .request_repaint_after(std::time::Duration::from_secs(1));
// });
// }
// }
//
