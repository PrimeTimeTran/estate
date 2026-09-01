use crate::{
	app::{prelude::*, state::EstateState},
	e,
	model::problem::StoredProblem,
	prelude::*,
	proto::leetcode::types::{ListProblemsRequest, PageRequest},
};

pub struct App<R: Runtime> {
	pub(crate) engine: EstateEngine<R>,
	pub(crate) view: ViewType,
	pub(crate) api: Arc<ApiClient>,
	pub(crate) state: AppState,
}
impl<R: Runtime> App<R> {
	pub fn new(engine: EstateEngine<R>, api: Arc<ApiClient>) -> Self {
		Self {
			api,
			engine,
			state: AppState::default(),
			view: crate::START_VIEW,
		}
	}
	pub fn initialize(&mut self, api: Arc<ApiClient>) {
		self.api = api;
		// self.handle = handle;
	}
	pub fn runtime(&self) -> Arc<R> {
		Arc::clone(&self.engine.runtime)
	}
	pub fn set_api(&mut self, api: Arc<ApiClient>) {
		self.api = api;
	}
	pub fn api(&self) -> Arc<ApiClient> {
		Arc::clone(&self.api)
	}
	pub fn update(&mut self) {
		while let Some(event) = self.engine.runtime.try_recv() {
			tracing::info!("update;");

			match event.kind {
				e::EventKind::Navigate(view) => {
					self.view = view;
				}
				e::EventKind::ProblemsLoaded(problems) => {
					self.state.problems = problems;
					self.state.problems_loading = false;
					self.state.problems_error = None;
				}
				e::EventKind::ApiError(error) => {
					self.state.problems_loading = false;
					self.state.problems_error = Some(error);
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
impl<R: Runtime + 'static> App<R> {
	pub fn load_problems(&mut self) {
		if self.state.problems_loading {
			return;
		}

		self.state.problems_loading = true;
		self.state.problems_error = None;

		let api = Arc::clone(&self.api);
		let events = self.engine.runtime.clone();

		self.engine.runtime.spawn(async move {
			let mut client = api.problems.clone();

			let result = client
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
				.await;
			match result {
				Ok(response) => {
					let problems = response
						.into_inner()
						.problems
						.into_iter()
						.map(StoredProblem::from)
						.collect();
					tracing::info!("ProblemsLoaded");
					events.emit(e::Event::app(e::EventKind::ProblemsLoaded(problems)));
				}
				Err(error) => {
					events.emit(e::Event::app(e::EventKind::ApiError(error.to_string())));
				}
			}
		});
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
			.emit(e::Event::app(e::EventKind::SessionStart));
	}
	pub fn new_task(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			}));
	}
	pub fn clear_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	pub fn stop_session(&mut self) {
		println!("stop_session from app");
	}
}
impl<R: Runtime> App<R> {
	pub fn app_state(&self) -> &AppState {
		&self.state
	}
	// pub fn get_view(&self, view_type: ViewType) -> Ve<R> {
	// 	match view_type {
	// 		ViewType::MarkdownView => Ve::new(MarkdownView::new(crate::MARKDOWN)),
	// 		_ => self.default_view(),
	// 	}
	// }
	pub fn default_view(&self) -> Ve<R> {
		Ve::new(MarkdownView::new(crate::MARKDOWN))
	}
	pub fn view(&self) -> ViewType {
		self.view
	}
	pub fn show_view(&mut self, view: ViewType) {
		self.view = view;
	}
	pub fn show_dashboard(&mut self) {
		self.show_view(ViewType::Dashboard);
	}
	pub fn show_tasks(&mut self) {
		self.show_view(ViewType::TaskManager);
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_list".into(),
			}));
	}
}
