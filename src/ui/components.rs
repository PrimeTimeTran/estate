use egui::{ScrollArea, Ui};
use std::fmt;
use strum::IntoStaticStr;

use crate::{e, prelude::*, ui::Layout};

#[cfg(feature = "native")]
use crate::native::{DashboardScreen, WaterfallChart, prelude::*};

pub(crate) struct ScreenInstance {
	pub kind: ViewType,
	pub screen: Box<dyn Screen<NativeRuntime>>,
	pub layout: Layout<NativeRuntime>,
}

impl fmt::Debug for ScreenInstance {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("View").field("kind", &self.kind).finish()
	}
}
impl ScreenInstance {
	pub fn new(kind: ViewType, api: Arc<ApiClient>) -> Self {
		let screen: Box<dyn Screen<NativeRuntime>> = match kind {
			ViewType::DashboardScreen => Box::new(DashboardScreen::new()),
			ViewType::TaskManagerScreen => Box::new(TaskManagerScreen::new()),
			// ViewType::WaterfallScreen => Box::new(WaterfallScreen::new()),
			ViewType::ProblemsScreen => Box::new(ProblemsScreen::new()),
			ViewType::ProblemScreen => Box::new(ProblemScreen::new()),
			ViewType::MarkdownView => Box::new(MarkdownScreen::new(crate::MARKDOWN)),
			_ => Box::new(MarkdownScreen::new(crate::MARKDOWN)),
		};

		Self {
			kind,
			screen,
			layout: Layout::new(),
		}
	}

	pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
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

//
pub fn draw_tabbed_sidebar<T, F>(
	ui: &mut Ui,
	active_tab: &mut T,
	tabs: &[(T, &str)],
	mut draw_content: F,
) where
	T: Clone + PartialEq,
	F: FnMut(&mut Ui, &T),
{
	// Tab bar
	ui.horizontal(|ui| {
		for (tab, label) in tabs {
			let selected = *active_tab == *tab;

			if ui.selectable_label(selected, *label).clicked() {
				*active_tab = tab.clone();
			}
		}
	});

	ui.separator();

	// Tab content
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

pub struct ActivityBar {
	buttons: Vec<&'static str>,
}
impl ActivityBar {
	pub fn new() -> Self {
		Self { buttons: vec![] }
	}
}
impl<R: Runtime> ViewTrait<R> for ActivityBar {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		ui.vertical(|ui| {
			// buttons
		});
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}
pub struct PrimaryBar {
	buttons: Vec<&'static str>,
}

impl<R: Runtime> ViewTrait<R> for PrimaryBar {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		ui.horizontal(|ui| {
			// buttons
		});
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}
