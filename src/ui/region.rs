use egui::{ScrollArea, Ui};
use std::fmt;
use strum::IntoStaticStr;

use crate::{api::Api, e, prelude::*, ui::Layout};

#[cfg(feature = "native")]
use crate::native::{DashboardScreen, WaterfallChart, prelude::*};

// View
//   │
//   │ selects which screen is active
//   ▼
// Screen
//   │
//   │ composes/configures regions
//   ▼
// Ve
//   │
//   │ owns global layout + panels
//   ▼
// Veable
//   │
//   │ renders one concrete piece of UI
//   ▼
// egui
// 1. View — application-level navigation
// "What screen is the application currently displaying?"
// 2. Screen — composition/coordinator
// "What does this particular screen put into the available UI regions?"
// 3. Ve — the visual skeleton
// "Where do things go and how does the workspace behave?"
// 4. Veable — a renderable UI component
// "How do I render this particular piece of UI?"
pub struct Region {
	// Layout
	pub size: f32,
	pub min_size: f32,
	pub max_size: f32,
	pub resizable: bool,

	// Positioning
	pub docked: bool,

	// Presentation
	pub padding: egui::Margin,
	pub fill: Option<egui::Color32>,
	pub top_border: bool,
}
impl Region {
	pub fn new(size: f32) -> Self {
		Self {
			docked: true,
			fill: None,
			max_size: size,
			min_size: size,
			padding: egui::Margin::ZERO,
			resizable: false,
			size,
			top_border: false,
		}
	}
	pub fn fixed(size: f32) -> Self {
		Self {
			size,
			min_size: size,
			max_size: size,
			resizable: false,
			docked: true,
			fill: None,
			padding: egui::Margin::ZERO,
			top_border: false,
		}
	}
	pub fn resizable(size: f32, min_size: f32, max_size: f32) -> Self {
		Self {
			size: size.clamp(min_size, max_size),
			min_size,
			max_size,
			resizable: true,
			docked: true,
			fill: Some(palette::SURFACE),
			padding: egui::Margin::ZERO,
			top_border: false,
		}
	}
	pub fn content() -> Self {
		Self {
			size: 0.0,
			min_size: 0.0,
			max_size: f32::MAX,
			resizable: false,
			docked: false,
			fill: None,
			padding: egui::Margin::ZERO,
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

pub struct Panel<R: Runtime, E> {
	pub region: Region,
	pub content: Box<dyn ViewTrait<R, E>>,
	pub open: bool,
	pub overlay: bool,
	pub auto_hide: bool,
}
impl<R: Runtime, E> Panel<R, E> {
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
impl<R: Runtime, E> Panel<R, E> {
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
impl<R: Runtime, E> ViewTrait<R, E> for Panel<R, E> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		self.content.draw(ui, ctx);
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}

#[derive(Debug, Clone, Copy)]
pub struct PanelState {
	/// Is the panel "open"? Think sidebar.
	pub active: bool,
	pub size: f32,
	/// Drablable? Think left sidebar vs status bar
	pub resizable: bool,
	pub docked: bool,
}
impl PanelState {
	pub const fn new(active: bool, size: f32) -> Self {
		Self {
			active,
			size,
			resizable: true,
			docked: true,
		}
	}
	pub const fn size(&self) -> f32 {
		if self.active { self.size } else { 0.0 }
	}
	pub const fn is_active(&self) -> bool {
		self.active
	}
	pub const fn effective_size(&self) -> f32 {
		if self.active { self.size } else { 0.0 }
	}
	pub fn region(&self, min_size: f32, max_size: f32) -> Region {
		Region::resizable(self.size, min_size, max_size)
	}
}
impl PanelState {
	pub const fn activity_bar() -> Self {
		Self::new(true, 48.0)
	}
	pub const fn primary_bar() -> Self {
		Self::new(true, 40.0)
	}
	pub const fn secondary_bar() -> Self {
		Self::new(true, 48.0)
	}
	pub const fn bottom_panel() -> Self {
		Self::new(false, 240.0)
	}
	pub const fn status_bar() -> Self {
		Self::new(true, 24.0)
	}
	pub const fn dock_left() -> Self {
		Self::new(true, 280.0)
	}
	pub const fn dock_right() -> Self {
		Self::new(true, 320.0)
	}
}

pub struct ActivityBar {
	buttons: Vec<&'static str>,
}
impl ActivityBar {
	pub fn new() -> Self {
		Self { buttons: vec![] }
	}
}
impl<R: Runtime, E> ViewTrait<R, E> for ActivityBar {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		ui.vertical(|ui| {
			// buttons
		});
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}

pub struct PrimaryBar {
	buttons: Vec<&'static str>,
}
impl<R: Runtime, E> ViewTrait<R, E> for PrimaryBar {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		ui.horizontal(|ui| {
			// buttons
		});
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}
