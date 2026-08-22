use crate::prelude::*;
// use std::fs;
// use std::path::Path;
// use anyhow::Result;
// use serde::Deserialize;
use egui::Context;
use egui_plot::{Bar, BarChart, Plot};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ChartData {
	pub title: String,
	pub bars: Vec<ChartBar>,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ChartBar {
	pub label: String,
	pub value: f64,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Preview {
	data: ChartData,
}

impl Preview {
	pub fn load(path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(path)?;
		let data = serde_json::from_str(&contents)?;

		Ok(Self { data })
	}

	pub fn ui(&self, ui: &mut egui::Ui) {
		ui.heading(&self.data.title);

		let bars = self
			.data
			.bars
			.iter()
			.enumerate()
			.map(|(index, bar)| Bar::new(index as f64, bar.value))
			.collect::<Vec<_>>();

		let chart = BarChart::new("values", bars);

		Plot::new("chart")
			.height(ui.available_height())
			.show(ui, |plot_ui| {
				plot_ui.bar_chart(chart);
			});
	}
}

#[derive(Clone, Debug, Default)]
pub struct Window {
  pub is_visible: bool,
	pub preview: Preview,
}
