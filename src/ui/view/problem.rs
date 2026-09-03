use crate::api::Api;
use crate::model::{ProtoLanguage, StoredProblem, StoredSolution, StoredSubmission};

use crate::proto::types::SubmissionStatus;

use crate::{e, prelude::*, ui::Layout};
use egui::{ScrollArea, Ui};

#[derive(Debug, Default)]
pub struct ProblemScreen<R: Runtime> {
	source: String,
	submission_status: Option<SubmissionStatus>,
	solutions: Vec<StoredSolution>,
	submissions: Vec<StoredSubmission>,
	_marker: std::marker::PhantomData<R>,
}
impl<R: Runtime> ProblemScreen<R> {
	pub fn new() -> Self {
		tracing::debug!("ProblemScreen new");
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
		// println!("Problem Screen view draw")
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

#[derive(Debug, Default)]
pub struct ProblemViewSidebar<R: Runtime> {
	active_tab: Tab,
	solutions: Vec<StoredSolution>,
	submissions: Vec<StoredSubmission>,
	_marker: std::marker::PhantomData<R>,
}
impl<R: Runtime> ProblemViewSidebar<R> {
	pub fn new() -> Self {
		Self {
			active_tab: Tab::Problem,
			solutions: Vec::new(),
			submissions: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}
}
impl<R: Runtime> ViewTrait<R> for ProblemViewSidebar<R> {
	fn draw(&mut self, ui: &mut Ui, _ctx: &mut AppContext<'_, R>) {
		ui.heading("Problem");
		ui.separator();
		self.draw_solutions(ui);
		self.draw_submissions(ui);
	}
	fn update(&mut self, _ctx: &mut AppContext<'_, R>) {}
	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, R>) {}
}
impl<R: Runtime> ProblemViewSidebar<R> {
	fn draw_solutions(&self, ui: &mut Ui) {
		ui.heading("Solutions");
		if self.solutions.is_empty() {
			ui.label("No solutions available.");
			return;
		}
		ScrollArea::vertical()
			.auto_shrink([false, false])
			.show(ui, |ui| {
				for solution in &self.solutions {
					ui.group(|ui| {
						ui.horizontal(|ui| {
							ui.strong(&solution.title);

							ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
								if ui.button("View").clicked() {
									// Navigate to solution detail.
								}
							});
						});

						ui.add_space(4.0);

						ui.horizontal_wrapped(|ui| {
							for code in &solution.code {
								let language = ProtoLanguage::try_from(code.language)
									.map(|language| format!("{language:?}"))
									.unwrap_or_else(|_| "Unknown".to_string());

								ui.label(language);
							}
						});

						ui.add_space(4.0);

						ui.horizontal_wrapped(|ui| {
							if !solution.approach.is_empty() {
								ui.label(format!("Approach: {}", solution.approach));
							}

							if !solution.time_complexity.is_empty() {
								ui.label(format!("Time: {}", solution.time_complexity));
							}

							if !solution.space_complexity.is_empty() {
								ui.label(format!("Space: {}", solution.space_complexity));
							}
						});

						ui.add_space(4.0);

						ui.horizontal(|ui| {
							ui.label(format!("👁 {}", solution.view_count));
							ui.label(format!("▲ {}", solution.vote_count));

							if let Some(created_at) = solution.created_at {
								ui.label(created_at.format("%Y-%m-%d").to_string());
							}
						});
					});

					ui.add_space(6.0);
				}
			});
	}
	fn draw_submissions(&self, ui: &mut Ui) {
		ui.separator();
		ui.heading("Submissions");

		if self.submissions.is_empty() {
			ui.label("No submissions yet.");
			return;
		}

		ScrollArea::vertical()
			.auto_shrink([false, false])
			.show(ui, |ui| {
				for submission in &self.submissions {
					let clicked = ui
						.group(|ui| {
							ui.vertical(|ui| {
								ui.horizontal(|ui| {
									ui.strong(format!("{:?}", submission.status));

									ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
										ui.label("→");
									});
								});

								ui.horizontal_wrapped(|ui| {
									ui.label(format!("{:?}", submission.language));

									if let (Some(passed), Some(total)) =
										(submission.tests_passed, submission.tests_total)
									{
										ui.label(format!("· {passed}/{total} tests"));
									}

									if let Some(runtime_ms) = submission.runtime_ms {
										ui.label(format!("· {runtime_ms} ms"));
									}

									if let Some(memory_bytes) = submission.memory_bytes {
										let memory_kb = memory_bytes as f64 / 1024.0;
										ui.label(format!("· {memory_kb:.1} KB"));
									}
								});

								if let Some(created_at) = submission.created_at {
									ui.small(created_at.format("%Y-%m-%d %H:%M:%S").to_string());
								}
							});
						})
						.response
						.clicked();

					if clicked {
						// Set selected submission / navigate to submission detail.
					}

					ui.add_space(4.0);
				}
			});
	}
}
#[derive(Debug, Default)]
pub struct ProblemViewBottomPanel<R: Runtime> {
	_marker: std::marker::PhantomData<R>,
}
impl<R: Runtime> ProblemViewBottomPanel<R> {
	pub fn new() -> Self {
		Self {
			_marker: std::marker::PhantomData,
		}
	}
}
impl<R: Runtime> ViewTrait<R> for ProblemViewBottomPanel<R> {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {}
	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}
#[derive(Debug, Default)]
pub struct ProblemView<R: Runtime> {
	_marker: std::marker::PhantomData<R>,
}
impl<R: Runtime> ProblemView<R> {
	pub fn new() -> Self {
		Self {
			_marker: std::marker::PhantomData,
		}
	}
}
impl<R: Runtime> ViewTrait<R> for ProblemView<R> {
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		let (loading, error, problem) = {
			let state = ctx.app.app_state();
			(
				state.problem.loading,
				state.problem.error.clone(),
				state.problem.value.clone(),
			)
		};

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

	fn update(&mut self, _ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, R>) {}
}

impl<R: Runtime> ProblemView<R> {
	fn draw_problem(&self, ui: &mut Ui, problem: &StoredProblem) {
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
impl<R: Runtime> ProblemView<R> {
	fn draw<A: Api>(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, R>) {
		let (loading, error, problem) = {
			let state = ctx.app.app_state();
			(
				state.problem.loading,
				state.problem.error.clone(),
				state.problem.value.clone(),
			)
		};
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
// impl<R: Runtime> ProblemView<R> {
// 	// fn draw_problem(&self, ui: &mut egui::Ui, problem: &StoredProblem) {
// 	// 	egui::Frame::group(ui.style()).show(ui, |ui| {
// 	// 		ui.horizontal(|ui| {
// 	// 			ui.strong(&problem.title);
// 	// 			ui.separator();
// 	// 			ui.monospace(&problem.slug);
// 	// 		});
// 	// 		ui.label(format!("ID: {}", problem.id));
// 	// 	});
// 	// }
// 	// fn draw_solutions(&self, ui: &mut Ui, solutions: &[StoredSolution]) {
// 	// 	ui.heading("Solutions");
// 	// 	if solutions.is_empty() {
// 	// 		ui.label("No solutions available.");
// 	// 		return;
// 	// 	}
// 	// 	ScrollArea::vertical()
// 	// 		.auto_shrink([false, false])
// 	// 		.show(ui, |ui| {
// 	// 			for solution in solutions {
// 	// 				ui.group(|ui| {
// 	// 					ui.horizontal(|ui| {
// 	// 						ui.strong(&solution.title);
// 	// 						ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
// 	// 							if ui.button("View").clicked() {
// 	// 								// Navigate to solution detail.
// 	// 							}
// 	// 						});
// 	// 					});
// 	// 					ui.add_space(4.0);
// 	// 					ui.horizontal_wrapped(|ui| {
// 	// 						for code in &solution.code {
// 	// 							let language = ProtoLanguage::try_from(code.language)
// 	// 								.map(|language| format!("{language:?}"))
// 	// 								.unwrap_or_else(|_| "Unknown".to_string());
// 	// 							ui.label(language);
// 	// 						}
// 	// 					});
// 	// 					ui.add_space(4.0);
// 	// 					ui.horizontal_wrapped(|ui| {
// 	// 						if !solution.approach.is_empty() {
// 	// 							ui.label(format!("Approach: {}", solution.approach));
// 	// 						}
// 	// 						if !solution.time_complexity.is_empty() {
// 	// 							ui.label(format!("Time: {}", solution.time_complexity));
// 	// 						}
// 	// 						if !solution.space_complexity.is_empty() {
// 	// 							ui.label(format!("Space: {}", solution.space_complexity));
// 	// 						}
// 	// 					});
// 	// 					ui.add_space(4.0);
// 	// 					ui.horizontal(|ui| {
// 	// 						ui.label(format!("👁 {}", solution.view_count));
// 	// 						ui.label(format!("▲ {}", solution.vote_count));
// 	// 						if let Some(created_at) = solution.created_at {
// 	// 							ui.label(created_at.format("%Y-%m-%d").to_string());
// 	// 						}
// 	// 					});
// 	// 				});
// 	// 				ui.add_space(6.0);
// 	// 			}
// 	// 		});
// 	// }
// 	// fn draw_submissions(&self, ui: &mut Ui, submissions: &[StoredSubmission]) {
// 	// 	ui.heading("Submissions");
// 	// 	ui.add_space(4.0);
// 	// 	if submissions.is_empty() {
// 	// 		ui.label("No submissions yet.");
// 	// 		return;
// 	// 	}
// 	// 	ScrollArea::vertical()
// 	// 		.auto_shrink([false, false])
// 	// 		.show(ui, |ui| {
// 	// 			for submission in submissions {
// 	// 				let clicked = ui
// 	// 					.group(|ui| {
// 	// 						ui.vertical(|ui| {
// 	// 							ui.horizontal(|ui| {
// 	// 								ui.strong(format!("{:?}", submission.status));
// 	// 								ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
// 	// 									ui.label("→");
// 	// 								});
// 	// 							});
// 	// 							ui.horizontal_wrapped(|ui| {
// 	// 								ui.label(format!("{:?}", submission.language));
// 	// 								if let (Some(passed), Some(total)) =
// 	// 									(submission.tests_passed, submission.tests_total)
// 	// 								{
// 	// 									ui.label(format!("· {passed}/{total} tests"));
// 	// 								}
// 	// 								if let Some(runtime_ms) = submission.runtime_ms {
// 	// 									ui.label(format!("· {runtime_ms} ms"));
// 	// 								}
// 	// 								if let Some(memory_bytes) = submission.memory_bytes {
// 	// 									let memory_kb = memory_bytes as f64 / 1024.0;
// 	// 									ui.label(format!("· {memory_kb:.1} KB"));
// 	// 								}
// 	// 							});
// 	// 							if let Some(created_at) = submission.created_at {
// 	// 								ui.small(created_at.format("%Y-%m-%d %H:%M:%S").to_string());
// 	// 							}
// 	// 						});
// 	// 					})
// 	// 					.response
// 	// 					.clicked();
// 	// 				if clicked {
// 	// 					// Set selected submission / navigate to submission detail.
// 	// 				}
// 	// 				ui.add_space(4.0);
// 	// 			}
// 	// 		});
// 	// }
// }
