use std::fmt;

use crate::{
	native::{Dashboard, WaterfallChart},
	prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
	EguiVeable,
	Dashboard,
	WaterfallChart,
	TelemetryInspector,
	TaskManager,
	Markdown,
}

#[derive(Debug, Copy, Default, Clone, Hash, Deserialize, Serialize, Eq, PartialEq)]
pub enum ViewType {
	Dashboard,
	WaterfallChart,
	TelemetryInspector,
	TaskManager,
	MarkdownView,
	#[default]
	Markdown,
}

pub struct View {
	pub kind: ViewType,
	pub content: Ve<NativeRuntime>,
}

impl fmt::Debug for View {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		Ok(())
	}
}

impl View {
	pub fn new(kind: ViewType) -> Self {
		let content = match kind {
			ViewType::Dashboard => Ve::new(Dashboard::new()),
			ViewType::TaskManager => Ve::new(TaskManager::new()),
			ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::data::MARKDOWN)),
			ViewType::WaterfallChart => Ve::new(WaterfallChart::new()),
			// ViewType::TelemetryInspector => Ve::new(TelemetryInspectorView::new()),
			_ => Ve::new(MarkdownView::new(crate::data::MARKDOWN)),
		};

		Self { kind, content }
	}

	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		self.content.draw(ui, ctx);
	}
}
