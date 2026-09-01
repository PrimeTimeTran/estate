use crate::{model::problem::StoredProblem, *};
use egui::Ui;

#[derive(Debug, Default)]
pub struct ProblemScreen;

impl ProblemScreen {
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
impl<R: Runtime + 'static> Veable<R> for ProblemScreen {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		let (loading, error, problem) = {
			let state = ctx.app.app_state();

			(
				state.problem.loading,
				state.problem.error.clone(),
				state.problem.value.clone(),
			)
		};

		// ui.horizontal(|ui| {
		// 	if ui.button("← Back").clicked() {
		// 		ctx.app.navigate(ViewType::ProblemsScreen);
		// 	}

		// 	ui.heading("Problem");
		// });

		ui.add_space(8.0);

		if loading {
			ui.horizontal(|ui| {
				ui.spinner();
				ui.label("Loading problem...");
			});
			return;
		}

		if let Some(error) = error {
			ui.colored_label(
				egui::Color32::RED,
				format!("Failed to load problem: {error}"),
			);

			if ui.button("Retry").clicked() {
				ctx.app.sample_problem();
			}

			return;
		}

		if let Some(problem) = problem {
			self.draw_problem(ui, &problem);

			ui.add_space(12.0);

			if ui.button("Sample Another Problem").clicked() {
				ctx.app.sample_problem();
			}
		} else {
			ui.label("No problem loaded.");

			if ui.button("Sample Problem").clicked() {
				ctx.app.sample_problem();
			}
		}
	}
}
// impl<R: Runtime + 'static> Veable<R> for ProblemScreen {
// 	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
// 		let (loading, error, problem) = {
// 			let state = ctx.app.app_state();

// 			(
// 				state.problem.loading,
// 				state.problem.error.clone(),
// 				state.problem.value.clone(),
// 			)
// 		};

// 		ui.horizontal(|ui| {
// 			if ui.button("← Back").clicked() {
// 				ctx.app.navigate(ViewType::ProblemsScreen);
// 			}

// 			ui.heading("Problem");
// 		});

// 		ui.add_space(8.0);

// 		if loading {
// 			ui.horizontal(|ui| {
// 				ui.spinner();
// 				ui.label("Loading problem...");
// 			});
// 			return;
// 		}

// 		if let Some(error) = error {
// 			ui.colored_label(
// 				egui::Color32::RED,
// 				format!("Failed to load problem: {error}"),
// 			);

// 			if ui.button("Retry").clicked() {
// 				if let Some(problem) = &problem {
// 					ctx.app.get_problem(problem.id);
// 				}
// 			}

// 			return;
// 		}

// 		let Some(problem) = problem else {
// 			ui.label("No problem selected.");
// 			return;
// 		};

// 		ui.horizontal(|ui| {
// 			ui.strong(&problem.title);
// 			ui.separator();
// 			ui.monospace(&problem.slug);

// 			if ui.button("Reload").clicked() {
// 				ctx.app.get_problem(problem.id);
// 			}
// 		});

// 		ui.add_space(12.0);

// 		self.draw_problem(ui, &problem);
// 	}
// }

// #[derive(Debug, Clone)]
// enum RequestState {
// 	Idle,
// 	Loading,
// 	Success,
// 	Error(String),
// }

// impl Default for RequestState {
// 	fn default() -> Self {
// 		Self::Idle
// 	}
// }

// #[derive(Debug, Default)]
// pub struct ProblemScreen {
// 	request: RequestState,
// }
