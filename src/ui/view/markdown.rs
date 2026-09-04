use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{app::*, e, ui::Layout};

pub struct MarkdownScreen {
	markdown: String,
	cache: CommonMarkCache,
}

impl MarkdownScreen {
	pub fn new(markdown: impl Into<String>) -> Self {
		Self {
			markdown: markdown.into(),
			cache: CommonMarkCache::default(),
		}
	}

	pub fn draw(&mut self, ui: &mut Ui) {
		CommonMarkViewer::new().show(ui, &mut self.cache, &mut self.markdown);
	}
}
impl<R: Runtime, E: Executor> Screen<R, E> for MarkdownScreen {
	fn configure(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {}
	fn update(&mut self, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {}
	fn event(&mut self, event: &e::Event, layout: &mut Layout<R, E>, ctx: &mut AppContext<'_, R, E>) {
	}
}
