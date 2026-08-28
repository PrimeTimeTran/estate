use crate::{
	prelude::*,
	theme::palette,
	ui::{DEFAULT_CONFIG as CONFIG, *},
};

use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points};

pub trait Veable {
	fn draw(&mut self, ui: &mut egui::Ui);
}
pub struct Ve {
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
		let config = CONFIG;
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
	pub fn draw(&mut self, ui: &mut egui::Ui) {
		let available = ui.available_rect_before_wrap();
		ui.painter().rect_filled(available, 0.0, CONFIG.bg);
		let (
			left_rect,
			right_rect,
			tabs_rect,
			breadcrumbs_rect,
			main_rect,
			bottom_rect,
			status_bar_rect,
		) = self.calculute_layout(available);

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

	fn calculute_layout(
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
		let status_bar_height = CONFIG.status_bar.size;
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
		let tabs_height = CONFIG.primary_bar.size;
		let breadcrumbs_height = CONFIG.secondary_bar.size;
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
		let fill = region.fill.unwrap_or(CONFIG.bg);
		ui.painter().rect_filled(rect, 0.0, fill);
		if region.top_border {
			ui.painter().line_segment(
				[
					egui::pos2(rect.left(), rect.top()),
					egui::pos2(rect.right(), rect.top()),
				],
				egui::Stroke::new(1.0, CONFIG.surface),
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
			let sweep = (fraction as f32) * std::f32::consts::TAU;
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
			let t = (i as f32) / (segments as f32);
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
	data: ChartsFile,
	dirty: bool,
	error: Option<String>,
	scroll_x: f32,
	scroll_y: f32,
	last_direction: String,
	pub side_panel_width: f32,
	pub active_focus: FocusedPane,
	pub secondary_scroll_offset: f32,
}
impl Graphics {
	pub fn new() -> Self {
		let data = Self::load_data();

		Self {
			data,
			dirty: false,
			error: None,
			scroll_x: 0.0,
			scroll_y: 0.0,
			last_direction: String::new(),
			side_panel_width: 0.0,
			active_focus: FocusedPane::MainEditor,
			secondary_scroll_offset: 0.0,
		}
	}
	fn load_data() -> ChartsFile {
		serde_json::from_str(include_str!("../data/chart.json")).expect("failed to parse chart.json")
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
		// self.check_for_changes(ui.ctx());
		// 2. Split the available space to reserve room for the bottom status bar
		let available_size = ui.available_size();
		let status_bar_height = CONFIG.status_bar.size;
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
			// if let Some(last_loaded) = self.last_loaded {
			// 	if let Ok(elapsed) = last_loaded.elapsed() {
			// 		let secs = elapsed.as_secs();
			// 		let time_str = if secs < 60 {
			// 			format!("Loaded {secs}s ago")
			// 		} else {
			// 			format!("Loaded {}m {}s ago", secs / 60, secs % 60)
			// 		};
			// 		ui.label(time_str);
			// 	}
			// } else {
			// 	ui.label("Not loaded yet");
			// }
			// Request a continuous repaint so the timer increments live every second
			ui.ctx()
				.request_repaint_after(std::time::Duration::from_secs(1));
		});
	}
}
impl Graphics {
	// #[cfg(not(target_arch = "wasm32"))]
	// pub fn new() -> Self {
	// 	let path = "...";
	// 	// Self::from_path(path)
	// }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
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
