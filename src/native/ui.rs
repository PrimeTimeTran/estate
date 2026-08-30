use crate::{app::Runtime, native::prelude::*, ui::Veable};

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

pub fn bootstrap() -> Result<(TrayMenu, TrayIcon)> {
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
