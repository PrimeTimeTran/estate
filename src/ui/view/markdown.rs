use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::PathBuf;

use crate::app::*;

pub struct MarkdownView {
	path: PathBuf,
	markdown: String,
	cache: CommonMarkCache,
}
impl MarkdownView {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		let path = path.into();
		let markdown = std::fs::read_to_string(&path)
			.unwrap_or_else(|err| format!("# Failed to read Markdown\n\n`{err}`"));

		Self {
			path,
			markdown,
			cache: CommonMarkCache::default(),
		}
	}
}
impl<R: Runtime> Veable<R> for MarkdownView {
	fn draw(&mut self, ui: &mut Ui, _ctx: &mut AppContext<'_, R>) {
		egui::ScrollArea::vertical()
			.auto_shrink([false, false])
			.show(ui, |ui| {
				CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown);
			});
	}
}
