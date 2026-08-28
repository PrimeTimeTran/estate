use crate::{app::Runtime, native::prelude::*};

use core_graphics::{
	display::CGDisplay,
	event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry::CGPoint,
};

use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuItem, Submenu},
};

pub fn bootstrap() -> anyhow::Result<(TrayMenu, TrayIcon)> {
	let menu = Menu::new();
	let clock_item = MenuItem::new("Clock: 30s", true, None);
	let scroll_item = MenuItem::new("Scroll: Idle", true, None);
	let status = MenuItem::new("● Estate Daemon Running", false, None);
	let dev = MenuItem::new("Dashboard", true, None);
	let telemetry = MenuItem::new("Telemetry", true, None);
	let task_manager = MenuItem::new("Task Manager", true, None);
	let new_task = MenuItem::new("New Task", true, None);
	let list_tasks = MenuItem::new("List Tasks", true, None);
	let clear_tasks = MenuItem::new("Clear Tasks", true, None);
	let tasks = Submenu::new("Tasks", true);
	tasks.append(&new_task)?;
	tasks.append(&list_tasks)?;
	tasks.append(&clear_tasks)?;
	let quit = MenuItem::new("Quit", true, None);
	menu.append(&clock_item)?;
	menu.append(&scroll_item)?;
	menu.append(&status)?;
	menu.append(&dev)?;
	menu.append(&telemetry)?;
	menu.append(&task_manager)?;
	menu.append(&tasks)?;
	menu.append(&quit)?;
	let tray = TrayIconBuilder::new()
		.with_icon(tray_icon())
		.with_menu(Box::new(menu))
		.with_tooltip("Estate Daemon — Running")
		.build()
		.map_err(|e| anyhow::anyhow!("failed to create tray icon: {e}"))?;

	Ok((
		TrayMenu {
			clear_tasks,
			dev,
			list_tasks,
			new_task,
			quit,
			status,
			tasks,
			telemetry,
			task_manager,
		},
		tray,
	))
}

pub fn tray_icon() -> Icon {
	let image = image::load_from_memory(crate::ui::TRAY_ICON)
		.expect("failed to load generated tray icon")
		.into_rgba8();
	let (width, height) = image.dimensions();
	Icon::from_rgba(image.into_raw(), width, height).expect("failed to create tray icon")
}

pub fn scroll_tray_icon() -> tray_icon::Icon {
	let image = image::load_from_memory(crate::ui::TRAY_SCROLL_ICON)
		.expect("failed to load scroll tray icon")
		.into_rgba8();
	let (width, height) = image.dimensions();
	tray_icon::Icon::from_rgba(image.into_raw(), width, height)
		.expect("failed to create scroll tray icon")
}

#[derive(Debug, Copy, Clone)]
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
pub enum ScreenPosition {
	Left,
	Center,
	Right,
}
pub fn move_cursor_to(pos: ScreenPosition) {
	let bounds = CGDisplay::main().bounds();

	let x = match pos {
		ScreenPosition::Left => bounds.origin.x + bounds.size.width * 0.125,

		ScreenPosition::Center => bounds.origin.x + bounds.size.width * 0.5,

		ScreenPosition::Right => bounds.origin.x + bounds.size.width * 0.875,
	};

	let y = bounds.origin.y + bounds.size.height * 0.5;

	let point = CGPoint { x, y };

	if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
		if let Ok(event) =
			CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
		{
			event.post(CGEventTapLocation::HID);
		}
	}
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

// impl<R: Runtime> Panel<R> {
// 	pub fn new(region: Region<R>) -> Self {
// 		Self {
// 			region,
// 			open: true,
// 			overlay: false,
// 			auto_hide: false,
// 		}
// 	}
// 	pub fn with_open(mut self, open: bool) -> Self {
// 		self.open = open;
// 		self
// 	}
// 	pub fn with_overlay(mut self, overlay: bool) -> Self {
// 		self.overlay = overlay;
// 		self
// 	}
// 	pub fn with_auto_hide(mut self, auto_hide: bool) -> Self {
// 		self.auto_hide = auto_hide;
// 		self
// 	}
// 	pub fn open(&mut self) {
// 		self.open = true;
// 	}
// 	pub fn close(&mut self) {
// 		self.open = false;
// 	}
// 	pub fn toggle(&mut self) {
// 		self.open = !self.open;
// 	}
// }

use crate::app::{AppContext, state::NativeRuntime, ve::Veable};

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
