use strum::IntoStaticStr;

use crate::prelude::*;
use std::fmt;

#[cfg(feature = "native")]
use crate::native::{Dashboard, WaterfallChart};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
pub enum WindowType {
	EguiVeable,
	Dashboard,
	WaterfallChart,
	OracleView,
	TaskManager,
	Markdown,
	ProblemsScreen,
}

impl WindowType {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}

#[derive(
	Debug, Copy, Default, Clone, Hash, Deserialize, Serialize, Eq, PartialEq, IntoStaticStr,
)]
pub enum ViewType {
	Dashboard,
	WaterfallChart,
	OracleView,
	TaskManager,
	MarkdownView,
	#[default]
	Markdown,
	ProblemsScreen,
}

impl ViewType {
	pub fn name(&self) -> &'static str {
		self.into()
	}
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
	pub fn new(kind: ViewType, api: Arc<ApiClient>) -> Self {
		let content = match kind {
			ViewType::Dashboard => Ve::new(Dashboard::new()),
			ViewType::TaskManager => Ve::new(TaskManager::new()),
			ViewType::WaterfallChart => Ve::new(WaterfallChart::new()),
			ViewType::ProblemsScreen => Ve::new(ProblemsScreen::new()),
			ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::MARKDOWN)),
			_ => Ve::new(MarkdownView::new(crate::MARKDOWN)),
		};
		Self { kind, content }
	}
	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		self.content.draw(ui, ctx);
	}
}

// pub struct View<R: Runtime> {
// 	pub kind: ViewType,
// 	pub content: Ve<R>,
// }

// impl<R: Runtime> fmt::Debug for View<R> {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		f.debug_struct("View").field("kind", &self.kind).finish()
// 	}
// }

// impl<R: Runtime> View<R> {
// 	pub fn new(kind: ViewType, api: Arc<ApiClient>) -> Self {
// 		let content = match kind {
// 			// ViewType::Dashboard => Ve::new(Dashboard::new()),
// 			// ViewType::TaskManager => Ve::new(TaskManager::new()),
// 			// ViewType::WaterfallChart => Ve::new(WaterfallChart::new()),
// 			// ViewType::ProblemsScreen => Ve::new(ProblemsScreen::new()),
// 			ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::MARKDOWN)),
// 			_ => Ve::new(MarkdownView::new(crate::MARKDOWN)),
// 		};

// 		Self { kind, content }
// 	}

// 	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
// 		self.content.draw(ui, ctx);
// 	}
// }

// impl View {
// 	pub fn new(kind: ViewType, api: Arc<ApiClient>) -> Self {
// 		let content = match kind {
// 			ViewType::Dashboard => Ve::new(Dashboard::new()),
// 			ViewType::TaskManager => Ve::new(TaskManager::new()),
// 			ViewType::WaterfallChart => Ve::new(WaterfallChart::new()),
// 			ViewType::ProblemsScreen => Ve::new(ProblemsScreen::new()),
// 			ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::MARKDOWN)),
// 			_ => Ve::new(MarkdownView::new(crate::MARKDOWN)),
// 		};
// 		Self { kind, content }
// 	}
// 	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
// 		self.content.draw(ui, ctx);
// 	}
// }
