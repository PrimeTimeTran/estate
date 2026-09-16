use crate::{
	model::{ProtoLanguage, StoredProblem, StoredSolution, StoredSubmission},
	prelude::*,
	proto::types::SubmissionStatus,
	ui::Layout,
};

use egui::{ScrollArea, Ui};

#[derive(Debug, Default)]
pub struct ProblemScreen<C> {
	idx: i32,
	source: String,
	submission_status: Option<SubmissionStatus>,
	solutions: Vec<StoredSolution>,
	submissions: Vec<StoredSubmission>,
	ticker: usize,

	_marker: std::marker::PhantomData<C>,
}
impl<C> ProblemScreen<C> {
	pub fn new() -> Self {
		tracing::debug!("ProblemScreen new");
		Self {
			idx: 0,
			ticker: 0,
			source: String::new(),
			submission_status: None,
			solutions: Vec::new(),
			submissions: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}
}

impl<C, S> ViewTrait<C, S> for ProblemScreen<C>
where
	C: Ctx,
{
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, C, S>) {
		// println!("Problem Screen view draw")
	}
	fn update(&mut self, ctx: &mut AppContext<'_, C, S>) {
		println!("Problem Screen view update")
	}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, C, S>) {
		println!("Problem Screen view event")
	}
}
impl<C, S> Screen<C, S> for ProblemScreen<C>
where
	C: Ctx,
{
	fn configure(&mut self, layout: &mut Layout<C, S>, ctx: &mut AppContext<'_, C, S>) {
		println!("Problem Screen configure")
		// Configure the regions this screen uses.
	}
	fn update(&mut self, layout: &mut Layout<C, S>, ctx: &mut AppContext<'_, C, S>) {
		println!("Problem Screen update")
	}
	fn event(&mut self, event: &e::Event, layout: &mut Layout<C, S>, ctx: &mut AppContext<'_, C, S>) {
		println!("Problem Screen screen eventupdate")
	}
}

#[derive(Debug, Default)]
pub struct ProblemViewSidebar<C> {
	active_tab: Tab,
	solutions: Vec<StoredSolution>,
	submissions: Vec<StoredSubmission>,
	_marker: std::marker::PhantomData<C>,
}
impl<C> ProblemViewSidebar<C> {
	pub fn new() -> Self {
		Self {
			active_tab: Tab::Problem,
			solutions: Vec::new(),
			submissions: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}
}
impl<C, S> ViewTrait<C, S> for ProblemViewSidebar<C>
where
	C: Ctx,
{
	fn draw(&mut self, ui: &mut Ui, _ctx: &mut AppContext<'_, C, S>) {
		ui.heading("Problem");
		ui.separator();
		self.draw_solutions(ui);
		self.draw_submissions(ui);
	}
	fn update(&mut self, _ctx: &mut AppContext<'_, C, S>) {}
	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, C, S>) {}
}
impl<C> ProblemViewSidebar<C> {
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
pub struct ProblemViewBottomPanel<C> {
	_marker: std::marker::PhantomData<C>,
}
impl<C> ProblemViewBottomPanel<C> {
	pub fn new() -> Self {
		Self {
			_marker: std::marker::PhantomData,
		}
	}
}
impl<C, S> ViewTrait<C, S> for ProblemViewBottomPanel<C>
where
	C: Ctx,
{
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, C, S>) {}
	fn update(&mut self, ctx: &mut AppContext<'_, C, S>) {}
	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, C, S>) {}
}
#[derive(Debug, Default)]
pub struct ProblemView<C, S> {
	ticker: usize,
	_marker: std::marker::PhantomData<(C, S)>,
}
impl<C, S> ProblemView<C, S> {
	pub fn new() -> Self {
		Self {
			ticker: 0,
			_marker: std::marker::PhantomData,
		}
	}
}
impl<C, S> ViewTrait<C, S> for ProblemView<C, S>
where
	C: Ctx,
{
	fn draw(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, C, S>) {
		ui.vertical_centered(|ui| {
			ui.add_space(16.0);

			ui.heading("Problem View");

			ui.add_space(16.0);

			// ============================================================
			// TICKER
			// ============================================================

			ui.heading(format!("Step {}", self.ticker));

			ui.add_space(12.0);

			ui.horizontal(|ui| {
				// Keep the whole control group centered.
				ui.add_space((ui.available_width() - 420.0).max(0.0) / 2.0);

				if ui
					.add_sized([120.0, 48.0], egui::Button::new("◀ Previous"))
					.clicked()
				{
					// 0 -> 5 -> 4 -> 3...
					self.ticker = (self.ticker + 5) % 6;
				}

				if ui
					.add_sized([100.0, 48.0], egui::Button::new("Reset"))
					.clicked()
				{
					self.ticker = 0;
				}

				if ui
					.add_sized([120.0, 48.0], egui::Button::new("Next ▶"))
					.clicked()
				{
					// 0 -> 1 -> 2 -> ... -> 5 -> 0
					self.ticker = (self.ticker + 1) % 6;
				}
			});

			ui.add_space(20.0);

			// ============================================================
			// TIMELINE
			// ============================================================

			ui.horizontal(|ui| {
				let width = ui.available_width();

				ui.add_space((width - 500.0).max(0.0) / 2.0);

				for i in 0..=5 {
					let active = i == self.ticker;

					let label = if active {
						format!("● {}", i)
					} else {
						format!("○ {}", i)
					};

					let button = egui::Button::new(label);

					if ui.add_sized([60.0, 36.0], button).clicked() {
						self.ticker = i;
					}

					if i < 5 {
						ui.label("──");
					}
				}
			});

			ui.add_space(20.0);

			ui.label(format!("Position: {} / 5", self.ticker));

			ui.add_space(16.0);

			// ============================================================
			// CODE AREA
			// ============================================================

			ui.separator();

			ui.add_space(12.0);

			ui.heading("Code");

			ui.add_space(8.0);

			egui::Frame::group(ui.style()).show(ui, |ui| {
				ui.set_min_height(300.0);
				ui.set_min_width(ui.available_width());

				ui.vertical(|ui| {
					ui.label(
						egui::RichText::new(format!(
							"// Step {}\n\nfn solution() {{\n    // code goes here\n}}",
							self.ticker
						))
						.monospace()
						.size(16.0),
					);
				});
			});

			ui.add_space(20.0);

			// ============================================================
			// API
			// ============================================================

			ui.separator();

			ui.add_space(12.0);

			ui.heading("API");

			ui.add_space(8.0);

			// let api = ctx.app.runtime().services().api();

			ui.horizontal_wrapped(|ui| {
				if ui
					.add_sized([150.0, 40.0], egui::Button::new("Load Problems"))
					.clicked()
				{
					tracing::info!("UI → api.load_problems");
				}

				if ui
					.add_sized([150.0, 40.0], egui::Button::new("Load Problem"))
					.clicked()
				{
					tracing::info!("UI → api.load_problem");
				}

				if ui
					.add_sized([150.0, 40.0], egui::Button::new("Sample Problem"))
					.clicked()
				{
					tracing::info!("UI → api.sample_problem");
				}
			});

			// Prevent unused-variable warning until the API calls are wired.
			// let _ = api;
		});
	}
	fn update(&mut self, _ctx: &mut AppContext<'_, C, S>) {}
	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, C, S>) {}
}

impl<C, S> ProblemView<C, S> {
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
impl<C, S> ProblemView<C, S>
where
	C: Ctx,
{
	fn draw<E: Executor>(&mut self, ui: &mut Ui, ctx: &mut AppContext<'_, C, S>) {
		ui.heading("Problem View");

		ui.horizontal(|ui| {
			ui.label(format!("Ticker: {}", self.ticker));

			if ui.button("−").clicked() {
				self.ticker = self.ticker.saturating_sub(1);
			}

			if ui.button("Reset").clicked() {
				self.ticker = 0;
			}

			if ui.button("+").clicked() {
				self.ticker = (self.ticker + 1).min(5);
			}
		});

		ui.add_space(10.0);

		ui.horizontal(|ui| {
			for i in 0..=5 {
				if i > 0 {
					ui.label("──");
				}

				let label = if i == self.ticker {
					format!("● {}", i)
				} else {
					format!("○ {}", i)
				};

				if ui.button(label).clicked() {
					self.ticker = i;
				}
			}
		});

		ui.label(format!("Position: {} / 5", self.ticker));

		ui.separator();

		// Direct service access, as you intended.
		// let api = ctx.app.runtime().services().api();

		ui.horizontal_wrapped(|ui| {
			if ui.button("load_problems").clicked() {
				tracing::info!("UI → api.load_problems");
				// ctx.app.executor.spawn();
				// api.load_problems(...)
			}

			if ui.button("load_problem").clicked() {
				tracing::info!("UI → api.load_problem");
				// api.load_problem(...)
			}

			if ui.button("sample_problem").clicked() {
				tracing::info!("UI → api.sample_problem");
				// api.sample_problem(...)
			}
		});
	}
}
// impl<C> ProblemView<C> {
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
