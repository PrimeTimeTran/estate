use crate::{app::*, prelude::*};

use ::serde::Deserialize;
use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points};

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

#[derive(Clone, Debug, Deserialize)]
pub struct ChartsFile {
	pub charts: Vec<Chart>,
}
impl ChartsFile {
	pub fn load(path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&contents)?)
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

		// (job, start, end)
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
impl Veable for WaterfallChart {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		if ctx.poll_state() {
			ui.ctx().request_repaint();
		}

		ui.heading("Job History");

		let state = ctx.state();

		self.draw_chart(ui, state.jobs.iter());
	}
}
