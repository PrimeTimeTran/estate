use crate::app::{Runtime, ve::Veable};
#[cfg(not(target_arch = "wasm32"))]
use crate::{
	app::{event_channel::EventReceiver, monitor_native::StateMonitor, *},
	prelude::*,
	theme::palette,
	ui::{chart::*, *},
};

use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::{
	display::CGDisplay,
	event::{
		CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
		CGEventTapProxy, CGEventType, CGMouseButton, CallbackResult, *,
	},
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry::CGPoint,
};
use egui::Ui;
use egui_plot::{Bar, BarChart, Plot};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

// pub trait Veable {
// 	///      A trait implemented by types which agree to its contract.
// 	///
// 	///      Any type which implements this contract must provide `draw`.
// 	///      Code which depends on `Veable` can therefore o on that capability
// 	///      without needing to know how the concrete type implements it.
// 	///
// 	///      The implementation details belong to the concrete type; the caller
// 	///      only depends on the behavior promised by the contract.
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>);
// }

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

impl<R: Runtime> Ve<R> {
	///! Rust uses ownership,borrowing, and lifetimes to determine when values
	/// may be safely destroyed, allowing memory to be reclaimed deterministically
	/// without a garbage collector.
	pub fn new(view: impl Veable<R> + 'static) -> Self {
		let config = DEFAULT_CONFIG;
		Self {
			activity_bar: Region::fixed(DebugPanel::new("ACTIVITY"), config.activity_bar.size),
			dock_left: Panel::new(
				Region::resizable(Sidebar::new(), config.dock_left.size, 0.0, 600.0).with_fill(config.bg),
			)
			.with_open(config.dock_left.active),
			primary_bar: Region::fixed(DebugPanel::new("TABS"), config.primary_bar.size),
			secondary_bar: Region::fixed(DebugPanel::new("BREADCRUMBS"), config.activity_bar.size),
			main: Region::content(view).with_padding(8 as i32),
			bottom_panel: Panel::new(Region::resizable(
				WaterfallChart::new(),
				config.bottom_panel.size,
				0.0,
				600.0,
			)),
			status_bar: Region::fixed(DebugPanel::new("STATUS BAR"), config.status_bar.size)
				.with_fill(config.bg)
				.with_top_border(true),
			dock_right: Panel::new(
				Region::resizable(DebugPanel::new("RIGHT"), config.dock_right.size, 0.0, 600.0)
					.with_fill(config.bg),
			)
			.with_open(config.dock_right.active),
		}
	}
	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		/// Forwards the drawing contract to the concrete implementation.
		///
		/// `Ve` doesn't know how the view is drawn. It only knows that the
		/// contained implementation satisfies `Veable`.
		let mouse_pos = ui.input(|i| i.pointer.hover_pos());
		let available = ui.available_rect_before_wrap();
		ui.painter().rect_filled(available, 0.0, DEFAULT_CONFIG.bg);

		let layout = self.calculate_region_boundaries(available);
		let VeLayout {
			activity_bar,
			bottom_panel,
			dock_left,
			dock_right,
			main,
			primary_bar,
			secondary_bar,
			status_bar,
		} = layout;
		let cursor_target = self.cursor_target(mouse_pos, &layout);

		ctx.input = VeInputState {
			cursor_pos: mouse_pos,
			cursor_target,
			shift_held: ui.input(|i| i.modifiers.shift),
			ctrl_held: ui.input(|i| i.modifiers.ctrl),
			alt_held: ui.input(|i| i.modifiers.alt),
			command_held: ui.input(|i| i.modifiers.command),
			primary_down: ui.input(|i| i.pointer.primary_down()),
		};
		if self.dock_left.open {
			Self::draw_panel(ui, ctx, dock_left, &mut self.dock_left);
			Self::resize_region(
				ui,
				"dock_left_resize",
				dock_left,
				&mut self.dock_left.region,
				ResizeEdge::Right,
				1.0,
			);
		}
		Self::draw_region(ui, ctx, primary_bar, &mut self.primary_bar);
		Self::draw_region(ui, ctx, secondary_bar, &mut self.secondary_bar);
		Self::draw_region(ui, ctx, main, &mut self.main);
		if self.bottom_panel.open {
			Self::draw_panel(ui, ctx, bottom_panel, &mut self.bottom_panel);
			Self::resize_region(
				ui,
				"bottom_panel_resize",
				bottom_panel,
				&mut self.bottom_panel.region,
				ResizeEdge::Top,
				-1.0,
			);
		}
		if self.dock_right.open {
			Self::draw_panel(ui, ctx, dock_right, &mut self.dock_right);
			Self::resize_region(
				ui,
				"dock_right_resize",
				dock_right,
				&mut self.dock_right.region,
				ResizeEdge::Left,
				-1.0,
			);
		}
		Self::draw_region(ui, ctx, status_bar, &mut self.status_bar);
	}
	fn draw_region(
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R>,
		rect: egui::Rect,
		region: &mut Region<R>,
	) {
		let fill = region.fill.unwrap_or(DEFAULT_CONFIG.bg);
		ui.painter().rect_filled(rect, 0.0, fill);
		if region.top_border {
			ui.painter().line_segment(
				[
					egui::pos2(rect.left(), rect.top()),
					egui::pos2(rect.right(), rect.top()),
				],
				egui::Stroke::new(1.0, DEFAULT_CONFIG.surface),
			);
		}
		let content_rect = region.content_rect(rect);
		Self::draw_view(ui, content_rect, &mut *region.content, ctx);
	}
	fn draw_view(
		ui: &mut egui::Ui,
		rect: egui::Rect,
		view: &mut dyn Veable<R>,
		ctx: &mut AppContext<'_, R>,
	) {
		let mut child = ui.new_child(
			egui::UiBuilder::new()
				.max_rect(rect)
				.layout(egui::Layout::top_down(egui::Align::LEFT)),
		);

		view.draw(&mut child, ctx);
	}
	fn draw_panel(
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R>,
		rect: egui::Rect,
		panel: &mut Panel<R>,
	) {
		if !panel.open {
			return;
		}

		if let Some(fill) = panel.region.fill {
			ui.painter().rect_filled(rect, 0.0, fill);
		}

		Self::draw_view(ui, rect, &mut *panel.region.content, ctx);
	}

	fn calculate_region_boundaries(&mut self, available: egui::Rect) -> VeLayout {
		// =========================================================
		// Bottom Status Bar
		// =========================================================
		let status_bar_height = DEFAULT_CONFIG.status_bar.size;
		let activity_bar_width = if self.activity_bar.is_docked {
			self.activity_bar.size
		} else {
			0.0
		};

		let activity_bar = egui::Rect::from_min_max(
			available.min,
			egui::pos2(available.left() + activity_bar_width, available.bottom()),
		);

		let workspace_rect = egui::Rect::from_min_max(
			egui::pos2(available.left() + activity_bar_width, available.top()),
			egui::pos2(available.right(), available.bottom() - status_bar_height),
		);
		let status_bar = egui::Rect::from_min_max(
			egui::pos2(available.left(), workspace_rect.bottom()),
			available.max,
		);

		// =========================================================
		// Workspace: left / center / right
		// =========================================================
		let min_main_width = 100.0;
		let requested_left = if self.dock_left.open {
			self.dock_left.region.size
		} else {
			0.0
		};
		let requested_right = if self.dock_right.open {
			self.dock_right.region.size
		} else {
			0.0
		};
		let available_side_width = (workspace_rect.width() - min_main_width).max(0.0);
		let requested_total = requested_left + requested_right;
		let scale = if requested_total > available_side_width {
			available_side_width / requested_total
		} else {
			1.0
		};
		let left_width = requested_left * scale;
		let right_width = requested_right * scale;
		let dock_left = egui::Rect::from_min_max(
			workspace_rect.min,
			egui::pos2(workspace_rect.left() + left_width, workspace_rect.bottom()),
		);
		let dock_right = egui::Rect::from_min_max(
			egui::pos2(workspace_rect.right() - right_width, workspace_rect.top()),
			workspace_rect.max,
		);
		let center_rect = egui::Rect::from_min_max(
			egui::pos2(workspace_rect.left() + left_width, workspace_rect.top()),
			egui::pos2(
				workspace_rect.right() - right_width,
				workspace_rect.bottom(),
			),
		);
		// =========================================================
		// CENTER: tabs / breadcrumbs / main / bottom
		// =========================================================
		let tabs_height = DEFAULT_CONFIG.primary_bar.size;
		let breadcrumbs_height = DEFAULT_CONFIG.secondary_bar.size;
		let min_main_height = 100.0;
		// =========================================================
		// Tabs
		// =========================================================
		let primary_bar = egui::Rect::from_min_max(
			center_rect.min,
			egui::pos2(center_rect.right(), center_rect.top() + tabs_height),
		);
		// =========================================================
		// Breadcrumbs
		// =========================================================
		let secondary_bar = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), primary_bar.bottom()),
			egui::pos2(
				center_rect.right(),
				primary_bar.bottom() + breadcrumbs_height,
			),
		);
		// =========================================================
		// Center content
		// =========================================================
		let content_rect = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), secondary_bar.bottom()),
			center_rect.max,
		);
		// =========================================================
		// Main / bottom
		// =========================================================
		let bottom_height = if self.bottom_panel.open {
			self
				.bottom_panel
				.region
				.size
				.min((content_rect.height() - min_main_height).max(0.0))
		} else {
			0.0
		};
		let main = egui::Rect::from_min_max(
			content_rect.min,
			egui::pos2(content_rect.right(), content_rect.bottom() - bottom_height),
		);
		let bottom_panel = egui::Rect::from_min_max(
			egui::pos2(content_rect.left(), content_rect.bottom() - bottom_height),
			content_rect.max,
		);
		VeLayout {
			activity_bar,
			bottom_panel,
			dock_left,
			dock_right,
			main,
			primary_bar,
			secondary_bar,
			status_bar,
		}
	}
	fn resize_handle(ui: &mut egui::Ui, id: &str, rect: egui::Rect, mut resize: impl FnMut(f32)) {
		let cursor = if id == "bottom_panel_resize" {
			egui::CursorIcon::ResizeVertical
		} else {
			egui::CursorIcon::ResizeHorizontal
		};
		let id = egui::Id::new(id);
		let response = ui.interact(rect, id, egui::Sense::drag());
		if response.hovered() || response.dragged() {
			ui.ctx().set_cursor_icon(cursor);
		}
		let hovered = response.hovered();
		let dragged = response.dragged();
		let stroke = if hovered || dragged {
			ui.visuals().widgets.active.bg_stroke
		} else {
			ui.visuals().widgets.noninteractive.bg_stroke
		};
		ui.painter().rect_filled(rect, 0.0, stroke.color);
		if dragged {
			let delta = match cursor {
				egui::CursorIcon::ResizeVertical => response.drag_motion().y,
				_ => response.drag_motion().x,
			};
			resize(delta);
		}
	}
	fn resize_region(
		ui: &mut egui::Ui,
		id: &str,
		rect: egui::Rect,
		region: &mut Region<R>,
		edge: ResizeEdge,
		direction: f32,
	) {
		if !region.resizable {
			return;
		}
		let handle = match edge {
			ResizeEdge::Left => egui::Rect::from_min_max(
				egui::pos2(rect.left() - 3.0, rect.top()),
				egui::pos2(rect.left() + 3.0, rect.bottom()),
			),
			ResizeEdge::Right => egui::Rect::from_min_max(
				egui::pos2(rect.right() - 3.0, rect.top()),
				egui::pos2(rect.right() + 3.0, rect.bottom()),
			),
			ResizeEdge::Top => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.top() - 3.0),
				egui::pos2(rect.right(), rect.top() + 3.0),
			),
			ResizeEdge::Bottom => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.bottom() - 3.0),
				egui::pos2(rect.right(), rect.bottom() + 3.0),
			),
		};
		Self::resize_handle(ui, id, handle, |delta| {
			let delta = match edge {
				ResizeEdge::Left | ResizeEdge::Right => delta,
				ResizeEdge::Top | ResizeEdge::Bottom => delta,
			};
			region.size = (region.size + delta * direction).clamp(region.min_size, region.max_size);
		});
	}
	pub fn cursor_target(&self, pos: Option<egui::Pos2>, layout: &VeLayout) -> CursorTarget {
		let Some(pos) = pos else {
			return CursorTarget::None;
		};

		if self.activity_bar.is_docked && layout.activity_bar.contains(pos) {
			CursorTarget::ActivityBar
		} else if self.dock_left.open && layout.dock_left.contains(pos) {
			CursorTarget::DockLeft
		} else if layout.primary_bar.contains(pos) {
			CursorTarget::PrimaryBar
		} else if layout.secondary_bar.contains(pos) {
			CursorTarget::SecondaryBar
		} else if layout.main.contains(pos) {
			CursorTarget::Main
		} else if self.bottom_panel.open && layout.bottom_panel.contains(pos) {
			CursorTarget::BottomPanel
		} else if self.dock_right.open && layout.dock_right.contains(pos) {
			CursorTarget::DockRight
		} else if layout.status_bar.contains(pos) {
			CursorTarget::StatusBar
		} else {
			CursorTarget::None
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

///! The first concrete implementation of Veable is here.
///!
///! EguiVeable defines it's own state which is specific to its own implementation
///! and the correponding methods which operate on those properties.
///!
///! The draw method is the gateway for this struct to inject behavior thats independent of the
///! generic base and unique to itself as package or an instance of Veable.
#[derive(Clone, Debug, Default)]
pub struct EguiVeable {
	state: EstateState,
	top_tab: DevTopTab,
	side_tab: DevSideTab,
}
impl Veable<NativeRuntime> for EguiVeable {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		self.draw_ui(ui);
	}
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
				tracing::info!(">>> TAB CLICKED: {:?}", tab);
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
			tracing::info!(
				target: "estate::app",
				"Copy button: hovered={} clicked={} enabled={}",
				response.hovered(),
				response.clicked(),
				response.enabled(),
			);
			if response.clicked() {
				tracing::info!(target: "estate::app", "CLICKED COPY");
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

// -----------------------------------------------------------------------------
// ORACLE
// -----------------------------------------------------------------------------

pub struct Oracle {
	pub active_focus: FocusedPane,
	dirty: bool,
	error: Option<String>,
	last_direction: String,
	last_loaded: Option<SystemTime>,
	scroll_x: f32,
	scroll_y: f32,
	gesture: GestureController,
}

impl Oracle {
	pub fn new() -> Self {
		Self {
			active_focus: FocusedPane::MainEditor,
			dirty: false,
			error: None,
			last_direction: String::new(),
			last_loaded: None,
			scroll_x: 0.0,
			scroll_y: 0.0,
			gesture: GestureController::new(),
		}
	}

	// -------------------------------------------------------------------------
	// INPUT
	// -------------------------------------------------------------------------

	fn inspect_trackpad(
		&mut self,
		ui: &mut egui::Ui,
		ctx: &AppContext<'_, NativeRuntime>,
	) -> TrackpadState {
		self.gesture.inspect(ui, &ctx.input)
	}

	// -------------------------------------------------------------------------
	// UI
	// -------------------------------------------------------------------------

	fn draw_ui(&mut self, ui: &mut egui::Ui, ctx: &AppContext<'_, NativeRuntime>) {
		self.draw_header(ui);

		let trackpad = self.inspect_trackpad(ui, ctx);

		self.draw_telemetry(ui, &trackpad);
		self.draw_trigger_preview(ui, &trackpad);
		self.draw_controls(ui);
	}

	fn draw_header(&mut self, ui: &mut egui::Ui) {
		ui.heading("Trackpad & Gesture Telemetry PoC");

		ui.label(
			"Goal: Inspect raw multi-axis vectors, modifiers, cursor position, hover target, and focus.",
		);

		ui.separator();
	}

	// -------------------------------------------------------------------------
	// TELEMETRY
	// -------------------------------------------------------------------------

	fn draw_telemetry(&mut self, ui: &mut egui::Ui, trackpad: &TrackpadState) {
		ui.columns(2, |columns| {
			// =================================================================
			// INPUT
			// =================================================================

			columns[0].group(|ui| {
				ui.heading("Input");

				ui.add_space(4.0);

				ui.label(format!("Scroll Delta X: {:.2}", trackpad.delta.x));

				ui.label(format!("Scroll Delta Y: {:.2}", trackpad.delta.y));

				ui.label(format!("Primary Axis: {}", trackpad.primary_axis()));

				ui.label(format!("Direction: {:?}", trackpad.direction));

				ui.add_space(8.0);

				Self::draw_modifier(ui, "Shift", trackpad.shift_held);
				Self::draw_modifier(ui, "Ctrl", trackpad.ctrl_held);
				Self::draw_modifier(ui, "Alt", trackpad.alt_held);
				Self::draw_modifier(ui, "Command", trackpad.command_held);
			});

			// =================================================================
			// CURSOR / FOCUS
			// =================================================================

			columns[1].group(|ui| {
				ui.heading("Cursor & Focus");

				ui.add_space(4.0);

				// -------------------------------------------------------------
				// Cursor position
				// -------------------------------------------------------------

				match trackpad.mouse_pos {
					Some(pos) => {
						ui.label(format!("Cursor X: {:.1}", pos.x));
						ui.label(format!("Cursor Y: {:.1}", pos.y));
						ui.label(format!("Cursor: ({:.1}, {:.1})", pos.x, pos.y));
					}

					None => {
						ui.label("Cursor: Outside viewport");
					}
				}

				ui.add_space(8.0);

				// -------------------------------------------------------------
				// Cursor target
				// -------------------------------------------------------------

				ui.horizontal(|ui| {
					ui.label("Hovered:");

					let hovered = trackpad.hovered != CursorTarget::None;

					ui.colored_label(
						if hovered {
							egui::Color32::LIGHT_GREEN
						} else {
							egui::Color32::GRAY
						},
						trackpad.hovered_name(),
					);
				});

				// -------------------------------------------------------------
				// Focus
				// -------------------------------------------------------------

				ui.horizontal(|ui| {
					ui.label("Focus:");

					ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{:?}", trackpad.focus));
				});

				ui.add_space(8.0);

				// -------------------------------------------------------------
				// Gesture state
				// -------------------------------------------------------------

				ui.label(format!(
					"Side Panel Width: {:.1}px",
					self.gesture.state.side_panel_width
				));

				ui.label(format!(
					"Secondary Scroll: {:.1}",
					self.gesture.state.secondary_scroll_offset
				));
			});
		});

		ui.add_space(12.0);
		ui.separator();
	}

	fn draw_modifier(ui: &mut egui::Ui, name: &str, held: bool) {
		ui.horizontal(|ui| {
			ui.label(format!("{name}:"));

			ui.colored_label(
				if held {
					egui::Color32::LIGHT_GREEN
				} else {
					egui::Color32::GRAY
				},
				if held { "HELD" } else { "Released" },
			);
		});
	}

	// -------------------------------------------------------------------------
	// TRIGGER PREVIEW
	// -------------------------------------------------------------------------

	fn draw_trigger_preview(&mut self, ui: &mut egui::Ui, trackpad: &TrackpadState) {
		ui.group(|ui| {
			ui.heading("Target Action Trigger Preview");

			ui.add_space(4.0);

			// First verify the modifier independently of the gesture.
			if trackpad.shift_held {
				ui.colored_label(egui::Color32::LIGHT_GREEN, "SHIFT DETECTED");
			} else {
				ui.label("SHIFT NOT HELD");
			}

			ui.add_space(4.0);

			let is_horizontal = trackpad.delta.x.abs() > trackpad.delta.y.abs();

			let is_vertical = trackpad.delta.y.abs() > trackpad.delta.x.abs();

			if trackpad.shift_held && is_horizontal {
				ui.colored_label(
					egui::Color32::LIGHT_BLUE,
					format!(
						"⚡ TRIGGER MATCH: Resize Panel Vector -> {:.2}px",
						trackpad.delta.x
					),
				);
			} else if trackpad.shift_held && is_vertical {
				ui.colored_label(
					egui::Color32::LIGHT_GREEN,
					format!(
						"⚡ TRIGGER MATCH: Cross-Scroll Secondary Pane -> {:.2} units",
						trackpad.delta.y
					),
				);
			} else {
				ui.label("Waiting for scroll...");
			}
		});
	}

	fn draw_controls(&mut self, _ui: &mut egui::Ui) {}

	fn draw_status_bar(&mut self, ui: &mut egui::Ui) {
		ui.add_space(8.0);

		ui.horizontal(|ui| {
			if ui.button("Reset Telemetry States").clicked() {
				// Reset state here when you decide what "reset" means.
			}

			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				ui.label("PoC V1.0 - Ready for OS Daemon translation");
			});
		});
	}
}

// -----------------------------------------------------------------------------
// VEABLE
// -----------------------------------------------------------------------------

impl Veable<NativeRuntime> for Oracle {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		self.draw_ui(ui, ctx);
		self.draw_status_bar(ui);
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
	state: GestureState,
}

impl GestureController {
	pub fn new() -> Self {
		Self {
			state: GestureState::default(),
		}
	}
	fn inspect(&mut self, ui: &egui::Ui, input: &VeInputState) -> TrackpadState {
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
	fn direction(delta: egui::Vec2) -> ScrollDirection {
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
	fn hover_target(mouse_pos: Option<egui::Pos2>, viewport: egui::Rect) -> Option<CursorTarget> {
		let Some(pos) = mouse_pos else {
			return None;
		};
		if !viewport.contains(pos) {
			return None;
		}
		// Replace this with your actual target rectangles.
		Some(CursorTarget::Main)
	}
	fn focus_for_target(target: CursorTarget) -> FocusedPane {
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	#[default]
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
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
	fn primary_axis(&self) -> &'static str {
		if self.delta.x.abs() > self.delta.y.abs() {
			"Horizontal (X)"
		} else if self.delta.y.abs() > self.delta.x.abs() {
			"Vertical (Y)"
		} else {
			"None"
		}
	}
	fn hovered_name(&self) -> &'static str {
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
			buttons: vec!["New Task", "Show Tasks", "Clear Tasks"],
		}
	}
}

impl<R: Runtime> Veable<R> for Sidebar {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		ui.vertical(|ui| {
			for button in &self.buttons {
				if ui.button(*button).clicked() {
					match *button {
						"New Task" => ctx.app.new_task(),
						"Show Tasks" => ctx.app.show_tasks(),
						"Clear Tasks" => ctx.app.clear_tasks(),
						_ => {}
					}
				}
			}
		});
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VeInputState {
	pub alt_held: bool,
	pub command_held: bool,
	pub ctrl_held: bool,
	pub cursor_pos: Option<egui::Pos2>,
	pub cursor_target: CursorTarget,
	pub primary_down: bool,
	pub shift_held: bool,
}

pub struct WaterfallChart;
impl WaterfallChart {
	pub fn new() -> Self {
		Self
	}
	pub fn draw_chart<'a>(&self, ui: &mut Ui, jobs: impl Iterator<Item = &'a Job>) {
		let jobs: Vec<&Job> = jobs.collect();
		if jobs.is_empty() {
			ui.centered_and_justified(|ui| {
				ui.label("No job history");
			});
			return;
		}
		let now = EstateState::now();
		let mut timed_jobs = Vec::new();
		for job in jobs {
			let Some(started_at) = job.started_at else {
				continue;
			};
			let start = started_at as f64;
			let end = job.completed_at.unwrap_or(now) as f64;
			timed_jobs.push((job, start, end.max(start + 1.0)));
		}
		if timed_jobs.is_empty() {
			ui.centered_and_justified(|ui| {
				ui.label("No timed jobs");
			});
			return;
		}
		// Sort chronologically so we can pack jobs into the
		// smallest possible number of horizontal lanes.
		timed_jobs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
		// Each lane stores the end time of its last job.
		let mut lanes: Vec<f64> = Vec::new();
		let mut bars = Vec::with_capacity(timed_jobs.len());
		let mut min_time = f64::MAX;
		let mut max_time = f64::MIN;
		for (job, start, end) in timed_jobs {
			min_time = min_time.min(start);
			max_time = max_time.max(end);
			// Reuse the first lane whose previous job has
			// already finished.
			let lane = lanes
				.iter()
				.position(|lane_end| *lane_end <= start)
				.unwrap_or_else(|| {
					lanes.push(0.0);
					lanes.len() - 1
				});

			lanes[lane] = end;

			let duration = end - start;

			bars.push(
				Bar::new(lane as f64, duration)
					.horizontal()
					.base_offset(start)
					.width(0.7)
					.name(job.kind.name()),
			);
		}

		let padding = ((max_time - min_time) * 0.05).max(1.0);

		let lane_count = lanes.len() as f64;

		Plot::new("job_timeline")
			.height(280.0)
			// Horizontal range is time.
			.include_x(min_time - padding)
			.include_x(max_time + padding)
			// Vertical range is ONLY the lanes we actually needed.
			.include_y(-0.75)
			.include_y((lane_count - 1.0).max(0.0) + 0.75)
			.allow_drag(true)
			.allow_zoom(true)
			.allow_scroll(true)
			// Don't allow independent Y-axis zooming.
			.allow_axis_zoom_drag(false)
			.show_x(true)
			.show_y(false)
			.legend(egui_plot::Legend::default())
			.x_axis_formatter(|mark, _range| format_timestamp(mark.value))
			.show(ui, |plot_ui| {
				plot_ui.bar_chart(BarChart::new("jobs", bars).horizontal());
			});
	}
}
impl<R: Runtime> Veable<R> for WaterfallChart {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		if ctx.poll_state() {
			ui.ctx().request_repaint();
		}
		ui.heading("Job History");
		let state = ctx.state();
		self.draw_chart(ui, state.jobs.iter());
	}
}
