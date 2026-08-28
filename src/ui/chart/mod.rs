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
