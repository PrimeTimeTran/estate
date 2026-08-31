use egui::Ui;
use tonic::transport::Channel;

use crate::{
	app::state::EstateState,
	proto::leetcode::{
		ListProblemsRequest, PageRequest, Problem, problem_service_client::ProblemServiceClient,
	},
	repo::problem::StoredProblem,
	*,
};

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

impl<R: Runtime + 'static> Veable<R> for ProblemsScreen {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		let should_load = {
			let state = ctx.app.app_state();

			!state.problems_loading && state.problems.is_empty() && state.problems_error.is_none()
		};

		if should_load {
			ctx.app.load_problems();
		}

		ui.heading("Problems");
		ui.add_space(8.0);

		let state = ctx.app.app_state();

		if state.problems_loading {
			ui.spinner();
			return;
		}

		if let Some(error) = &state.problems_error {
			ui.colored_label(egui::Color32::RED, error);
			return;
		}

		if state.problems.is_empty() {
			ui.label("No problems found.");
			return;
		}

		egui::ScrollArea::vertical().show(ui, |ui| {
			for problem in &state.problems {
				self.draw_problem(ui, problem);
			}
		});
	}
}
