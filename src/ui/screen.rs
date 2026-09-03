use egui::{ScrollArea, Ui};
use std::fmt;
use strum::IntoStaticStr;

use crate::{api::Api, e, prelude::*, ui::Layout};

#[cfg(not(target_arch = "wasm32"))]
use crate::native::{DashboardScreen, WaterfallChart, prelude::*};

pub(crate) struct ScreenInstance<R: Runtime, E: Executor> {
	pub kind: ViewType,
	pub screen: Box<dyn Screen<R, E>>,
	pub layout: Layout<R, E>,
}

impl<R: Runtime, E: Executor> fmt::Debug for ScreenInstance<R, E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("View").field("kind", &self.kind).finish()
	}
}

impl<R: Runtime, E: Executor> ScreenInstance<R, E> {
	pub fn new(kind: ViewType) -> Self {
		tracing::debug!("📺 Screen Instance {:?}", kind);
		let screen: Box<dyn Screen<R, E>> = match kind {
			// ViewType::DashboardScreen => Box::new(DashboardScreen::new()),
			ViewType::MarkdownView => Box::new(MarkdownScreen::new(crate::MARKDOWN)),
			ViewType::ProblemScreen => Box::new(ProblemScreen::new()),
			ViewType::ProblemsScreen => Box::new(ProblemsScreen::new()),
			ViewType::TaskManagerScreen => Box::new(TaskManagerScreen::new()),
			ViewType::WaterfallScreen => Box::new(WaterfallScreen::new()),
			_ => Box::new(MarkdownScreen::new(crate::MARKDOWN)),
		};
		Self {
			kind,
			screen,
			layout: Layout::new(),
		}
	}
	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		self.layout.draw(ui, ctx);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
pub(crate) enum WindowType {
	EguiVeable,
	DashboardScreen,
	WaterfallScreen,
	OracleScreen,
	TaskManagerScreen,
	MarkdownScreen,
	ProblemsScreen,
	ProblemScreen,
}

impl WindowType {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}
#[derive(
	Debug, Copy, Default, Clone, Hash, Deserialize, Serialize, Eq, PartialEq, IntoStaticStr,
)]
pub(crate) enum ViewType {
	#[default]
	DashboardScreen,
	MarkdownScreen,
	MarkdownView,
	OracleScreen,
	ProblemScreen,
	ProblemsScreen,
	TaskManagerScreen,
	WaterfallScreen,
}

impl ViewType {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}

pub fn draw_tabbed_sidebar<T, F>(
	ui: &mut Ui,
	active_tab: &mut T,
	tabs: &[(T, &str)],
	mut draw_content: F,
) where
	T: Clone + PartialEq,
	F: FnMut(&mut Ui, &T),
{
	ui.horizontal(|ui| {
		for (tab, label) in tabs {
			let selected = *active_tab == *tab;

			if ui.selectable_label(selected, *label).clicked() {
				*active_tab = tab.clone();
			}
		}
	});

	ui.separator();
	ScrollArea::vertical()
		.auto_shrink([false, false])
		.show(ui, |ui| {
			draw_content(ui, active_tab);
		});
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

// 	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
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
