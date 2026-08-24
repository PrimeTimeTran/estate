use crate::{native::prelude::*, prelude::*};

use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuItem, Submenu},
};

pub fn bootstrap() -> anyhow::Result<(TrayMenu, TrayIcon)> {
	let menu = Menu::new();
	let clock_item = MenuItem::new("Clock: 30s", true, None);
	let scroll_item = MenuItem::new("Scroll: Idle", true, None);
	let status = MenuItem::new("● Estate Daemon Running", false, None);
	let dev = MenuItem::new("Open Dashboard", true, None);
	let telemetry = MenuItem::new("Open Telemetry Inspector", true, None);
	let task_manager = MenuItem::new("Open Task Manager", true, None);
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
	let image = image::load_from_memory(native::constants::TRAY_ICON)
		.expect("failed to load generated tray icon")
		.into_rgba8();
	let (width, height) = image.dimensions();
	Icon::from_rgba(image.into_raw(), width, height).expect("failed to create tray icon")
}

pub fn scroll_tray_icon() -> tray_icon::Icon {
	// constants::TRAY_SCROLL_ICON;
	let image = image::load_from_memory(native::constants::TRAY_SCROLL_ICON)
		.expect("failed to load scroll tray icon")
		.into_rgba8();
	let (width, height) = image.dimensions();
	tray_icon::Icon::from_rgba(image.into_raw(), width, height)
		.expect("failed to create scroll tray icon")
}
