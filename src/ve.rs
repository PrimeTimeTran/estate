use crate::prelude::*;

use egui::Ui;
use egui_plot::{Line, PlotPoints, Points};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

use egui::Context;
use egui_plot::{Bar, BarChart, Plot};

/// A trait implemented by types which agree to its contract.
///
/// Any type which implements this contract must provide `draw`.
/// Code which depends on `Veable` can therefore rely on that capability
/// without needing to know how the concrete type implements it.
///
/// The implementation details belong to the concrete type; the caller
/// only depends on the behavior promised by the contract.
pub trait Veable {
	fn draw(&mut self, ui: &mut egui::Ui);
}

/// A type-erased container for any concrete `Veable`.
///
/// `Box<dyn Veable>` stores the concrete implementation on the heap while
/// exposing only the `Veable` interface to callers. This allows different
/// concrete implementations to be substituted without changing the code
/// which consumes them.
pub struct Ve {
	view: Box<dyn Veable>,
}

impl Ve {
	/// Rust uses ownership, borrowing, and lifetimes to determine when values
	/// may be safely destroyed, allowing memory to be reclaimed deterministically
	/// without a garbage collector.
	pub fn new(view: impl Veable + 'static) -> Self {
		Self {
			view: Box::new(view),
		}
	}

	/// Forwards the drawing contract to the concrete implementation.
	///
	/// `Ve` doesn't know how the view is drawn. It only knows that the
	/// contained implementation satisfies `Veable`.
	pub fn draw(&mut self, ui: &mut egui::Ui) {
		self.view.draw(ui);
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
	// rx: tokio::sync::broadcast::Receiver<()>,
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
	pub fn from_path(path: impl Into<PathBuf>) -> Self {
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
	pub fn reload(&mut self) {
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
	pub fn check_for_changes(&mut self, ctx: &egui::Context) {
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
	// fn draw_ui(&mut self, ui: &mut egui::Ui) {
	// 	// 1. Capture clean trackpad and modifier state
	// 	let trackpad = self.inspect_trackpad(ui);

	// 	// Keep repainting live while interacting to maintain smooth telemetry display
	// 	if trackpad.direction != ScrollDirection::None
	// 		|| trackpad.shift_held
	// 		|| trackpad.delta != egui::Vec2::ZERO
	// 	{
	// 		ui.ctx().request_repaint();
	// 	}

	// 	// 2. Telemetry Header & Status
	// 	ui.heading("Trackpad & Gesture Telemetry PoC");
	// 	ui.label(
	// 		"Goal: Inspect raw multi-axis vectors, modifiers, and spatial state for cross-app redirection.",
	// 	);
	// 	ui.separator();

	// 	// 3. Live State Grid
	// 	ui.columns(2, |columns| {
	// 		// --- Column A: Raw Input Vectors ---
	// 		columns[0].group(|ui| {
	// 			ui.heading("Raw Input Vectors");
	// 			ui.add_space(4.0);

	// 			ui.label(format!("Scroll Delta X (Horiz): {:.2}", trackpad.delta.x));
	// 			ui.label(format!("Scroll Delta Y (Vert):  {:.2}", trackpad.delta.y));
	// 			ui.add_space(8.0);

	// 			let primary_axis = if trackpad.delta.x.abs() > trackpad.delta.y.abs() {
	// 				"Horizontal (X)"
	// 			} else if trackpad.delta.y.abs() > trackpad.delta.x.abs() {
	// 				"Vertical (Y)"
	// 			} else {
	// 				"None"
	// 			};
	// 			ui.label(format!("Primary Axis: {}", primary_axis));
	// 			ui.label(format!("Direction State: {:?}", trackpad.direction));
	// 		});

	// 		// --- Column B: Modifiers & Spatial Focus ---
	// 		columns[1].group(|ui| {
	// 			ui.heading("Modifiers & Environment");
	// 			ui.add_space(4.0);

	// 			// Visual badge for Shift state
	// 			ui.horizontal(|ui| {
	// 				ui.label("Shift Key:");
	// 				if trackpad.shift_held {
	// 					ui.colored_label(egui::Color32::GREEN, "HELD (Active Modifier)");
	// 				} else {
	// 					ui.colored_label(egui::Color32::GRAY, "Released");
	// 				}
	// 			});

	// 			// Mouse Position telemetry
	// 			if let Some(pos) = trackpad.mouse_pos {
	// 				ui.label(format!("Pointer Coords: x={:.1}, y={:.1}", pos.x, pos.y));
	// 			} else {
	// 				ui.label("Pointer Coords: Out of bounds");
	// 			}

	// 			ui.add_space(8.0);
	// 			ui.label(format!(
	// 				"Target Layout Width: {:.1}px",
	// 				self.side_panel_width
	// 			));
	// 		});
	// 	});

	// 	ui.add_space(12.0);
	// 	ui.separator();

	// 	// 4. Gesture Trigger Simulation Log / Target Action Preview
	// 	ui.group(|ui| {
	// 		ui.heading("Target Action Trigger Preview");
	// 		ui.add_space(4.0);

	// 		let shift_active = trackpad.shift_held;
	// 		let is_horizontal = trackpad.delta.x.abs() > trackpad.delta.y.abs();
	// 		let is_vertical = trackpad.delta.y.abs() > trackpad.delta.x.abs();

	// 		if shift_active && is_horizontal {
	// 			ui.colored_label(
	// 				egui::Color32::LIGHT_BLUE,
	// 				format!(
	// 					"⚡ TRIGGER MATCH: Resize Panel Vector -> {:.2}px",
	// 					trackpad.delta.x
	// 				),
	// 			);
	// 		} else if shift_active && is_vertical {
	// 			ui.colored_label(
	// 				egui::Color32::LIGHT_GREEN,
	// 				format!(
	// 					"⚡ TRIGGER MATCH: Cross-Scroll Secondary Pane -> {:.2} units",
	// 					trackpad.delta.y
	// 				),
	// 			);
	// 		} else {
	// 			ui.label("Waiting for trigger combo (Hold Shift + Swipe/Scroll)...");
	// 		}
	// 	});

	// 	// 5. Minimal footer instructions
	// 	ui.add_space(8.0);
	// 	ui.horizontal(|ui| {
	// 		if ui.button("Reset Telemetry States").clicked() {
	// 			self.secondary_scroll_offset = 0.0;
	// 			self.side_panel_width = 300.0;
	// 		}
	// 		ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
	// 			ui.label("PoC V1.0 - Ready for OS Daemon translation");
	// 		});
	// 	});
	// }
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
		let status_bar_height = 24.0;

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
	scroll_x: f32,
	scroll_y: f32,
	last_direction: String,
	dirty: bool,
	last_loaded: Option<SystemTime>,
	error: Option<String>,
	pub side_panel_width: f32,
	pub active_focus: FocusedPane,
	pub secondary_scroll_offset: f32,
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
	pub fn inspect_trackpad(&self, ui: &egui::Ui) -> TrackpadState {
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
	pub fn is_mouse_over(state: &TrackpadState, target_rect: egui::Rect) -> bool {
		if let Some(pos) = state.mouse_pos {
			target_rect.contains(pos)
		} else {
			false
		}
	}

	/// Optional helper to draw a quick diagnostic heads-up display overlay
	pub fn draw_trackpad_poc_hud(&self, ui: &mut egui::Ui, state: &TrackpadState) {
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
	pub fn determine_focus(
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
	pub fn handle_shift_gestures(&mut self, trackpad: &TrackpadState, focus: FocusedPane) {
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

use core_foundation::{
	base::{CFAllocatorRef, TCFType, kCFAllocatorDefault},
	mach_port::{CFMachPort, CFMachPortRef},
	runloop::{CFRunLoop, CFRunLoopSource, CFRunLoopSourceRef, kCFRunLoopCommonModes},
};
use core_graphics::{
	event::{
		self, CGEvent, CGEventField, CGEventTap, CGEventTapLocation, CGEventTapOptions,
		CGEventTapPlacement, CGEventTapProxy, CGEventType, CallbackResult, ScrollEventUnit, *,
	},
	event_source::{CGEventSource, CGEventSourceRef, CGEventSourceStateID},
};
use std::sync::atomic::{AtomicBool, Ordering};
static SHIFT_HELD: AtomicBool = AtomicBool::new(false);

pub fn start_global_scroll_daemon(proxy: EventLoopProxy<AppEvent>) {
	let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
	if !trusted {
		eprintln!("❌ App is not trusted for accessibility/input monitoring.");
		return;
	}
	println!("✅ App is trusted! Initializing event tap with keyboard/mouse listeners...");

	let callback =
		move |_proxy_cg: CGEventTapProxy, event_type: CGEventType, event: &CGEvent| -> CallbackResult {
			match event_type {
				CGEventType::MouseMoved => {
					let location = event.location();
					let _ = proxy.send_event(AppEvent::CursorPosition {
						x: location.x,
						y: location.y,
					});
					CallbackResult::Keep
				}
				CGEventType::FlagsChanged => {
					let flags = event.get_flags();
					let shift_is_down = flags.contains(core_graphics::event::CGEventFlags::CGEventFlagShift);
					SHIFT_HELD.store(shift_is_down, Ordering::Relaxed);
					CallbackResult::Keep
				}
				// --- 🟢 LISTEN FOR KEY PRESSES (1, 2, 3) ---
				CGEventType::KeyDown => {
					let keycode =
						event.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE);
					match keycode {
						18 => {
							println!("Key '1' pressed -> Jumping Left!");
							move_cursor_to(ScreenPosition::Left);
						}
						19 => {
							println!("Key '2' pressed -> Jumping Center!");
							move_cursor_to(ScreenPosition::Center);
						}
						20 => {
							println!("Key '3' pressed -> Jumping Right!");
							move_cursor_to(ScreenPosition::Right);
						}
						_ => {}
					}
					CallbackResult::Keep
				}
				// ---------------------------------------------
				CGEventType::ScrollWheel => {
					if SHIFT_HELD.load(Ordering::Relaxed) {
						let location = event.location();
						if location.x < 960.0 {
							if let Some(synth) = redirect_scroll(event) {
								synth.post(CGEventTapLocation::HID);
							}
							return CallbackResult::Drop;
						}
					}
					CallbackResult::Keep
				}
				_ => CallbackResult::Keep,
			}
		};

	// Include KeyDown alongside mouse events in the tap mask/vector
	let tap = CGEventTap::new(
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
	);

	match tap {
		Ok(t) => {
			println!("SUCCESS: Global daemon CGEventTap created successfully!");
			unsafe {
				let port = t.mach_port();
				let source = port
					.create_runloop_source(0)
					.expect("failed to create run loop source");

				CFRunLoop::get_current().add_source(&source, kCFRunLoopCommonModes);
				t.enable();

				std::thread::spawn(move || {
					CFRunLoop::run_current();
				});
			}
		}
		Err(e) => {
			eprintln!(
				"CRITICAL ERROR: CGEventTap creation failed: {:?} (Check Accessibility permissions!)",
				e
			);
		}
	}
}
fn redirect_scroll(original: &CGEvent) -> Option<CGEvent> {
	let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
	let delta_y = original.get_integer_value_field(115);
	let delta_x = original.get_integer_value_field(116);
	CGEvent::new_scroll_event(
		source,
		ScrollEventUnit::PIXEL,
		2,
		delta_y as i32,
		delta_x as i32,
		0,
	)
	.ok()
}
use core_graphics::event::CGMouseButton;
use core_graphics::geometry::CGPoint;

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

use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};
use std::sync::OnceLock;
static HOTKEY_MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

pub fn setup_global_shortcuts() {
	dbg!("dbg setup_global_shortcuts");
	eprintln!("eprintln setup_global_shortcuts");
	println!("println setup_global_shortcuts");
	let manager = GlobalHotKeyManager::new().expect("Failed to initialize GlobalHotKeyManager");

	// Register Shift + Alt + 1
	let hotkey_left = HotKey::new(Some(Modifiers::SHIFT | Modifiers::ALT), Code::Digit1);
	let left_id = hotkey_left.id();

	manager
		.register(hotkey_left)
		.expect("Failed to register hotkey");
	println!("✨ Global hotkey (Shift + Alt + 1) registered and active!");

	let _ = HOTKEY_MANAGER.set(manager);

	std::thread::spawn(move || {
		let receiver = GlobalHotKeyEvent::receiver();
		loop {
			if let Ok(event) = receiver.try_recv() {
				if event.state == global_hotkey::HotKeyState::Pressed && event.id == left_id {
					println!("🔥 Global Hotkey Triggered via OS Event!");

					// 1. Trigger your cursor jump
					move_cursor_to(ScreenPosition::Left);

					// 2. Fire a native macOS notification banner to prove it caught the key combo
					let _ = std::process::Command::new("osascript")
						.arg("-e")
						.arg("display notification \"Shift+Alt+1 intercepted!\" with title \"Estate Daemon\"")
						.spawn();
				}
			}
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
	});
}
