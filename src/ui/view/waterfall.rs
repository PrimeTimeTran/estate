use crate::{app::state::EstateState, e, prelude::*, theme::palette, ui::Layout};

#[derive(Debug, Default, Clone, Copy)]
pub struct WaterfallScreen;

impl WaterfallScreen {
	pub fn new() -> Self {
		Self
	}
}

impl<R: Runtime, E: Executor> Screen<R, E> for WaterfallScreen {
	fn configure(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
		// Configure the regions this screen uses.
	}

	fn update(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
	}
}

#[derive(Debug, Default, Clone)]
pub struct WaterfallChart {
	jobs: Vec<&'static Job>,
}
impl<R: Runtime, E: Executor> ViewTrait<R, E> for WaterfallChart {
	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R, E>) {
		if ctx.state_changed() {
			ui.ctx().request_repaint();
		}
		ui.heading("Job History");
		let state = ctx.state();
		self.draw_chart(ui, state.jobs.iter());
	}
}
impl WaterfallChart {
	// 	pub fn new() -> Self {
	// 	Self
	// }
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
		// Oldest -> newest.
		timed_jobs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
		let mut lanes: Vec<f64> = Vec::new();
		let mut bars = Vec::with_capacity(timed_jobs.len());
		let mut min_time = f64::MAX;
		let mut max_time = f64::MIN;
		for (job, start, end) in timed_jobs {
			min_time = min_time.min(start);
			max_time = max_time.max(end);
			let lane = lanes
				.iter()
				.position(|lane_end| *lane_end <= start)
				.unwrap_or_else(|| {
					lanes.push(0.0);
					lanes.len() - 1
				});
			lanes[lane] = end;
			bars.push(
				Bar::new(lane as f64, end - start)
					.horizontal()
					.base_offset(start)
					.width(0.7)
					.name(job.kind.name()),
			);
		}
		let padding = ((max_time - min_time) * 0.05).max(1.0);
		Plot::new("job_history")
			.height(300.0)
			.include_x(min_time - padding)
			.include_x(max_time + padding)
			.include_y(-0.75)
			.include_y(lanes.len() as f64)
			.show_axes([true, true])
			.show_grid([true, true])
			.legend(egui_plot::Legend::default())
			.x_axis_formatter(|mark, _range| format_timestamp(mark.value))
			.show(ui, |plot_ui| {
				// Keep the Y viewport bounded.
				let mut bounds = plot_ui.plot_bounds();
				bounds.set_y_center_height(bounds.center().y.clamp(0.0, 20.0), 20.0);
				plot_ui.set_plot_bounds(bounds);
				plot_ui.bar_chart(BarChart::new("jobs", bars).horizontal());
			});
	}
	pub fn draw_job(&self, ui: &mut Ui, job: &Job, timeline_start: u64, timeline_end: u64) {
		let height = 28.0;
		let (response, painter) = ui.allocate_painter(
			egui::vec2(ui.available_width(), height),
			egui::Sense::hover(),
		);
		let total = (timeline_end - timeline_start).max(1) as f32;
		let x = |timestamp: u64| {
			let offset = timestamp.saturating_sub(timeline_start) as f32;
			response.rect.left() + (offset / total) * response.rect.width()
		};
		let started_at = job.started_at.unwrap_or(timeline_start);
		let ended_at = job.completed_at.unwrap_or_else(EstateState::now);
		let x1 = x(started_at);
		let x2 = x(ended_at);
		let bar_rect = egui::Rect::from_min_max(
			egui::pos2(x1, response.rect.top() + 5.0),
			egui::pos2(x2.max(x1 + 2.0), response.rect.bottom() - 5.0),
		);
		painter.rect_filled(response.rect, 0.0, palette::SURFACE);
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
