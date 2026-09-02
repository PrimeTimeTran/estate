use egui::Ui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::PathBuf;

use crate::{app::*, e, ui::Layout};

pub struct MarkdownScreen {
	path: PathBuf,
	markdown: String,
	cache: CommonMarkCache,
}
impl MarkdownScreen {
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
	// fn draw(&mut self, ui: &mut Ui, _ctx: &mut AppContext<'_, R>) {
	// 	egui::ScrollArea::vertical()
	// 		.auto_shrink([false, false])
	// 		.show(ui, |ui| {
	// 			CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown);
	// 		});
	// }
}
impl<R: Runtime> Screen<R> for MarkdownScreen {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Configure the regions this screen uses.
	}
	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
}
