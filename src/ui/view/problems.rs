use crate::{
	model::problem::StoredProblem,
	ui::{Component, r#trait::Screen},
	*,
};
use egui::Ui;

#[derive(Debug, Default)]
pub struct ProblemsScreen;
impl ProblemsScreen {
	pub fn new() -> Self {
		Self
	}
	fn draw_problem(&self, ui: &mut egui::Ui, problem: &StoredProblem) {
		egui::Frame::group(ui.style()).show(ui, |ui| {
			ui.horizontal(|ui| {
				ui.strong(&problem.title);
				ui.separator();
				ui.monospace(&problem.slug);
			});
			ui.label(format!("ID: {}", problem.id));
		});
	}
}

impl<R: Runtime> Screen<R> for ProblemsScreen {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Configure the regions this screen uses.
	}

	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
}

impl ProblemsScreen {
	// fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
	// 	let should_load = {
	// 		let state = ctx.app.app_state();

	// 		!state.problems.loading && state.problems.items.is_empty() && state.problems.error.is_none()
	// 	};

	// 	if should_load {
	// 		tracing::info!("🔥 Loading problems");
	// 		ctx.app.load_problems();
	// 	}

	// 	ui.heading("Problems");
	// 	ui.add_space(8.0);

	// 	let (loading, error, problems) = {
	// 		let state = ctx.app.app_state();

	// 		(
	// 			state.problems.loading,
	// 			state.problems.error.clone(),
	// 			state.problems.items.clone(),
	// 		)
	// 	};

	// 	if loading {
	// 		ui.horizontal(|ui| {
	// 			ui.spinner();
	// 			ui.label("Loading problems...");
	// 		});
	// 		return;
	// 	}

	// 	if let Some(error) = error {
	// 		ui.colored_label(egui::Color32::RED, error);
	// 		return;
	// 	}

	// 	if problems.is_empty() {
	// 		ui.label("No problems found.");
	// 		return;
	// 	}

	// 	ui.label(format!("{} problems", problems.len()));
	// 	ui.add_space(8.0);

	// 	egui::ScrollArea::vertical().show(ui, |ui| {
	// 		for problem in &problems {
	// 			self.draw_problem(ui, problem);
	// 		}
	// 	});
	// }
}
