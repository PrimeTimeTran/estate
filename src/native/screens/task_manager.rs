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

impl TaskManager {
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
					ui.strong(job.kind.name());
					ui.small(format!("Job #{}", job.id));
				});

				ui.add_space(20.0);

				// Status
				ui.label(job.status.label());

				ui.add_space(20.0);

				// Duration
				if let Some(started_at) = job.started_at {
					let duration_ms = match job.completed_at {
						Some(completed_at) => completed_at.saturating_sub(started_at),
						None => {
							// Still running. Calculate elapsed wall-clock time.
							EstateState::now().saturating_sub(started_at)
						}
					};

					ui.label(format_duration_ms(duration_ms));
				}

				ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
					ui.button("⋮");
				});
			});
		});
	}
}
impl Veable for TaskManager {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		if self.poll_changes() {
			ui.ctx().request_repaint();
		}
		if let Some(error) = &self.state.error {
			ui.heading("Task Manager");
			ui.colored_label(palette::ERROR, error);
			ui.label(self.state.state_path.display().to_string());
			return;
		}
		let Some(state) = &self.state.state else {
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