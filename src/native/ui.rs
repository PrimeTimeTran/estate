use crate::{
	app,
	app::{Runtime, state::EstateState},
	e,
	native::prelude::*,
	ui::{*,Layout, r#trait::*},

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
impl<R: Runtime> Screen<R> for EguiVeable {
  fn configure(
		&mut self,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
	  todo!("")
	}

	fn update(
		&mut self,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
	todo!("")
	}

	fn event(
		&mut self,
		event: &e::Event,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
	todo!("")
	}
}
impl<R: Runtime> ViewTrait<R> for EguiVeable {
 	fn draw(
		&mut self,
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R>,
	) {
	  self.draw_ui(ui);
	}

	fn update(
		&mut self,
		ctx: &mut AppContext<'_, R>,
	){}

	fn event(
		&mut self,
		event: &e::Event,
		ctx: &mut AppContext<'_, R>,
	){}
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

#[derive(Debug)]
pub struct GestureState {
	pub active_focus: FocusedPane,
	pub side_panel_width: f32,
	pub secondary_scroll_offset: f32,
	pub last_delta: egui::Vec2,
	pub last_direction: ScrollDirection,
}
impl Default for GestureState {
	fn default() -> Self {
		Self {
			active_focus: FocusedPane::MainEditor,
			side_panel_width: 300.0,
			secondary_scroll_offset: 0.0,
			last_delta: egui::Vec2::ZERO,
			last_direction: ScrollDirection::None,
		}
	}
}
#[derive(Debug)]
pub struct GestureController {
	pub state: GestureState,
}
impl GestureController {
	pub(crate) fn new() -> Self {
		Self {
			state: GestureState::default(),
		}
	}
	pub(crate) fn inspect(&mut self, ui: &egui::Ui, input: &IOState) -> TrackpadState {
		let current = ui.input(|input| {
			let delta = input.smooth_scroll_delta;
			(
				delta,
				input.modifiers.shift,
				input.modifiers.ctrl,
				input.modifiers.alt,
				input.modifiers.command,
				input.pointer.hover_pos(),
			)
		});
		let (delta, shift, ctrl, alt, command, mouse_pos) = current;
		if delta != egui::Vec2::ZERO {
			self.state.last_delta = delta;
			self.state.last_direction = Self::direction(delta);
		}
		TrackpadState {
			alt_held: alt,
			clicked: None,
			command_held: input.command_held,
			ctrl_held: ctrl,
			delta: self.state.last_delta,
			direction: self.state.last_direction,
			focus: self.state.active_focus,
			hovered: input.cursor_target,
			mouse_pos: input.cursor_pos,
			shift_held: shift,
		}
	}
	pub(crate) fn direction(delta: egui::Vec2) -> ScrollDirection {
		if delta.x == 0.0 && delta.y == 0.0 {
			ScrollDirection::None
		} else if delta.x.abs() > delta.y.abs() {
			if delta.x > 0.0 {
				ScrollDirection::Right
			} else {
				ScrollDirection::Left
			}
		} else if delta.y > 0.0 {
			ScrollDirection::Down
		} else {
			ScrollDirection::Up
		}
	}
	pub(crate) fn hover_target(
		mouse_pos: Option<egui::Pos2>,
		viewport: egui::Rect,
	) -> Option<CursorTarget> {
		let Some(pos) = mouse_pos else {
			return None;
		};
		if !viewport.contains(pos) {
			return None;
		}
		Some(CursorTarget::Main)
	}
	pub(crate) fn focus_for_target(target: CursorTarget) -> FocusedPane {
		match target {
			CursorTarget::Main => FocusedPane::MainEditor,
			CursorTarget::DockLeft | CursorTarget::DockRight => FocusedPane::SidePanel,
			CursorTarget::BottomPanel
			| CursorTarget::ActivityBar
			| CursorTarget::PrimaryBar
			| CursorTarget::SecondaryBar
			| CursorTarget::StatusBar
			| CursorTarget::None => FocusedPane::Unknown,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct TrackpadState {
	pub delta: egui::Vec2,
	pub direction: ScrollDirection,
	pub shift_held: bool,
	pub ctrl_held: bool,
	pub alt_held: bool,
	pub command_held: bool,
	pub mouse_pos: Option<egui::Pos2>,
	pub hovered: CursorTarget,
	pub clicked: Option<CursorTarget>,
	pub focus: FocusedPane,
}
impl TrackpadState {
	pub(crate) fn primary_axis(&self) -> &'static str {
		if self.delta.x.abs() > self.delta.y.abs() {
			"Horizontal (X)"
		} else if self.delta.y.abs() > self.delta.x.abs() {
			"Vertical (Y)"
		} else {
			"None"
		}
	}
	pub(crate) fn hovered_name(&self) -> &'static str {
		self.hovered.name()
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
	None,
	Up,
	Down,
	Left,
	Right,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorTarget {
	ActivityBar,
	DockLeft,
	#[default]
	Main,
	PrimaryBar,
	SecondaryBar,
	BottomPanel,
	StatusBar,
	DockRight,
	None,
}

impl CursorTarget {
	pub fn name(self) -> &'static str {
		match self {
			Self::ActivityBar => "Activity Bar",
			Self::DockLeft => "Dock Left",
			Self::Main => "Main",
			Self::PrimaryBar => "Primary Bar",
			Self::SecondaryBar => "Secondary Bar",
			Self::BottomPanel => "Bottom Panel",
			Self::StatusBar => "Status Bar",
			Self::DockRight => "Dock Right",
			Self::None => "Nothing",
		}
	}
}
#[derive(Debug, Clone, Copy)]
pub struct ScrollRedirectState {
	pub active: bool,
	pub redirected: bool,
	pub original_position: CGPoint,
	pub target_position: CGPoint,
}

pub fn scroll_state() -> &'static Mutex<ScrollRedirectState> {
	SCROLL_STATE.get_or_init(|| {
		Mutex::new(ScrollRedirectState {
			active: false,
			redirected: false,
			original_position: CGPoint { x: 0.0, y: 0.0 },
			target_position: CGPoint { x: 0.0, y: 0.0 },
		})
	})
}
pub fn spawn_global_cursor_daemon(proxy: EventLoopProxy<AppEvent>) {
	std::thread::spawn(move || {
		// May need to grant permissions multiple times if the user runs the app from a different tool?
		// When I run from Zed/VSCode it runs fine. But Ghosty it doesn't The scroll tracker doesn't activate
		let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
		if !trusted {
			return;
		}
		let callback = move |_proxy_cg: CGEventTapProxy,
		                     event_type: CGEventType,
		                     event: &CGEvent|
		      -> CallbackResult {
			match event_type {
				CGEventType::MouseMoved => {
					if REDIRECTING_SCROLL.load(Ordering::Relaxed) {
						return CallbackResult::Keep;
					}
					let location = event.location();
					let _ = proxy.send_event(AppEvent::CursorPosition {
						x: location.x,
						y: location.y,
					});
					CallbackResult::Keep
				}
				CGEventType::FlagsChanged => {
					let flags = event.get_flags();
					let shift = flags.contains(CGEventFlags::CGEventFlagShift);
					let ctrl = flags.contains(CGEventFlags::CGEventFlagControl);
					let alt = flags.contains(CGEventFlags::CGEventFlagAlternate);
					let command = flags.contains(CGEventFlags::CGEventFlagCommand);
					// Get the previous state BEFORE updating it.
					let was_shift_down = SHIFT_HELD.swap(shift, Ordering::Relaxed);
					// -------------------------------------------------------------
					// Tell the application immediately about modifier state.
					// -------------------------------------------------------------
					let _ = proxy.send_event(AppEvent::ModifiersChanged {
						shift,
						ctrl,
						alt,
						command,
					});
					// -------------------------------------------------------------
					// SHIFT DOWN
					// -------------------------------------------------------------
					if shift && !was_shift_down {
						let location = event.location();
						let bounds = CGDisplay::main().bounds();
						let midpoint = bounds.origin.x + bounds.size.width * 0.5;
						let target = if location.x < midpoint {
							ScreenPosition::Right
						} else {
							ScreenPosition::Left
						};
						let target_position = target_position(bounds, target, location.y);
						let mut state = scroll_state().lock().unwrap();
						state.active = true;
						state.redirected = true;
						state.original_position = location;
						state.target_position = target_position;
						if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
							if let Ok(move_event) = CGEvent::new_mouse_event(
								source,
								CGEventType::MouseMoved,
								target_position,
								CGMouseButton::Left,
							) {
								move_event.post(CGEventTapLocation::HID);
							}
						}
					}
					// -------------------------------------------------------------
					// SHIFT UP
					// -------------------------------------------------------------

					if !shift && was_shift_down {
						let mut state = scroll_state().lock().unwrap();

						let original = state.original_position;

						if state.active {
							if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
								if let Ok(restore_event) = CGEvent::new_mouse_event(
									source,
									CGEventType::MouseMoved,
									original,
									CGMouseButton::Left,
								) {
									restore_event.post(CGEventTapLocation::HID);
								}
							}
						}

						state.active = false;
						state.redirected = false;
					}

					CallbackResult::Keep
				}
				CGEventType::KeyDown => {
					let keycode =
						event.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE);
					match keycode {
						// 18 => {
						// 	println!("Key '1' pressed");
						// 	move_cursor_to(ScreenPosition::Left);
						// }
						// 19 => {
						// 	println!("Key '2' pressed");
						// 	move_cursor_to(ScreenPosition::Center);
						// }
						// 20 => {
						// 	println!("Key '3' pressed");
						// 	move_cursor_to(ScreenPosition::Right);
						// }
						_ => {}
					}
					CallbackResult::Keep
				}
				CGEventType::ScrollWheel => {
					if !SHIFT_HELD.load(Ordering::Relaxed) {
						return CallbackResult::Keep;
					}

					let state = scroll_state().lock().unwrap();

					if !state.active {
						return CallbackResult::Keep;
					}
					CallbackResult::Keep
				}
				_ => CallbackResult::Keep,
			}
		};
		let tap = match CGEventTap::new(
			CGEventTapLocation::HID,
			CGEventTapPlacement::HeadInsertEventTap,
			CGEventTapOptions::Default,
			vec![
				CGEventType::ScrollWheel,
				CGEventType::FlagsChanged,
				CGEventType::MouseMoved,
				CGEventType::KeyDown,
			],
			callback,
		) {
			Ok(tap) => tap,
			Err(error) => {
				eprintln!("❌ Failed to create CGEventTap: {:?}", error);
				return;
			}
		};
		unsafe {
			let port = tap.mach_port();
			let source = match port.create_runloop_source(0) {
				Ok(source) => source,
				Err(_) => {
					eprintln!("❌ Failed to create CFRunLoopSource");
					return;
				}
			};
			let run_loop = CFRunLoop::get_current();
			run_loop.add_source(&source, kCFRunLoopCommonModes);
			tap.enable();
			CFRunLoop::run_current();
		}
	});
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
impl<R: Runtime> Screen<R> for Sidebar {
  fn configure(
		&mut self,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
		// Configure the regions this screen uses.
	}

	fn update(
		&mut self,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
	}

	fn event(
		&mut self,
		event: &e::Event,
		layout: &mut Layout<R>,
		ctx: &mut AppContext<'_, R>,
	) {
	}
	// fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
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

#[derive(Debug, Default, Clone, Copy)]

pub struct IOState {
	pub alt_held: bool,
	pub command_held: bool,
	pub ctrl_held: bool,
	pub cursor_pos: Option<egui::Pos2>,
	pub cursor_target: CursorTarget,
	pub primary_down: bool,
	pub shift_held: bool,
}
