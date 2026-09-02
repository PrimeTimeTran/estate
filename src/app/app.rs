use crate::{
	app::{prelude::*, state::EstateState},
	e,
	model::StoredProblem,
	prelude::*,
	proto::leetcode::types::{ListProblemsRequest, PageRequest, SampleProblemRequest},
	ui::Layout,
};
pub struct App<R: Runtime> {
	pub(crate) engine: EstateEngine<R>,
	pub(crate) view: ViewType,
	pub(crate) api: Arc<ApiClient>,
	pub(crate) state: AppState,
	events: R::EventReceiver,
}
impl<R: Runtime> App<R> {
	pub fn new(engine: EstateEngine<R>, api: Arc<ApiClient>) -> Self {
		let events = engine.runtime.subscribe();
		Self {
			api,
			engine,
			events,
			state: AppState::default(),
			view: crate::START_VIEW,
		}
	}
}
impl<R: Runtime> App<R> {
	pub fn runtime(&self) -> Arc<R> {
		Arc::clone(&self.engine.runtime)
	}
	pub fn start(&self) {
		if !crate::START_APP_CLOCK {
			return;
		}
		let mut count = 0;
		println!("⏰ App<T> START CLOCK: {}", count);
		tracing::info!("⏰ App<T> START CLOCK: {}", count);

		let runtime = self.engine.runtime.clone();

		self.spawn(async move {
			println!("⏰ App<T> CLOCK TASK STARTED: {}", count);
			tracing::info!("⏰ App<T> CLOCK TASK STARTED: {}", count);

			loop {
				count += 1;
				runtime.sleep(std::time::Duration::from_secs(1)).await;
				println!("⏰ App<T> CLOCK TICK: {}", count);
				tracing::info!("⏰ App<T> CLOCK TICK: {}", count);
			}
		});
	}
	pub fn api(&self) -> Arc<ApiClient> {
		Arc::clone(&self.api)
	}
	pub fn update(&mut self) {
		while let Some(event) = self.events.try_recv() {
			match event.kind {
				e::Klass::ProblemsLoaded(problems) => {
					self.state.problems.items = problems;
					self.state.problems.loading = false;
					self.state.problems.error = None;
				}
				e::Klass::ProblemsLoadFailed(error) => {
					self.state.problems.loading = false;
					self.state.problems.error = Some(error);
				}
				e::Klass::ProblemLoaded(problem) | e::Klass::ProblemSampled(problem) => {
					self.state.problem.value = Some(problem);
					self.state.problem.loading = false;
					self.state.problem.error = None;
				}
				e::Klass::ProblemLoadFailed(error) | e::Klass::ProblemSampleFailed(error) => {
					self.state.problem.loading = false;
					self.state.problem.error = Some(error);
				}
				e::Klass::Navigate(view) => {
					self.view = view;
				}
				e::Klass::ApiError(error) => {
					// If you have separate error events for different
					// requests, handle them separately here.
					self.state.problems.loading = false;
					self.state.problems.error = Some(error);
				}

				_ => {}
			}
		}
	}
	pub fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + Send + 'static,
	{
		self.engine.runtime.spawn(future);
	}
}
impl<R: Runtime> App<R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.engine.runtime.state().read()
	}
	pub fn jobs(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.state()
	}
}
impl<R: Runtime> App<R> {
	pub fn on_start(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::Klass::SessionStart));
	}
	pub fn new_task(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::Klass::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			}));
	}
	pub fn clear_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::Klass::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	pub fn stop_session(&mut self) {
		println!("stop_session from app");
	}
}
impl<R: Runtime> App<R> {
	pub fn view(&self) -> ViewType {
		self.view
	}
	pub fn show_view(&mut self, view: ViewType) {
		self.view = view;
	}
}
impl<R: Runtime> App<R> {
	pub fn app_state(&self) -> &AppState {
		&self.state
	}
	pub fn show_dashboard(&mut self) {
		self.show_view(ViewType::DashboardScreen);
	}
	pub fn show_tasks(&mut self) {
		self.show_view(ViewType::TaskManagerScreen);
		self
			.engine
			.runtime
			.emit(e::Event::app(e::Klass::CommandExecuted {
				command: "task_list".into(),
			}));
	}
}

impl<R: Runtime + 'static> App<R> {
	fn spawn_request<F, T, E>(&self, future: F, on_success: impl FnOnce(T) + Send + 'static)
	where
		F: Future<Output = Result<T, E>> + Send + 'static,
		E: std::fmt::Display + Send + 'static,
		T: Send + 'static,
	{
		let runtime = self.engine.runtime.clone();

		self.engine.runtime.spawn(async move {
			match future.await {
				Ok(value) => on_success(value),
				Err(error) => {
					runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
				}
			}
		});
	}
	fn start_problems_request(&mut self) -> bool {
		if self.state.problems.loading {
			tracing::info!("⚠️ problems already loading");
			return false;
		}

		self.state.problems.loading = true;
		self.state.problems.error = None;

		true
	}
	fn start_problem_request(&mut self) -> bool {
		if self.state.problem.loading {
			tracing::info!("⚠️ problem already loading");
			return false;
		}

		self.state.problem.loading = true;
		self.state.problem.error = None;

		true
	}
}

impl<R: Runtime + 'static> App<R> {
	pub fn sample_problem(&mut self) {
		if !self.start_problems_request() {
			return;
		}

		let api = Arc::clone(&self.api);
		let runtime = self.engine.runtime.clone();

		self.engine.runtime.spawn(async move {
			let mut client = api.problems.clone();

			match client
				.sample_problem(SampleProblemRequest {
					page: None,
					difficulty: None,
					tags: vec![],
					search: String::new(),
					published_only: None,
				})
				.await
			{
				Ok(response) => match StoredProblem::try_from(response.into_inner()) {
					Ok(problem) => {
						runtime.emit(e::Event::app(e::Klass::ProblemSampled(problem)));
					}
					Err(error) => {
						runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
					}
				},

				Err(error) => {
					runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
				}
			}
		});
	}
	pub fn load_problems(&mut self) {
		if !self.start_problems_request() {
			return;
		}

		let api = Arc::clone(&self.api);
		let runtime = self.engine.runtime.clone();

		self.engine.runtime.spawn(async move {
			let mut client = api.problems.clone();

			match client
				.list_problems(ListProblemsRequest {
					tags: vec![],
					search: String::new(),
					published_only: None,
					page: Some(PageRequest {
						page: 0,
						page_size: 100,
					}),
					difficulty: None,
				})
				.await
			{
				Ok(response) => {
					let result = response
						.into_inner()
						.problems
						.into_iter()
						.map(StoredProblem::try_from)
						.collect::<Result<Vec<_>, _>>();
					match result {
						Ok(problems) => {
							runtime.emit(e::Event::app(e::Klass::ProblemsLoaded(problems)));
						}
						Err(error) => {
							runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
						}
					}
				}

				Err(error) => {
					runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
				}
			}
		});
	}
}
