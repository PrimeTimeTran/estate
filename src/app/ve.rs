use crate::app::{AppContext, Runtime};

pub trait Veable<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);
}
