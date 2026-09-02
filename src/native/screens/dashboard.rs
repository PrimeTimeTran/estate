use crate::{
	app::*,
	e,
	native::{OracleScreen, monitor::StateMonitor, runtime::NativeRuntime},
	ui::{Component, Layout, chart::*},
};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Debug, Default)]
pub struct DashboardScreen {
	#[cfg(not(target_arch = "wasm32"))]
	data_path: PathBuf,

	data: ChartsFile,
	dirty: bool,
	last_loaded: Option<SystemTime>,
	error: Option<String>,

	#[cfg(not(target_arch = "wasm32"))]
	monitor: StateMonitor,

	scroll_x: f32,
	scroll_y: f32,
	last_direction: String,

	pub side_panel_width: f32,
	pub active_focus: FocusedPane,
	pub secondary_scroll_offset: f32,
}
impl<R: Runtime> Screen<R> for DashboardScreen {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Configure the regions this screen uses.
	}

	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
}
impl DashboardScreen {
	pub fn new() -> Self {
		let data_path = PathBuf::from(HMR_CHART_JSON);
		let mut dashboard = Self {
			data_path,
			..Self::default()
		};
		dashboard.monitor =
			StateMonitor::new(&dashboard.data_path).expect("failed to watch dashboard data");
		dashboard.load();
		dashboard
	}

	fn load(&mut self) {
		match fs::read_to_string(&self.data_path) {
			Ok(contents) => match serde_json::from_str::<ChartsFile>(&contents) {
				Ok(data) => {
					self.data = data;
					self.last_loaded = fs::metadata(&self.data_path)
						.and_then(|meta| meta.modified())
						.ok();
					self.error = None;
					self.dirty = false;

					tracing::debug!(
						path = %self.data_path.display(),
						"dashboard data loaded"
					);
				}
				Err(error) => {
					self.error = Some(error.to_string());

					tracing::error!(
						%error,
						path = %self.data_path.display(),
						"failed to parse dashboard data"
					);
				}
			},
			Err(error) => {
				self.error = Some(error.to_string());
				tracing::error!(
					%error,
					path = %self.data_path.display(),
					"failed to read dashboard data"
				);
			}
		}
	}
}
impl DashboardScreen {
	fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
		use crate::native::monitor::StateMonitor;
		let data_path = path.into();
		let monitor = StateMonitor::new(&data_path)?;
		let mut graphics = Self {
			data_path: data_path.clone(),
			monitor,
			data: ChartsFile { charts: Vec::new() },
			dirty: false,
			error: None,
			last_loaded: None,
			scroll_x: 0.0,
			scroll_y: 0.0,
			last_direction: String::new(),
			side_panel_width: 0.0,
			active_focus: FocusedPane::MainEditor,
			secondary_scroll_offset: 0.0,
		};

		graphics.reload();

		Ok(graphics)
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

				tracing::debug!(
					"Graphics loaded {} charts from {}",
					self.data.charts.len(),
					self.data_path.display()
				);
			}

			Err(error) => {
				self.error = Some(error.to_string());
				self.dirty = true;

				tracing::error!(
					"Graphics failed to load {}: {error:#}",
					self.data_path.display()
				);
			}
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn check_for_changes(&mut self, ctx: &egui::Context) {
		if self.monitor.poll() {
			tracing::debug!(
				"File change detected, reloading {}",
				self.data_path.display()
			);
			self.reload();
			ctx.request_repaint();
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
		oracle: Arc<Mutex<OracleScreen>>,
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
impl DashboardScreen {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		// 1. Poll the channel for file changes on every frame render tick
		#[cfg(not(target_arch = "wasm32"))]
		self.check_for_changes(ui.ctx());
		// 2. Split the available space to reserve room for the bottom status bar
		let available_size = ui.available_size();
		let status_bar_height = LAYOUT.status_bar.size;
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
