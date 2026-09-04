use crate::{
	app,
	app::{Runtime, state::EstateState},
	e,
	native::prelude::*,
	ui::{Layout, ViewType, scroll::*, r#trait::*, *},
};
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::{
	display::CGDisplay,
	event::*,
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry::CGPoint,
};
use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuItem, Submenu},
};
use winit::event_loop::EventLoopProxy;

// The first concrete implementation of Veable is here.
//
// EguiVeable defines it's own state which is specific to its own implementation
// and the correponding methods which operate on those properties.
//
// The draw method is the gateway for this struct to inject behavior thats independent of the
// generic base and unique to itself as package or an instance of Veable.
#[derive(Clone, Debug, Default)]
pub struct EguiVeable {
	state: EstateState,
	top_tab: DevTopTab,
	side_tab: DevSideTab,
}
impl<R: Runtime, E: Executor> Screen<R, E> for EguiVeable {
	fn configure(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
		todo!("")
	}

	fn update(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
		todo!("")
	}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
		todo!("")
	}
}
impl<R: Runtime, E: Executor> ViewTrait<R, E> for EguiVeable {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		self.draw_ui(ui);
	}

	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
}
impl EguiVeable {
	pub fn new() -> Self {
		Self {
			state: EstateState::load_from_disk().unwrap(),
			top_tab: DevTopTab::Status,
			side_tab: DevSideTab::Overview,
		}
	}
	fn draw_ui(&mut self, ui: &mut egui::Ui) {
		self.draw_side_tabs(ui);
		egui::CentralPanel::default().show_inside(ui, |ui| {
			self.draw_content(ui);
		});
	}
	fn draw_side_tabs(&mut self, ui: &mut egui::Ui) {
		ui.heading("Estate");
		ui.separator();
		for &tab in DevSideTab::ALL {
			let response = ui.selectable_label(self.side_tab == tab, tab.label());
			if response.clicked() {
				tracing::debug!(">>> TAB CLICKED: {:?}", tab);
				self.side_tab = tab;
			}
		}
	}
	fn draw_registry(&self, ui: &mut egui::Ui) {
		ui.heading("Registry");
		ui.separator();
		ui.label("Registry view");
	}
	fn draw_daemon(&self, ui: &mut egui::Ui) {
		ui.heading("Daemon");
		ui.separator();
		ui.label("Daemon view");
	}
	fn draw_engine(&self, ui: &mut egui::Ui) {
		ui.heading("Engine");
		ui.separator();
		ui.label("Engine view");
	}
	fn draw_workspace(&self, ui: &mut egui::Ui) {
		ui.heading("Workspace");
		ui.separator();
		ui.label("Workspace view");
	}
	fn draw_runtime(&self, ui: &mut egui::Ui) {
		ui.heading("Runtime");
		ui.separator();
		ui.label("Runtime view");
	}
	fn draw_tasks(&self, ui: &mut egui::Ui) {
		ui.heading("Tasks");
		ui.separator();
		ui.label("Task manager coming soon.");
	}
	fn draw_logs(&self, ui: &mut egui::Ui) {
		ui.heading("Logs");
		ui.separator();
		ui.label("Logs coming soon.");
	}
	fn draw_config(&self, ui: &mut egui::Ui) {
		ui.heading("Configuration");
		ui.separator();
		ui.label("Configuration coming soon.");
	}
	fn draw_content(&mut self, ui: &mut egui::Ui) {
		match self.top_tab {
			DevTopTab::Status => self.draw_status(ui),
			DevTopTab::Tasks => self.draw_tasks(ui),
			DevTopTab::Logs => self.draw_logs(ui),
			DevTopTab::Config => self.draw_config(ui),
		}
	}
	fn draw_status(&self, ui: &mut egui::Ui) {
		match self.side_tab {
			DevSideTab::Overview => self.draw_overview(ui),
			DevSideTab::Registry => self.draw_registry(ui),
			DevSideTab::Daemon => self.draw_daemon(ui),
			DevSideTab::Engine => self.draw_engine(ui),
			DevSideTab::Workspace => self.draw_workspace(ui),
			DevSideTab::Runtime => self.draw_runtime(ui),
		}
	}
	fn draw_overview(&self, ui: &mut egui::Ui) {
		ui.horizontal(|ui| {
			ui.heading("Overview");
			ui.label(format!("Pointer: {:?}", ui.ctx().pointer_latest_pos()));
			let response = ui.button("📋 Copy");
			tracing::debug!(
				target: "estate::app",
				"Copy button: hovered={} clicked={} enabled={}",
				response.hovered(),
				response.clicked(),
				response.enabled(),
			);
			if response.clicked() {
				tracing::debug!(target: "estate::app", "CLICKED COPY");
				let json =
					serde_json::to_string_pretty(&self.state).expect("failed to serialize estate state");
				ui.output_mut(|o| {
					o.commands.push(egui::OutputCommand::CopyText(json));
				});
			}
		});
		ui.separator();
		let metrics = [
			("Starts", self.state.starts.to_string()),
			("Longest run", format!("{}s", self.state.longest_run)),
			("Status checks", self.state.status_checks.to_string()),
			("Started at", self.state.started_at.to_string()),
			("Events processed", self.state.events_processed.to_string()),
			("Tasks created", self.state.tasks_created.to_string()),
			("Tasks completed", self.state.tasks_completed.to_string()),
			("Files indexed", self.state.files_indexed.to_string()),
		];
		for (name, value) in metrics {
			ui.horizontal(|ui| {
				ui.label(name);
				ui.monospace(value);
			});
		}
	}
}

pub fn bootstrap() -> Result<(TrayMenu, TrayIcon)> {
	let menu = Menu::new();
	let clock_item = MenuItem::new("Clock: 30s", true, None);
	let scroll_item = MenuItem::new("Scroll: Idle", true, None);
	let status = MenuItem::new("● Estate Daemon Running", false, None);
	let dev = MenuItem::new("Dashboard", true, None);
	let oracle = MenuItem::new("Oracle", true, None);
	let task_manager = MenuItem::new("Task Manager", true, None);
	let problem_screen = MenuItem::new("Problems", true, None);
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
	menu.append(&oracle)?;
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
			problem_screen,
			new_task,
			quit,
			status,
			tasks,
			oracle,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DevTopTab {
	#[default]
	Status,
	Tasks,
	Logs,
	Config,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DevSideTab {
	#[default]
	Overview,
	Registry,
	Daemon,
	Engine,
	Workspace,
	Runtime,
}
impl DevSideTab {
	const ALL: &[Self] = &[
		Self::Overview,
		Self::Registry,
		Self::Daemon,
		Self::Engine,
		Self::Workspace,
		Self::Runtime,
	];
	fn label(self) -> &'static str {
		match self {
			Self::Overview => "Overview",
			Self::Registry => "Registry",
			Self::Daemon => "Daemon",
			Self::Engine => "Engine",
			Self::Workspace => "Workspace",
			Self::Runtime => "Runtime",
		}
	}
}

pub struct Sidebar {
	buttons: Vec<&'static str>,
}
impl Sidebar {
	pub fn new() -> Self {
		Self {
			buttons: vec!["New Task", "Show Tasks", "Clear Tasks", "Stop Session"],
		}
	}
}
impl<R: Runtime, E: Executor> Screen<R, E> for Sidebar {
	fn configure(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
		// Configure the regions this screen uses.
	}

	fn update(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
	}
	// fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
	// 	ui.vertical(|ui| {
	// 		for button in &self.buttons {
	// 			if ui.button(*button).clicked() {
	// 				match *button {
	// 					"New Task" => ctx.app.new_task(),
	// 					"Show Tasks" => ctx.app.show_tasks(),
	// 					"Clear Tasks" => ctx.app.clear_tasks(),
	// 					"Stop Session" => {
	// 						ctx
	// 							.app
	// 							.engine
	// 							.runtime
	// 							.emit(e::Event::app(e::EventKind::SessionStop {
	// 								session: ctx.app.engine.runtime.session(),
	// 							}));
	// 					}
	// 					_ => {}
	// 				}
	// 			}
	// 		}
	// 	});
	// }
}
