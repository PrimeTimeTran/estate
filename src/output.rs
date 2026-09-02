use crate::{
	e,
	model::{StoredProblem, StoredSolution, StoredSubmission},
	prelude::*,
	ui::Layout,
};
use egui::Ui;

#[derive(Debug, Default)]
pub struct ProblemScreen<R: Runtime> {
	source: String,
	submission_status: Option<SubmissionStatus>,
	// Screen-level state shared by multiple Views.
	solutions: Vec<StoredSolution>,
	submissions: Vec<StoredSubmission>,
	_marker: std::marker::PhantomData<R>,
}
impl<R: Runtime> ProblemScreen<R> {
	pub fn new() -> Self {
		Self {
			source: String::new(),
			submission_status: None,
			solutions: Vec::new(),
			submissions: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}
}
impl<R: Runtime> ViewTrait<R> for ProblemScreen<R> {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen view draw")
	}
	fn update(&mut self, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen view update")
	}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen view event")
	}
}
impl<R: Runtime> Screen<R> for ProblemScreen<R> {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen configure")
		// Configure the regions this screen uses.
	}
	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen update")
	}
	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		println!("Problem Screen screen eventupdate")
	}
}
