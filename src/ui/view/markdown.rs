use crate::prelude::*;

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
impl<C, S> Screen<C, S> for MarkdownScreen
where
	C: Ctx,
{
	fn configure(&mut self, layout: &mut Layout<C, S>, _ctx: &mut AppContext<'_, C, S>) {}
	fn update(&mut self, layout: &mut Layout<C, S>, _ctx: &mut AppContext<'_, C, S>) {}
	fn event(
		&mut self,
		_event: &e::Event,
		_layout: &mut Layout<C, S>,
		_ctx: &mut AppContext<'_, C, S>,
	) {
	}
}
