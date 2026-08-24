use crate::prelude::*;

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
use egui_plot::{Bar, BarChart, Line, Plot, Points};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;
///      A trait implemented by types which agree to its contract.
///
///      Any type which implements this contract must provide `draw`.
///      Code which depends on `Veable` can therefore rely on that capability
///      without needing to know how the concrete type implements it.
///
///      The implementation details belong to the concrete type; the caller
///      only depends on the behavior promised by the contract.
pub trait Veable {
	fn draw(&mut self, ui: &mut egui::Ui);
}
///      A type-erased container for any concrete `Veable`.
///
///      `Box<dyn Veable>` stores the concrete implementation on the heap while
///      exposing only the `Veable` interface to callers. This allows different
///      concrete implementations to be substituted without changing the code
///      which consumes them.
pub struct Ve {
	// Top left to bottom right ordering for mental model.
	pub activity_bar: Region,
	pub dock_left: Panel,
	pub main: Region,
	pub primary_bar: Region,
	pub secondary_bar: Region,
	pub bottom_panel: Panel,
	pub status_bar: Region,
	pub dock_right: Panel,
}
impl Ve {
	///! Rust uses ownership, borrowing, and lifetimes to determine when values
	/// may be safely destroyed, allowing memory to be reclaimed deterministically
	/// without a garbage collector.
	pub fn new(view: impl Veable + 'static) -> Self {
		let config = DEFAULT_CONFIG;
		Self {
			activity_bar: Region::fixed(DebugPanel::new("ACTIVITY"), config.activity_bar.size),
			dock_left: Panel::new(
				Region::resizable(DebugPanel::new("LEFT"), config.dock_left.size, 0.0, 600.0)
					.with_fill(config.bg),
			)
			.with_open(config.dock_left.active),
			primary_bar: Region::fixed(DebugPanel::new("TABS"), config.primary_bar.size),
			secondary_bar: Region::fixed(DebugPanel::new("BREADCRUMBS"), config.activity_bar.size),
			main: Region::content(view).with_padding(8 as i32),
			bottom_panel: Panel::new(Region::resizable(
				DebugPanel::new("BOTTOM"),
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
	/// Forwards the drawing contract to the concrete implementation.
	///
	/// `Ve` doesn't know how the view is drawn. It only knows that the
	/// contained implementation satisfies `Veable`.
	pub fn draw(&mut self, ui: &mut egui::Ui) {
		let available = ui.available_rect_before_wrap();
		ui.painter().rect_filled(available, 0.0, DEFAULT_CONFIG.bg);
		let (
			left_rect,
			right_rect,
			tabs_rect,
			breadcrumbs_rect,
			main_rect,
			bottom_rect,
			status_bar_rect,
		) = self.calculate_region_boundaries(available);
		if self.dock_left.open {
			Self::draw_panel(ui, left_rect, &mut self.dock_left);
			Self::resize_region(
				ui,
				"dock_left_resize",
				left_rect,
				&mut self.dock_left.region,
				ResizeEdge::Right,
				1.0,
			);
		}
		Self::draw_region(ui, tabs_rect, &mut self.primary_bar);
		Self::draw_region(ui, breadcrumbs_rect, &mut self.secondary_bar);
		Self::draw_region(ui, main_rect, &mut self.main);
		if self.bottom_panel.open {
			Self::draw_panel(ui, bottom_rect, &mut self.bottom_panel);
			Self::resize_region(
				ui,
				"bottom_panel_resize",
				bottom_rect,
				&mut self.bottom_panel.region,
				ResizeEdge::Top,
				-1.0,
			);
		}
		if self.dock_right.open {
			Self::draw_panel(ui, right_rect, &mut self.dock_right);
			Self::resize_region(
				ui,
				"dock_right_resize",
				right_rect,
				&mut self.dock_right.region,
				ResizeEdge::Left,
				-1.0,
			);
		}
		Self::draw_region(ui, status_bar_rect, &mut self.status_bar);
	}
	fn calculate_region_boundaries(
		&mut self,
		available: egui::Rect,
	) -> (
		egui::Rect,
		egui::Rect,
		egui::Rect,
		egui::Rect,
		egui::Rect,
		egui::Rect,
		egui::Rect,
	) {
		// =========================================================
		// Bottom Status Bar
		// =========================================================
		let status_bar_height = DEFAULT_CONFIG.status_bar.size;
		let workspace_rect = egui::Rect::from_min_max(
			available.min,
			egui::pos2(available.right(), available.bottom() - status_bar_height),
		);
		let status_bar_rect = egui::Rect::from_min_max(
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
		let left_rect = egui::Rect::from_min_max(
			workspace_rect.min,
			egui::pos2(workspace_rect.left() + left_width, workspace_rect.bottom()),
		);
		let right_rect = egui::Rect::from_min_max(
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
		let tabs_rect = egui::Rect::from_min_max(
			center_rect.min,
			egui::pos2(center_rect.right(), center_rect.top() + tabs_height),
		);
		// =========================================================
		// Breadcrumbs
		// =========================================================
		let breadcrumbs_rect = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), tabs_rect.bottom()),
			egui::pos2(center_rect.right(), tabs_rect.bottom() + breadcrumbs_height),
		);
		// =========================================================
		// Center content
		// =========================================================
		let content_rect = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), breadcrumbs_rect.bottom()),
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
		let main_rect = egui::Rect::from_min_max(
			content_rect.min,
			egui::pos2(content_rect.right(), content_rect.bottom() - bottom_height),
		);
		let bottom_rect = egui::Rect::from_min_max(
			egui::pos2(content_rect.left(), content_rect.bottom() - bottom_height),
			content_rect.max,
		);
		(
			left_rect,
			right_rect,
			tabs_rect,
			breadcrumbs_rect,
			main_rect,
			bottom_rect,
			status_bar_rect,
		)
	}
	fn draw_region(ui: &mut egui::Ui, rect: egui::Rect, region: &mut Region) {
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
		Self::draw_view(ui, content_rect, &mut *region.content);
	}
	fn draw_view(ui: &mut egui::Ui, rect: egui::Rect, view: &mut dyn Veable) {
		let mut child = ui.new_child(
			egui::UiBuilder::new()
				.max_rect(rect)
				.layout(egui::Layout::top_down(egui::Align::LEFT)),
		);
		view.draw(&mut child);
	}
	fn draw_panel(ui: &mut egui::Ui, rect: egui::Rect, panel: &mut Panel) {
		if !panel.open {
			return;
		}
		// Region owns visual styling.
		if let Some(fill) = panel.region.fill {
			ui.painter().rect_filled(rect, 0.0, fill);
		}
		// Region owns the actual content.
		Self::draw_view(ui, rect, &mut *panel.region.content);
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
		region: &mut Region,
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
impl Veable for EguiVeable {
	fn draw(&mut self, ui: &mut egui::Ui) {
		self.draw_ui(ui);
	}
}
impl EguiVeable {
	pub fn new() -> Self {
		Self {
			state: EstateState::load(),
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
pub struct GpuiVeable;
pub struct TaffyVeable;
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
#[derive(Clone, Debug, Deserialize)]
pub struct ChartsFile {
	pub charts: Vec<Chart>,
}
use ::serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Chart {
	Bar(BarData),
	Line(LineData),
	Pie(PieData),
	Scatter(ScatterData),
}
impl Chart {
	pub fn ui(&self, ui: &mut Ui) {
		match self {
			Chart::Bar(data) => data.ui(ui),
			Chart::Line(data) => data.ui(ui),
			Chart::Pie(data) => data.ui(ui),
			Chart::Scatter(data) => data.ui(ui),
		}
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct BarData {
	pub title: String,
	pub bars: Vec<ChartBar>,
}
impl BarData {
	pub fn ui(&self, ui: &mut Ui) {
		ui.heading(&self.title);
		let bars = self
			.bars
			.iter()
			.enumerate()
			.map(|(index, bar)| Bar::new(index as f64, bar.value))
			.collect::<Vec<_>>();
		let chart = BarChart::new("bars", bars);
		Plot::new("bar_chart")
			.height(ui.available_height() - 40.0)
			.show(ui, |plot_ui| {
				plot_ui.bar_chart(chart);
			});
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct LineData {
	pub title: String,
	pub points: Vec<ChartPoint>,
}
impl LineData {
	pub fn ui(&self, ui: &mut Ui) {
		ui.heading(&self.title);
		let points = self
			.points
			.iter()
			.map(|point| [point.x, point.y])
			.collect::<Vec<_>>();
		let line = Line::new("line", points);
		Plot::new("line_chart")
			.height(ui.available_height() - 40.0)
			.show(ui, |plot_ui| {
				plot_ui.line(line);
			});
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct PieData {
	pub title: String,
	pub slices: Vec<ChartSlice>,
}
impl PieData {
	pub fn ui(&self, ui: &mut Ui) {
		ui.heading(&self.title);
		let available = ui.available_size();
		let size = available.x.min(available.y);
		let radius = size * 0.35;
		let center = ui.available_rect_before_wrap().center();
		let total: f64 = self.slices.iter().map(|slice| slice.value).sum();
		if total <= 0.0 {
			ui.label("No data");
			return;
		}
		let painter = ui.painter();
		let mut start_angle = 0.0_f32;
		for (index, slice) in self.slices.iter().enumerate() {
			let fraction = slice.value / total;
			let sweep = fraction as f32 * std::f32::consts::TAU;
			let end_angle = start_angle + sweep;
			let points = Self::pie_slice_points(center, radius, start_angle, end_angle);
			painter.add(egui::Shape::convex_polygon(
				points,
				egui::Color32::from_rgb(
					((50 + index * 35) % 255) as u8,
					((100 + index * 45) % 255) as u8,
					((180 + index * 20) % 255) as u8,
				),
				egui::Stroke::NONE,
			));
			start_angle = end_angle;
		}
	}
	fn pie_slice_points(center: egui::Pos2, radius: f32, start: f32, end: f32) -> Vec<egui::Pos2> {
		let segments = 32;
		let mut points = Vec::with_capacity(segments + 2);
		points.push(center);
		for i in 0..=segments {
			let t = i as f32 / segments as f32;
			let angle = start + (end - start) * t;
			points.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
		}
		points
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct ScatterData {
	pub title: String,
	pub points: Vec<ChartPoint>,
}
impl ScatterData {
	pub fn ui(&self, ui: &mut Ui) {
		ui.heading(&self.title);
		let points = self
			.points
			.iter()
			.map(|point| [point.x, point.y])
			.collect::<Vec<_>>();
		let points = Points::new("scatter", points);
		Plot::new("scatter_chart")
			.height(ui.available_height() - 40.0)
			.show(ui, |plot_ui| {
				plot_ui.points(points);
			});
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct ChartBar {
	pub label: String,
	pub value: f64,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ChartSlice {
	pub label: String,
	pub value: f64,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ChartPoint {
	pub x: f64,
	pub y: f64,
}
impl ChartsFile {
	pub fn load(path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&contents)?)
	}
}
#[derive(Debug)]
pub struct Graphics {
	data_path: PathBuf,
	data: ChartsFile,
	dirty: bool,
	last_loaded: Option<SystemTime>,
	error: Option<String>,
	// Expose a receiver if your event loop wants to listen for changes,
	// or keep it internal if you poll it.
	rx: tokio::sync::mpsc::Receiver<()>,
	_watcher: RecommendedWatcher,
	scroll_x: f32,
	scroll_y: f32,
	last_direction: String,
	// PoC Interactive Layout States
	pub side_panel_width: f32, // Width of the expandable column
	pub active_focus: FocusedPane,
	pub secondary_scroll_offset: f32, // For testing cross-scrolling the "other" column
}
impl Graphics {
	pub fn new() -> Self {
		let path = "/Users/future/kb/project/crates/estate/src/data/chart.json";
		Self::from_path(path)
	}
	fn from_path(path: impl Into<PathBuf>) -> Self {
		let data_path = path.into();
		let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
		let mut oracle = Self {
			_watcher: Self::init_watcher(&data_path, tx).expect("Failed to initialize file watcher"),
			active_focus: FocusedPane::MainEditor,
			data: ChartsFile { charts: Vec::new() },
			data_path: data_path.clone(),
			dirty: false,
			error: None,
			last_direction: "".to_string(),
			last_loaded: None,
			rx,
			scroll_x: 0.0,
			scroll_y: 0.0,
			secondary_scroll_offset: 0.0,
			side_panel_width: 0.0,
		};
		oracle.reload();
		oracle
	}
	fn reload(&mut self) {
		match ChartsFile::load(&self.data_path) {
			Ok(data) => {
				self.data = data;
				self.dirty = false;
				self.error = None;
				self.last_loaded = fs::metadata(&self.data_path)
					.and_then(|metadata| metadata.modified())
					.ok();
				tracing::info!(
					"Oracle loaded {} charts from {}",
					self.data.charts.len(),
					self.data_path.display()
				);
			}
			Err(error) => {
				self.error = Some(error.to_string());
				self.dirty = true;
				tracing::error!(
					"Oracle failed to load {}: {error:#}",
					self.data_path.display()
				);
			}
		}
	}
	/// Call this inside your window event loop / frame tick to check if the file changed.
	fn check_for_changes(&mut self, ctx: &egui::Context) {
		if self.rx.try_recv().is_ok() {
			tracing::info!("File change detected via watcher, reloading Oracle...");
			self.reload();
			ctx.request_repaint(); // Forces egui to redraw immediately
		}
	}
	fn init_watcher(
		path: &Path,
		tx: tokio::sync::mpsc::Sender<()>,
	) -> Result<RecommendedWatcher, notify::Error> {
		let mut watcher = RecommendedWatcher::new(
			move |res: Result<Event, notify::Error>| {
				if let Ok(event) = res {
					if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
						// Use blocking_send since the notify callback is synchronous
						let _ = tx.blocking_send(());
					}
				}
			},
			Config::default(),
		)?;
		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		} else {
			watcher.watch(path, RecursiveMode::NonRecursive)?;
		}
		Ok(watcher)
	}
	fn setup_watcher(
		oracle: Arc<Mutex<Oracle>>,
		path: &Path,
	) -> Result<RecommendedWatcher, notify::Error> {
		// Watcher closure/event handler
		let mut watcher = RecommendedWatcher::new(
			move |res: Result<Event, notify::Error>| {
				match res {
					Ok(event) => {
						// Check if the event is a modification or creation event
						if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
							tracing::info!("Detected change on chart file, reloading...");
							if let Ok(mut locked) = oracle.lock() {
								// locked.reload();
							}
						}
					}
					Err(e) => tracing::error!("Watch error: {e:?}"),
				}
			},
			Config::default(),
		)?;
		// Note: It's often safest to watch the *parent directory* because many editors
		// write files via atomic replacement (saving to a temp file and renaming over the original),
		// which can break direct file watches on some operating systems.
		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		} else {
			watcher.watch(path, RecursiveMode::NonRecursive)?;
		}
		Ok(watcher)
	}
	fn draw_error(&self, ui: &mut egui::Ui, error: &str) {
		ui.heading("Preview Error");
		ui.colored_label(egui::Color32::RED, error);
		ui.separator();
		ui.label("Preview is showing the last valid state.");
	}
	fn draw_ui(&mut self, ui: &mut egui::Ui) {
		if let Some(error) = &self.error {
			self.draw_error(ui, error);
			return;
		}
		let charts = &self.data.charts;
		// Split the available window into a 2x2 grid.
		let available = ui.available_size();
		let cell_width = available.x / 2.0;
		let cell_height = available.y / 2.0;
		for row in 0..2 {
			ui.horizontal(|ui| {
				ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
				for column in 0..2 {
					let index = row * 2 + column;
					ui.allocate_ui(egui::vec2(cell_width - 8.0, cell_height - 8.0), |ui| {
						ui.group(|ui| {
							ui.set_min_size(ui.available_size());
							if let Some(chart) = charts.get(index) {
								chart.ui(ui);
							} else {
								ui.centered_and_justified(|ui| {
									ui.label("No chart");
								});
							}
						});
					});
				}
			});
		}
	}
}
impl Veable for Graphics {
	fn draw(&mut self, ui: &mut egui::Ui) {
		// 1. Poll the channel for file changes on every frame render tick
		self.check_for_changes(ui.ctx());
		// 2. Split the available space to reserve room for the bottom status bar
		let available_size = ui.available_size();
		let status_bar_height = DEFAULT_CONFIG.status_bar.size;
		let main_size = egui::vec2(available_size.x, available_size.y - status_bar_height);
		// Main Content Area
		ui.allocate_ui(main_size, |ui| {
			self.draw_ui(ui);
		});
		ui.separator();
		// Bottom Status Bar
		ui.horizontal(|ui| {
			// Left side: Status or error indicator
			if let Some(error) = &self.error {
				ui.colored_label(egui::Color32::RED, "Status: Error");
			} else if self.dirty {
				ui.colored_label(egui::Color32::YELLOW, "Status: Unsaved / Out of sync");
			} else {
				ui.colored_label(egui::Color32::GREEN, "Status: Connected");
			}
			ui.separator();
			// Right side: Timer / Last Loaded counter
			if let Some(last_loaded) = self.last_loaded {
				if let Ok(elapsed) = last_loaded.elapsed() {
					let secs = elapsed.as_secs();
					let time_str = if secs < 60 {
						format!("Loaded {secs}s ago")
					} else {
						format!("Loaded {}m {}s ago", secs / 60, secs % 60)
					};
					ui.label(time_str);
				}
			} else {
				ui.label("Not loaded yet");
			}
			// Request a continuous repaint so the timer increments live every second
			ui.ctx()
				.request_repaint_after(std::time::Duration::from_secs(1));
		});
	}
}
/// Telemetry
pub struct Oracle {
	pub active_focus: FocusedPane,
	pub secondary_scroll_offset: f32,
	pub side_panel_width: f32,
	dirty: bool,
	error: Option<String>,
	last_direction: String,
	last_loaded: Option<SystemTime>,
	scroll_x: f32,
	scroll_y: f32,
}
impl Oracle {
	pub fn new() -> Self {
		let path = "/Users/future/kb/project/crates/estate/src/data/chart.json";
		Self::from_path(path)
	}
	fn from_path(path: impl Into<PathBuf>) -> Self {
		let data_path = path.into();
		let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
		let mut oracle = Self {
			active_focus: FocusedPane::MainEditor,
			dirty: false,
			error: None,
			last_direction: "".to_string(),
			last_loaded: None,
			scroll_x: 0.0,
			scroll_y: 0.0,
			secondary_scroll_offset: 0.0,
			side_panel_width: 0.0,
		};
		// oracle.reload();
		oracle
	}
	/// Polls current frame inputs and extracts structured trackpad data.
	fn inspect_trackpad(&self, ui: &egui::Ui) -> TrackpadState {
		ui.input(|i| {
			let delta = i.smooth_scroll_delta;
			let shift_held = i.modifiers.shift;
			let mouse_pos = i.pointer.hover_pos();
			let direction = if delta.x == 0.0 && delta.y == 0.0 {
				ScrollDirection::None
			} else if delta.x.abs() > delta.y.abs() {
				if delta.x > 0.0 {
					ScrollDirection::Right
				} else {
					ScrollDirection::Left
				}
			} else {
				if delta.y > 0.0 {
					ScrollDirection::Down
				} else {
					ScrollDirection::Up
				}
			};
			TrackpadState {
				delta,
				direction,
				shift_held,
				mouse_pos,
			}
		})
	}
	/// Helper to check if the mouse is hovering inside a specific target rect
	fn is_mouse_over(state: &TrackpadState, target_rect: egui::Rect) -> bool {
		if let Some(pos) = state.mouse_pos {
			target_rect.contains(pos)
		} else {
			false
		}
	}
	/// Optional helper to draw a quick diagnostic heads-up display overlay
	fn draw_trackpad_poc_hud(&self, ui: &mut egui::Ui, state: &TrackpadState) {
		ui.group(|ui| {
			ui.heading("Trackpad PoC Diagnostics");
			ui.horizontal(|ui| {
				ui.label(format!("Direction: {:?}", state.direction));
				ui.separator();
				ui.label(format!("Shift Held: {}", state.shift_held));
			});
			ui.label(format!(
				"Delta X: {:.2} | Delta Y: {:.2}",
				state.delta.x, state.delta.y
			));
			if let Some(pos) = state.mouse_pos {
				ui.label(format!("Mouse Position: x={:.1}, y={:.1}", pos.x, pos.y));
			}
		});
	}
	fn determine_focus(
		&self,
		mouse_pos: Option<egui::Pos2>,
		main_rect: egui::Rect,
		side_rect: egui::Rect,
	) -> FocusedPane {
		if let Some(pos) = mouse_pos {
			if main_rect.contains(pos) {
				FocusedPane::MainEditor
			} else if side_rect.contains(pos) {
				FocusedPane::SidePanel
			} else {
				FocusedPane::CenterGrid
			}
		} else {
			FocusedPane::Unknown
		}
	}
	/// Handles layout resizing or cross-scrolling based on gestures + shift
	fn handle_shift_gestures(&mut self, trackpad: &TrackpadState, focus: FocusedPane) {
		if !trackpad.shift_held {
			return;
		}
		match focus {
			FocusedPane::MainEditor => {
				// Goal: Move left/right to expand/shrink side panel
				if trackpad.delta.x.abs() > 0.0 {
					// Scale width changes smoothly based on horizontal trackpad delta
					self.side_panel_width = (self.side_panel_width - trackpad.delta.x).clamp(150.0, 600.0);
				}
				// Goal: Scroll the *other* column/panel vertically
				if trackpad.delta.y.abs() > 0.0 {
					self.secondary_scroll_offset += trackpad.delta.y;
					// Clamp or handle your cross-scroll target here
				}
			}
			FocusedPane::SidePanel => {
				// Reverse behavior when your mouse is in the side panel
				if trackpad.delta.x.abs() > 0.0 {
					self.side_panel_width = (self.side_panel_width + trackpad.delta.x).clamp(150.0, 600.0);
				}
			}
			_ => {}
		}
	}
	fn draw_ui(&mut self, ui: &mut egui::Ui) {
		// 1. Capture clean trackpad and modifier state
		let trackpad = self.inspect_trackpad(ui);
		// Keep repainting live while interacting to maintain smooth telemetry display
		if trackpad.direction != ScrollDirection::None
			|| trackpad.shift_held
			|| trackpad.delta != egui::Vec2::ZERO
		{
			ui.ctx().request_repaint();
		}
		// 2. Telemetry Header & Status
		ui.heading("Trackpad & Gesture Telemetry PoC");
		ui.label(
			"Goal: Inspect raw multi-axis vectors, modifiers, and spatial state for cross-app redirection.",
		);
		ui.separator();
		// 3. Live State Grid
		ui.columns(2, |columns| {
			// --- Column A: Raw Input Vectors ---
			columns[0].group(|ui| {
				ui.heading("Raw Input Vectors");
				ui.add_space(4.0);
				ui.label(format!("Scroll Delta X (Horiz): {:.2}", trackpad.delta.x));
				ui.label(format!("Scroll Delta Y (Vert):  {:.2}", trackpad.delta.y));
				ui.add_space(8.0);
				let primary_axis = if trackpad.delta.x.abs() > trackpad.delta.y.abs() {
					"Horizontal (X)"
				} else if trackpad.delta.y.abs() > trackpad.delta.x.abs() {
					"Vertical (Y)"
				} else {
					"None"
				};
				ui.label(format!("Primary Axis: {}", primary_axis));
				ui.label(format!("Direction State: {:?}", trackpad.direction));
			});
			// --- Column B: Modifiers & Spatial Focus ---
			columns[1].group(|ui| {
				ui.heading("Modifiers & Environment");
				ui.add_space(4.0);
				// Visual badge for Shift state
				ui.horizontal(|ui| {
					ui.label("Shift Key:");
					if trackpad.shift_held {
						ui.colored_label(egui::Color32::GREEN, "HELD (Active Modifier)");
					} else {
						ui.colored_label(egui::Color32::GRAY, "Released");
					}
				});
				// Mouse Position telemetry
				if let Some(pos) = trackpad.mouse_pos {
					ui.label(format!("Pointer Coords: x={:.1}, y={:.1}", pos.x, pos.y));
				} else {
					ui.label("Pointer Coords: Out of bounds");
				}
				ui.add_space(8.0);
				ui.label(format!(
					"Target Layout Width: {:.1}px",
					self.side_panel_width
				));
			});
		});
		ui.add_space(12.0);
		ui.separator();
		// 4. Gesture Trigger Simulation Log / Target Action Preview
		ui.group(|ui| {
			ui.heading("Target Action Trigger Preview");
			ui.add_space(4.0);
			let shift_active = trackpad.shift_held;
			let is_horizontal = trackpad.delta.x.abs() > trackpad.delta.y.abs();
			let is_vertical = trackpad.delta.y.abs() > trackpad.delta.x.abs();
			if shift_active && is_horizontal {
				ui.colored_label(
					egui::Color32::LIGHT_BLUE,
					format!(
						"⚡ TRIGGER MATCH: Resize Panel Vector -> {:.2}px",
						trackpad.delta.x
					),
				);
			} else if shift_active && is_vertical {
				ui.colored_label(
					egui::Color32::LIGHT_GREEN,
					format!(
						"⚡ TRIGGER MATCH: Cross-Scroll Secondary Pane -> {:.2} units",
						trackpad.delta.y
					),
				);
			} else {
				ui.label("Waiting for trigger combo (Hold Shift + Swipe/Scroll)...");
			}
		});
		// 5. Minimal footer instructions
		ui.add_space(8.0);
		ui.horizontal(|ui| {
			if ui.button("Reset Telemetry States").clicked() {
				self.secondary_scroll_offset = 0.0;
				self.side_panel_width = 300.0;
			}
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				ui.label("PoC V1.0 - Ready for OS Daemon translation");
			});
		});
	}
}
impl Veable for Oracle {
	fn draw(&mut self, ui: &mut egui::Ui) {
		// 1. Poll the channel for file changes on every frame render tick
		// self.check_for_changes(ui.ctx());
		// 2. Split the available space to reserve room for the bottom status bar
		let available_size = ui.available_size();
		let status_bar_height = 24.0;
		let main_size = egui::vec2(available_size.x, available_size.y - status_bar_height);
		// Main Content Area
		ui.allocate_ui(main_size, |ui| {
			self.draw_ui(ui);
		});
		ui.separator();
		// Bottom Status Bar
		// Left side: Status or error indicator
		ui.horizontal(|ui| {
			if let Some(error) = &self.error {
				ui.colored_label(egui::Color32::RED, "Status: Error");
			} else if self.dirty {
				ui.colored_label(egui::Color32::YELLOW, "Status: Unsaved / Out of sync");
			} else {
				ui.colored_label(egui::Color32::GREEN, "Status: Connected");
			}
			ui.separator();
			// Right side: Timer / Last Loaded counter
			if let Some(last_loaded) = self.last_loaded {
				if let Ok(elapsed) = last_loaded.elapsed() {
					let secs = elapsed.as_secs();
					let time_str = if secs < 60 {
						format!("Loaded {secs}s ago")
					} else {
						format!("Loaded {}m {}s ago", secs / 60, secs % 60)
					};
					ui.label(time_str);
				}
			} else {
				ui.label("Not loaded yet");
			}
			// Request a continuous repaint so the timer increments live every second
			ui.ctx()
				.request_repaint_after(std::time::Duration::from_secs(1));
			if ui.button("Teleport Cursor to Center").clicked() {
				move_cursor_to(ScreenPosition::Center);
			}
		});
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
	None,
	Up,
	Down,
	Left,
	Right,
}
#[derive(Debug, Clone)]
pub struct TrackpadState {
	pub delta: egui::Vec2,
	pub direction: ScrollDirection,
	pub shift_held: bool,
	pub mouse_pos: Option<egui::Pos2>,
}
#[derive(Debug, Copy, Clone)]
pub enum ScreenPosition {
	Left,
	Center,
	Right,
}
pub fn move_cursor_to(pos: ScreenPosition) {
	let max_width = 1920.0; // Adjust to your primary display width
	let center_y = 500.0;
	let (x, y) = match pos {
		ScreenPosition::Left => (max_width * 0.2, center_y),
		ScreenPosition::Center => (max_width * 0.5, center_y),
		ScreenPosition::Right => (max_width * 0.8, center_y),
	};
	let point = CGPoint { x, y };
	if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
		if let Ok(event) =
			CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
		{
			event.post(CGEventTapLocation::HID);
			println!("✨ Teleported cursor to position: X={:.1}, Y={:.1}", x, y);
		}
	}
}
impl TaskManager {
	pub fn new() -> Self {
		Self::from_path("/Users/future/Library/Application Support/estate/state.json")
	}
	pub fn from_path(path: impl Into<PathBuf>) -> Self {
		let state_path = path.into();
		let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
		let watcher = Self::init_watcher(&state_path, tx).expect("Failed to initialize state watcher");
		let mut view = Self {
			tasks: HashMap::new(),
			state_path,
			state: None,
			dirty: false,
			last_loaded: None,
			error: None,
			rx,
			_watcher: watcher,
		};
		view.reload();
		view
	}
	pub fn reload(&mut self) {
		match EstateState::loadFromPath(&self.state_path) {
			Ok(state) => {
				self.state = Some(state);
				self.dirty = false;
				self.error = None;
				self.last_loaded = fs::metadata(&self.state_path)
					.and_then(|metadata| metadata.modified())
					.ok();
			}
			Err(error) => {
				self.error = Some(error.to_string());
				self.dirty = true;
			}
		}
	}
	pub fn check_for_changes(&mut self, ctx: &egui::Context) {
		if self.rx.try_recv().is_ok() {
			self.reload();
			ctx.request_repaint();
		}
	}
	fn init_watcher(
		path: &Path,
		tx: tokio::sync::mpsc::Sender<()>,
	) -> Result<notify::RecommendedWatcher, notify::Error> {
		use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
		let mut watcher = RecommendedWatcher::new(
			move |res: Result<Event, notify::Error>| {
				if let Ok(event) = res {
					if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
						let _ = tx.blocking_send(());
					}
				}
			},
			Config::default(),
		)?;
		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		}
		Ok(watcher)
	}
	fn draw_job(&self, ui: &mut egui::Ui, job: &Job) {
		egui::Frame::group(ui.style()).show(ui, |ui| {
			ui.horizontal(|ui| {
				ui.label(job.status.icon());
				ui.vertical(|ui| {
					ui.strong(&job.name);
					ui.small(format!("Job #{}", job.id));
				});
				ui.add_space(20.0);
				// Status
				ui.label(job.status.label());
				ui.add_space(20.0);
				// Progress
				if let Some(progress) = job.progress {
					ui.add(
						egui::ProgressBar::new(progress)
							.desired_width(180.0)
							.show_percentage(),
					);
				}
				// Runtime
				if let Some(started_at) = job.started_at {
					let elapsed = started_at.elapsed();
					ui.label(format_duration(elapsed));
				}
				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.button("⋮");
				});
			});
		});
	}
}
impl Veable for TaskManager {
	fn draw(&mut self, ui: &mut Ui) {
		self.check_for_changes(ui.ctx());
		if let Some(error) = &self.error {
			ui.heading("Task Manager");
			ui.colored_label(palette::DANGER, error);
			ui.label(self.state_path.display().to_string());
			return;
		}
		let Some(state) = &self.state else {
			ui.centered_and_justified(|ui| {
				ui.label("Loading task state...");
			});
			return;
		};
		// =========================================================
		// Header
		// =========================================================
		ui.vertical(|ui| {
			ui.label(
				egui::RichText::new("Task Overview")
					.size(24.0)
					.strong()
					.color(palette::TEXT),
			);
			ui.add_space(2.0);
			ui.label(
				egui::RichText::new("Estate Runtime")
					.size(12.0)
					.color(palette::TEXT_MUTED),
			);
		});
		ui.add_space(16.0);
		// =========================================================
		// Summary metrics
		// =========================================================
		ui.columns(4, |columns| {
			metric(
				&mut columns[0],
				"Tasks Created",
				state.tasks_created,
				palette::PRIMARY,
			);
			metric(
				&mut columns[1],
				"Tasks Completed",
				state.tasks_completed,
				palette::SUCCESS,
			);
			metric(
				&mut columns[2],
				"Events Processed",
				state.events_processed,
				palette::WARNING,
			);
			metric(
				&mut columns[3],
				"Status Checks",
				state.status_checks,
				palette::TEXT_MUTED,
			);
		});
		ui.add_space(16.0);
		// =========================================================
		// Charts
		// =========================================================
		let available = ui.available_size();
		let gap = 6.0;
		let card_width = (available.x - gap) / 2.0;
		let card_height = 280.0;
		render_graphs(ui, state, available, gap, card_width, card_height);
	}
}
fn render_graphs(
	ui: &mut Ui,
	state: &EstateState,
	available: egui::Vec2,
	gap: f32,
	card_width: f32,
	card_height: f32,
) {
	ui.allocate_ui_with_layout(
		egui::vec2(available.x, card_height),
		egui::Layout::left_to_right(egui::Align::TOP),
		|ui| {
			ui.spacing_mut().item_spacing.x = gap;
			draw_chart_card(
				ui,
				egui::vec2(card_width, card_height),
				"Tasks",
				"Created vs completed",
				// Metrics
				|ui| {
					let remaining = state.tasks_created.saturating_sub(state.tasks_completed);
					small_metric(ui, "Created", state.tasks_created, palette::PRIMARY);
					ui.add_space(20.0);
					small_metric(ui, "Completed", state.tasks_completed, palette::SUCCESS);
					ui.add_space(20.0);
					small_metric(ui, "Remaining", remaining, palette::TEXT_MUTED);
				},
				// Chart
				|ui| {
					let max_value = state.tasks_created.max(1) as f64;
					let bars = vec![
						Bar::new(0.0, state.tasks_created as f64).fill(palette::PRIMARY),
						Bar::new(1.0, state.tasks_completed as f64).fill(palette::SUCCESS),
					];
					let chart = BarChart::new("task_counts", bars);
					let max_y = state.tasks_created.max(1) as f64;
					Plot::new("task_counts_plot")
						.height(190.0)
						.show_axes([true, true])
						.show_grid([true, true])
						.allow_zoom(true)
						.allow_drag(true)
						.allow_scroll(true)
						.allow_axis_zoom_drag(true)
						.allow_boxed_zoom(true)
						.show(ui, |plot_ui| {
							plot_ui.bar_chart(chart);
						});
				},
			);
			// =========================================================
			// Completion
			// =========================================================
			draw_chart_card(
				ui,
				egui::vec2(card_width, card_height),
				"Completion",
				"Task completion ratio",
				// Metrics
				|ui| {
					let created = state.tasks_created as f64;
					let completed = state.tasks_completed as f64;
					let remaining = (created - completed).max(0.0);
					let percentage = if created > 0.0 {
						(completed / created) * 100.0
					} else {
						0.0
					};
					small_metric(ui, "Complete", state.tasks_completed, palette::SUCCESS);
					ui.add_space(20.0);
					small_metric(ui, "Remaining", remaining as u64, palette::TEXT_MUTED);
					ui.add_space(20.0);
					ui.label(
						egui::RichText::new(format!("{percentage:.1}%"))
							.size(14.0)
							.strong()
							.color(palette::SUCCESS),
					);
				},
				// Chart
				|ui| {
					let created = state.tasks_created as f64;
					let completed = state.tasks_completed as f64;
					let remaining = (created - completed).max(0.0);
					let percentage = if created > 0.0 {
						(completed / created) * 100.0
					} else {
						0.0
					};
					let bars = vec![
						Bar::new(0.0, completed).fill(palette::SUCCESS),
						Bar::new(1.0, remaining).fill(palette::SURFACE_HOVER),
					];
					let chart = BarChart::new("task_completion", bars);
					let max_value = completed.max(remaining);
					Plot::new("task_completion_plot")
						.height(190.0)
						.show_axes([true, true])
						.show_grid([true, false])
						.clamp_grid(true)
						// Initial/reset viewport:
						.auto_bounds([false, false])
						.default_x_bounds(-0.5, 1.5)
						.default_y_bounds(0.0, (max_value * 1.1).max(1.0))
						// Interactive:
						.allow_zoom(true)
						.allow_drag(true)
						.allow_scroll(true)
						.allow_axis_zoom_drag(true)
						.allow_boxed_zoom(false)
						.show(ui, |plot_ui| {
							plot_ui.bar_chart(chart);
						});
				},
			);
		},
	);
	ui.add_space(gap);
	ui.allocate_ui_with_layout(
		egui::vec2(available.x, card_height),
		egui::Layout::left_to_right(egui::Align::TOP),
		|ui| {
			ui.spacing_mut().item_spacing.x = gap;
			// =========================================================
			// System Activity
			// =========================================================
			draw_chart_card(
				ui,
				egui::vec2(card_width, card_height),
				"System Activity",
				"Runtime activity",
				// Metrics
				|ui| {
					small_metric(ui, "Starts", state.starts, palette::PRIMARY);
					ui.add_space(16.0);
					small_metric(ui, "Checks", state.status_checks, palette::TEXT_MUTED);
					ui.add_space(16.0);
					small_metric(ui, "Events", state.events_processed, palette::WARNING);
					ui.add_space(16.0);
					small_metric(ui, "Files", state.files_indexed, palette::SUCCESS);
				},
				// Chart
				|ui| {
					let max_value = [
						state.starts,
						state.status_checks,
						state.events_processed,
						state.files_indexed,
					]
					.into_iter()
					.max()
					.unwrap_or(1) as f64;
					let bars = vec![
						Bar::new(0.0, state.starts as f64).fill(palette::PRIMARY),
						Bar::new(1.0, state.status_checks as f64).fill(palette::TEXT_MUTED),
						Bar::new(2.0, state.events_processed as f64).fill(palette::WARNING),
						Bar::new(3.0, state.files_indexed as f64).fill(palette::SUCCESS),
					];
					let chart = BarChart::new("system_activity", bars);
					let max_value = [
						state.starts,
						state.status_checks,
						state.events_processed,
						state.files_indexed,
					]
					.into_iter()
					.max()
					.unwrap_or(1) as f64;
					// .default_y_bounds(0.0, max_value * 1.1)
					Plot::new("system_activity_plot")
						.height(190.0)
						.show_axes([true, true])
						.show_grid([true, false])
						.allow_zoom(true)
						.allow_drag(true)
						.allow_scroll(true)
						.allow_axis_zoom_drag(true)
						.allow_boxed_zoom(false)
						.default_x_bounds(-0.5, 3.5)
						.default_y_bounds(0.0, (max_value * 1.1).max(1.0))
						.show(ui, |plot_ui| {
							plot_ui.bar_chart(chart);
						});
				},
			);
			// =========================================================
			// Runtime
			// =========================================================
			draw_chart_card(
				ui,
				egui::vec2(card_width, card_height),
				"Runtime",
				"Task manager activity",
				// Metrics
				|ui| {
					small_metric(ui, "Starts", state.starts, palette::PRIMARY);
					ui.add_space(20.0);
					small_metric(ui, "Longest Run", state.longest_run, palette::WARNING);
					ui.add_space(20.0);
					small_metric(ui, "Events", state.events_processed, palette::TEXT_MUTED);
				},
				// Chart
				|ui| {
					let max_value = [state.starts, state.events_processed, state.files_indexed]
						.into_iter()
						.max()
						.unwrap_or(1) as f64;
					let bars = vec![
						Bar::new(0.0, state.starts as f64).fill(palette::PRIMARY),
						Bar::new(1.0, state.events_processed as f64).fill(palette::WARNING),
						Bar::new(2.0, state.files_indexed as f64).fill(palette::SUCCESS),
					];
					let chart = BarChart::new("runtime_activity", bars);
					Plot::new("runtime_activity_plot")
						.height(190.0)
						.show_axes([true, true])
						.show_grid([true, false])
						.allow_zoom(true)
						.allow_drag(true)
						.allow_scroll(true)
						.allow_axis_zoom_drag(true)
						.allow_boxed_zoom(false)
						.default_x_bounds(-0.5, 2.5)
						.default_y_bounds(0.0, (max_value * 1.1).max(1.0))
						.show(ui, |plot_ui| {
							plot_ui.bar_chart(chart);
						});
				},
			);
		},
	);
}
fn metric(ui: &mut Ui, label: &str, value: u64, color: egui::Color32) {
	ui.group(|ui| {
		ui.set_min_height(78.0);
		ui.vertical_centered(|ui| {
			ui.label(
				egui::RichText::new(label)
					.small()
					.color(palette::TEXT_MUTED),
			);
			ui.label(
				egui::RichText::new(value.to_string())
					.size(26.0)
					.strong()
					.color(color),
			);
		});
	});
}
fn draw_chart_card(
	ui: &mut Ui,
	size: egui::Vec2,
	title: &str,
	subtitle: &str,
	metrics: impl FnOnce(&mut Ui),
	chart: impl FnOnce(&mut Ui),
) {
	ui.allocate_ui(size, |ui| {
		egui::Frame::group(ui.style())
			.fill(palette::SURFACE)
			.stroke(egui::Stroke::new(1.0, palette::BORDER))
			// .inner_margin(egui::Margin::same(12))
			.show(ui, |ui| {
				// Force the card's contents into a vertical stack.
				ui.vertical(|ui| {
					// -------------------------------------------------
					// Header
					// -------------------------------------------------
					ui.label(
						egui::RichText::new(title)
							.size(15.0)
							.strong()
							.color(palette::TEXT),
					);
					ui.label(
						egui::RichText::new(subtitle)
							.size(11.0)
							.color(palette::TEXT_MUTED),
					);
					ui.add_space(8.0);
					// -------------------------------------------------
					// Metrics
					// -------------------------------------------------
					ui.horizontal(|ui| {
						metrics(ui);
					});
					ui.add_space(8.0);
					// -------------------------------------------------
					// Chart
					// -------------------------------------------------
					ui.vertical(|ui| {
						chart(ui);
					});
				});
			});
	});
}
fn small_metric(ui: &mut Ui, label: &str, value: u64, color: egui::Color32) {
	ui.horizontal(|ui| {
		ui.label(
			egui::RichText::new(label)
				.size(11.0)
				.color(palette::TEXT_MUTED),
		);
		ui.label(
			egui::RichText::new(value.to_string())
				.size(13.0)
				.strong()
				.color(color),
		);
	});
}
fn format_duration(duration: Duration) -> String {
	let secs = duration.as_secs();
	if secs < 60 {
		format!("{secs}s")
	} else if secs < 3600 {
		format!("{}m {}s", secs / 60, secs % 60)
	} else {
		format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
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
					let shift_is_down = flags.contains(CGEventFlags::CGEventFlagShift);
					let was_down = SHIFT_HELD.swap(shift_is_down, Ordering::Relaxed);
					// =========================================================
					// SHIFT DOWN
					// =========================================================
					if shift_is_down && !was_down {
						let location = event.location();
						let mut state = scroll_state().lock().unwrap();
						state.active = true;
						state.redirected = true;
						state.original_position = location;
						let target = if location.x < 960.0 {
							ScreenPosition::Right
						} else {
							ScreenPosition::Left
						};
						let bounds = CGDisplay::main().bounds();
						let target_x = match target {
							ScreenPosition::Left => bounds.origin.x + bounds.size.width * 0.25,
							ScreenPosition::Right => bounds.origin.x + bounds.size.width * 0.75,
							ScreenPosition::Center => bounds.origin.x + bounds.size.width * 0.50,
						};
						let target_position = CGPoint {
							x: target_x,
							y: location.y,
						};
						state.target_position = target_position;
						// println!(
						// 	"⬇️ SHIFT DOWN | ({:.0}, {:.0}) -> {:?} ({:.0}, {:.0})",
						// 	location.x, location.y, target, target_position.x, target_position.y,
						// );
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
					// =========================================================
					// SHIFT UP
					// =========================================================
					if !shift_is_down && was_down {
						let mut state = scroll_state().lock().unwrap();
						let original = state.original_position;
						// println!(
						// 	"⬆️ SHIFT UP | restoring ({:.0}, {:.0})",
						// 	original.x, original.y
						// );
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
						18 => {
							println!("Key '1' pressed");
							move_cursor_to(ScreenPosition::Left);
						}
						19 => {
							println!("Key '2' pressed");
							move_cursor_to(ScreenPosition::Center);
						}
						20 => {
							println!("Key '3' pressed");
							move_cursor_to(ScreenPosition::Right);
						}
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
struct DebugPanel {
	title: String,
}
impl DebugPanel {
	fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
		}
	}
}
impl Veable for DebugPanel {
	fn draw(&mut self, ui: &mut egui::Ui) {
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
// pub struct Size {
// 	pub value: f32,
// 	pub min: f32,
// 	pub max: f32,
// 	pub resizable: bool,
// }
// A layout area within the view.
// Regions define where content lives, how much space it occupies,
// and how that space is visually presented.
// A region may contain arbitrary UI content, potentially composed
// from smaller slots.
pub struct Region {
	pub content: Box<dyn Veable>,
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
impl Region {
	pub fn new(view: impl Veable + 'static, size: f32) -> Self {
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
	pub fn fixed(view: impl Veable + 'static, size: f32) -> Self {
		Self::new(view, size)
	}
	pub fn resizable(view: impl Veable + 'static, size: f32, min_size: f32, max_size: f32) -> Self {
		let mut region = Self::new(view, size);
		region.size = size.clamp(min_size, max_size);
		region.min_size = min_size;
		region.max_size = max_size;
		region.resizable = true;
		region.fill = Some(palette::SURFACE);
		region.is_docked = true;
		region
	}
	pub fn content(view: impl Veable + 'static) -> Self {
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
				rect.left() + self.padding.left as f32,
				rect.top() + self.padding.top as f32,
			),
			egui::pos2(
				rect.right() - self.padding.right as f32,
				rect.bottom() - self.padding.bottom as f32,
			),
		)
	}
}
/// A named, interactive view that occupies a region.
///
/// Panels add interaction and lifecycle behavior to a Region.
/// They may be opened, closed, overlaid, auto-hidden, moved,
/// or potentially detached from their parent layout.
pub struct Panel {
	pub region: Region,
	pub open: bool,
	pub overlay: bool,
	pub auto_hide: bool,
}
impl Panel {
	pub fn new(region: Region) -> Self {
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
pub struct PanelConfig {
	// It's region is visible & this panel is the focused one
	pub active: bool,
	pub size: f32,
	pub resizable: bool,
	pub docked: bool,
}
impl PanelConfig {
	pub const fn new(active: bool, size: f32) -> Self {
		Self {
			active,
			size,
			resizable: true,
			docked: true,
		}
	}
	fn with_active() -> Self {
		Self {
			active: true,
			docked: true,
			resizable: true,
			size: 280.0,
		}
	}
}
#[derive(Clone, Copy)]
enum ResizeEdge {
	Left,
	Right,
	Top,
	Bottom,
}
