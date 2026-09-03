use crate::{
	api::Api,
	app::{state::EstateState, *},
	e,
	native::runtime::NativeRuntime,
	prelude::*,
	theme::palette,
	ui::{Component, Layout},
};
use egui::Ui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Bar, BarChart, Plot};
use std::time::Duration;

impl TaskManager {
	pub fn new() -> Self {
		let path = PathBuf::from(STATE_PATH);
		Self::from_path(path).unwrap()
	}
	pub fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
		let state_path = path.into();
		let runtime = TaskManagerRuntime::new(&state_path)?;
		let mut state = TaskManagerState {
			state_path,
			..Default::default()
		};
		let mut manager = Self { state, runtime };
		manager.reload();

		Ok(manager)
	}
	pub fn reload(&mut self) {
		match EstateState::load_from_path(&self.state.state_path) {
			Ok(state) => {
				self.state.state = Some(state);
				self.state.dirty = false;
				self.state.error = None;
				self.state.last_loaded = fs::metadata(&self.state.state_path)
					.and_then(|metadata| metadata.modified())
					.ok();
			}
			Err(error) => {
				self.state.error = Some(error.to_string());
				self.state.dirty = true;
			}
		}
	}
	pub fn poll_changes(&mut self) -> bool {
		#[cfg(not(target_arch = "wasm32"))]
		{
			if self.runtime.rx.try_recv().is_ok() {
				self.reload();
				return true;
			}
		}

		false
	}
}
impl TaskManager {
	pub fn create(&mut self, kind: TaskKind) -> TaskId {
		let id = Uuid::new_v4();
		let task = Task {
			id,
			name: kind.name().into(),
			kind,
			status: TaskStatus::Pending,
		};
		self.state.tasks.insert(id, task);
		self.state.dirty = true;
		id
	}
	pub fn get(&self, id: TaskId) -> Option<&Task> {
		self.state.tasks.get(&id)
	}
	fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
		self.state.tasks.get_mut(&id)
	}
	pub fn count(&self) -> usize {
		self.state.tasks.len()
	}
	pub fn list(&self) -> impl Iterator<Item = &Task> {
		self.state.tasks.values()
	}
	pub fn set_status(&mut self, id: TaskId, status: TaskStatus) -> Result<()> {
		let task = self
			.state
			.tasks
			.get_mut(&id)
			.ok_or_else(|| anyhow::anyhow!("task {id} not found"))?;

		task.status = status;

		Ok(())
	}
	pub fn save(&mut self) -> Result<()> {
		todo!("save")
	}
	pub fn clear(&mut self) -> bool {
		if self.state.tasks.is_empty() {
			return false;
		}

		self.state.tasks.clear();
		self.state.dirty = true;
		true
	}
	pub fn delete(&mut self, id: TaskId) -> Option<Task> {
		let task = self.state.tasks.remove(&id);

		if task.is_some() {
			self.state.dirty = true;
		}

		task
	}
}

#[derive(Debug)]
pub struct TaskManagerScreen {
	pub waterfall: WaterfallChart,
	view: TaskManagerView,
}
impl TaskManagerScreen {
	pub fn new() -> Self {
		Self {
			view: TaskManagerView::default(),
			waterfall: WaterfallChart::default(),
		}
	}
}

impl<R: Runtime> Screen<R> for TaskManagerScreen {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Put TaskManagerView into the appropriate region/panel.
	}

	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// self.manager.poll_changes();
		self.view.update(ctx);
	}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Feature-level event handling.
		//
		// e.g. TaskCreated, TaskDeleted, etc.
	}
}
// impl<R: Runtime> TaskManagerScreen {
// 	fn event(&mut self, event: &e::Event, _ctx: &mut AppContext<'_, R>) {
// 		if let e::EventKind::TaskCreated { .. } = event.kind {}
// 	}
// }

#[derive(Debug, Default)]
pub struct TaskManagerView {
	waterfall: WaterfallChart,
}
impl TaskManagerView {
	// 	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
	// 	ui.heading("Task Manager draw");
	// 	if self.poll_changes() {
	// 		ui.ctx().request_repaint();
	// 	}
	// 	if let Some(error) = &self.state.error {
	// 		ui.heading("Task Manager");
	// 		ui.colored_label(palette::ERROR, error);
	// 		ui.label(self.state.state_path.display().to_string());
	// 		return;
	// 	}
	// 	let Some(state) = &self.state.state else {
	// 		ui.centered_and_justified(|ui| {
	// 			ui.label("Loading task state...");
	// 		});
	// 		return;
	// 	};
	// 	// =========================================================
	// 	// Header
	// 	// =========================================================
	// 	ui.vertical(|ui| {
	// 		ui.label(
	// 			egui::RichText::new("Task Overview")
	// 				.size(24.0)
	// 				.strong()
	// 				.color(palette::TEXT),
	// 		);
	// 		ui.add_space(2.0);
	// 		ui.label(
	// 			egui::RichText::new("Estate Runtime")
	// 				.size(12.0)
	// 				.color(palette::TEXT_MUTED),
	// 		);
	// 	});
	// 	ui.add_space(16.0);
	// 	// =========================================================
	// 	// Summary metrics
	// 	// =========================================================
	// 	ui.columns(4, |columns| {
	// 		metric(
	// 			&mut columns[0],
	// 			"Tasks Created",
	// 			state.tasks_created,
	// 			palette::PRIMARY,
	// 		);
	// 		metric(
	// 			&mut columns[1],
	// 			"Tasks Completed",
	// 			state.tasks_completed,
	// 			palette::SUCCESS,
	// 		);
	// 		metric(
	// 			&mut columns[2],
	// 			"Events Processed",
	// 			state.events_processed,
	// 			palette::WARNING,
	// 		);
	// 		metric(
	// 			&mut columns[3],
	// 			"Status Checks",
	// 			state.status_checks,
	// 			palette::TEXT_MUTED,
	// 		);
	// 	});
	// 	ui.add_space(16.0);
	// 	// =========================================================
	// 	// Charts
	// 	// =========================================================
	// 	let available = ui.available_size();
	// 	let gap = 6.0;
	// 	let card_width = (available.x - gap) / 2.0;
	// 	let card_height = 280.0;
	// 	// =========================================================
	// 	// Jobs / Waterfall
	// 	// =========================================================
	// 	ui.add_space(16.0);
	// 	ui.label(
	// 		egui::RichText::new("Jobs")
	// 			.size(16.0)
	// 			.strong()
	// 			.color(palette::TEXT),
	// 	);
	// 	// Replace this with however your EstateState stores jobs.
	// 	self.draw_jobs(ui, &ctx.state().jobs);
	// 	ui.add_space(24.0);
	// 	// =========================================================
	// 	// Aggregate Charts
	// 	// =========================================================
	// 	let available = ui.available_size();
	// 	let gap = 6.0;
	// 	let card_width = (available.x - gap) / 2.0;
	// 	let card_height = 280.0;
	// 	render_graphs(ui, state, available, gap, card_width, card_height);
	// }
}
impl TaskManagerView {
	fn draw_jobs(&self, ui: &mut egui::Ui, jobs: &std::collections::VecDeque<Job>) {
		let now = EstateState::now();
		let timeline_start = jobs
			.iter()
			.filter_map(|job| job.started_at)
			.min()
			.unwrap_or(now);
		let timeline_end = jobs
			.iter()
			.filter_map(|job| job.completed_at)
			.max()
			.unwrap_or(now)
			.max(timeline_start + 1);
		TableBuilder::new(ui)
			.striped(true)
			.resizable(true)
			.column(Column::initial(160.0).at_least(100.0))
			.column(Column::initial(90.0).at_least(70.0))
			.column(Column::initial(120.0).at_least(80.0))
			.column(Column::remainder())
			.header(24.0, |mut header| {
				header.col(|ui| {
					ui.label("Name");
				});
				header.col(|ui| {
					ui.label("Status");
				});
				header.col(|ui| {
					ui.label("Origin");
				});
				header.col(|ui| {
					ui.label("Timeline");
				});
			})
			.body(|mut body| {
				for job in jobs {
					body.row(28.0, |mut row| {
						row.col(|ui| {
							ui.label(job.kind.clone().name());
						});
						row.col(|ui| {
							ui.label(format!("{} {}", job.status.icon(), job.status.label()));
						});
						row.col(|ui| {
							ui.label("Runtime");
						});
						row.col(|ui| {
							self
								.waterfall
								.draw_job(ui, job, timeline_start, timeline_end);
						});
					});
				}
			});
	}
	fn draw_job(
		&self,
		ui: &mut egui::Ui,
		job: &Job,
		name_width: f32,
		status_width: f32,
		duration_width: f32,
		timeline_width: f32,
	) {
		let started_at = job.started_at.unwrap_or(0);
		let ended_at = match job.completed_at {
			Some(completed_at) => completed_at,
			None => EstateState::now(),
		};
		let duration_ms = ended_at.saturating_sub(started_at);
		ui.horizontal(|ui| {
			// -----------------------------------------------------
			// Job name
			// -----------------------------------------------------
			ui.add_sized(
				[name_width, 32.0],
				egui::Label::new(egui::RichText::new(job.kind.name()).color(palette::TEXT)),
			);
			// -----------------------------------------------------
			// Status
			// -----------------------------------------------------
			ui.add_sized(
				[status_width, 32.0],
				egui::Label::new(format!("{} {}", job.status.icon(), job.status.label())),
			);
			// -----------------------------------------------------
			// Duration
			// -----------------------------------------------------
			ui.add_sized(
				[duration_width, 32.0],
				egui::Label::new(
					egui::RichText::new(format_duration_ms(duration_ms)).color(palette::TEXT_MUTED),
				),
			);
			// -----------------------------------------------------
			// Waterfall
			// -----------------------------------------------------
			self.draw_job_timeline(ui, job, started_at, ended_at, timeline_width);
		});
	}
	fn draw_job_timeline(
		&self,
		ui: &mut egui::Ui,
		job: &Job,
		started_at: u64,
		ended_at: u64,
		width: f32,
	) {
		let height = 32.0;
		let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
		let now = EstateState::now();
		// ---------------------------------------------------------
		// Determine the global timeline window.
		//
		// For now this is relative to the runtime's current
		// observation window.
		// ---------------------------------------------------------
		let timeline_start = started_at.min(now);
		let timeline_end = ended_at.max(timeline_start + 1);
		let total = (timeline_end - timeline_start).max(1) as f32;
		// ---------------------------------------------------------
		// Convert timestamps -> pixels
		// ---------------------------------------------------------
		let x = |timestamp: u64| {
			let offset = timestamp.saturating_sub(timeline_start) as f32;
			response.rect.left() + (offset / total) * width
		};
		let x1 = x(started_at);
		let x2 = x(ended_at);
		let bar_rect = egui::Rect::from_min_max(
			egui::pos2(x1, response.rect.top() + 8.0),
			egui::pos2(x2.max(x1 + 2.0), response.rect.bottom() - 8.0),
		);
		// ---------------------------------------------------------
		// Background
		// ---------------------------------------------------------
		painter.rect_filled(response.rect, 0.0, palette::SURFACE);
		// ---------------------------------------------------------
		// Job bar
		// ---------------------------------------------------------
		painter.rect_filled(
			bar_rect,
			2.0,
			match job.status {
				JobStatus::Completed => palette::SUCCESS,
				JobStatus::Running => palette::PRIMARY,
				JobStatus::Failed => palette::ERROR,
				_ => palette::TEXT_MUTED,
			},
		);
		// ---------------------------------------------------------
		// Hover
		// ---------------------------------------------------------
		if response.hovered() {
			painter.rect_stroke(
				bar_rect,
				2.0,
				egui::Stroke::new(1.0, palette::TEXT),
				egui::StrokeKind::Outside,
			);
			response.on_hover_text(format!(
				"{}\n{}\n{}",
				job.kind.name(),
				job.status.label(),
				format_duration_ms(ended_at.saturating_sub(started_at)),
			));
		}
	}
}
impl<R: Runtime> ViewTrait<R> for TaskManagerView {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		// compose child views
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
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
