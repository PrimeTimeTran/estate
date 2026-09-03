use crate::{
	Executor,
	api::{Api, AppState},
	app::{prelude::*, state::EstateState},
	e,
	model::StoredProblem,
	prelude::*,
	proto::types::{ListProblemsRequest, PageRequest, SampleProblemRequest},
	r#trait::EventReceiver,
	ui::Layout,
};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use crate::logger::{LogConfig, Tracer};

pub struct App {
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	native: NativeApp,
}

impl App {
	pub fn new() -> Result<Self> {
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			return Ok(Self {
				native: NativeApp::new()?,
			});
		}
		#[cfg(any(not(feature = "native"), target_arch = "wasm32"))]
		{
			Ok(Self {})
		}
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub fn run(mut self, cli: Cli) -> Result<()> {
		self.native.run(cli)
	}
}

#[derive(Debug, Clone)]
pub struct AppRuntime<R: Runtime> {
	pub(crate) engine: EstateEngine<R>,
	pub(crate) view: ViewType,
	pub(crate) api: Arc<dyn Api>,
	pub(crate) state: AppState,
	// pub(crate) executor: dyn Executor,
	events: R::EventReceiver,
}
impl<R: Runtime> AppRuntime<R> {
	pub fn new(engine: EstateEngine<R>, api: Arc<dyn Api>) -> Self {
		let events = engine.runtime.subscribe();
		Self {
			api,
			engine,
			events,
			// executor,
			state: AppState::default(),
			view: crate::START_VIEW,
		}
	}
	pub fn start_services(&self) {
		println!("AppRuntime start_services");
		self
			.engine
			.runtime
			.emit(e::Event::app(e::Klass::SessionStart {}));
		tracing::info!("AppRuntime start_services");
	}
}
impl<R: Runtime> AppRuntime<R> {
	pub fn runtime(&self) -> Arc<R> {
		Arc::clone(&self.engine.runtime)
	}
	pub fn api(&self) -> Arc<dyn Api> {
		Arc::clone(&self.api)
	}
	/// # UI/application state
	/// "We have two event consumers, and we need to decide which events belong to which execution domain."
	// # Pull Based
	pub fn update(&mut self) {
		// Event routing
		// UI update cycle
		//      │
		//      ▼
		// events.try_recv()
		//      │
		//      ▼
		// match
		while let Some(event) = self.events.try_recv() {
			match event.kind {
				e::Klass::Navigate(view) => {
					tracing::debug!("♻️ App<T> new view from app update {:?}", view.name());
					self.view = view;
				}
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
}
impl<R: Runtime> AppRuntime<R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.engine.runtime.state().read()
	}
	pub fn app_state(&self) -> &AppState {
		&self.state
	}
	pub fn jobs(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.state()
	}
}
impl<R: Runtime> AppRuntime<R> {
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
		// self.session_service.end().await.unwrap_or_else(|e| {
		// 	tracing::error!("Error occurred while ending session: {}", e);
		// });
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
	pub fn view(&self) -> ViewType {
		self.view
	}
	pub fn show_view(&mut self, view: ViewType) {
		self.view = view;
	}
}
impl<R: Runtime + 'static> AppRuntime<R> {
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
impl<R: Runtime + 'static> AppRuntime<R> {
	pub fn start(&self) {
		if !crate::START_APP_CLOCK {
			return;
		}
		let views = [
			ViewType::ProblemScreen,
			ViewType::DashboardScreen,
			ViewType::MarkdownView,
			ViewType::ProblemScreen,
			ViewType::WaterfallScreen,
			ViewType::ProblemScreen,
			ViewType::TaskManagerScreen,
			ViewType::ProblemsScreen,
		];
		let mut view_idx = 0;
		let mut current_time = 5;
		let runtime = self.engine.runtime.clone();
		// self.executor.spawn(async move {
		// 	loop {
		// 		runtime.sleep(std::time::Duration::from_secs(1)).await;
		// 		if current_time == 0 {
		// 			current_time = 5;
		// 			view_idx = (view_idx + 1) % views.len();
		// 			let view = views[view_idx];
		// 			// println!("🧭 Clock navigation → {:?}", view);
		// 			runtime.emit(e::Event::app(e::Klass::Navigate(view)));
		// 		} else {
		// 			current_time -= 1;
		// 			println!("⏰ App<T> CLOCK TICK: {}", current_time);
		// 		}
		// 	}
		// });
	}
	pub fn sample_problem(&mut self) {
		if !self.start_problems_request() {
			return;
		}

		let api = Arc::clone(&self.api);
		let runtime = self.engine.runtime.clone();
		// self.executor.spawn(async move {
		// 	match api
		// 		.sample_problem(SampleProblemRequest {
		// 			page: None,
		// 			difficulty: None,
		// 			tags: vec![],
		// 			search: String::new(),
		// 			published_only: None,
		// 		})
		// 		.await
		// 	{
		// 		Ok(problem) => {
		// 			runtime.emit(e::Event::app(e::Klass::ProblemSampled(problem)));
		// 		}
		// 		Err(error) => {
		// 			runtime.emit(e::Event::app(e::Klass::ProblemSampleFailed(
		// 				error.to_string(),
		// 			)));
		// 		}
		// 	}
		// });
	}
	pub fn load_problems(&mut self) {
		if !self.start_problems_request() {
			return;
		}
		let api = Arc::clone(&self.api);
		let runtime = self.engine.runtime.clone();
		// self.executor.spawn(async move {
		// 	match api.load_problems().await {
		// 		Ok(problems) => {
		// 			runtime.emit(e::Event::app(e::Klass::ProblemsLoaded(problems)));
		// 		}
		// 		Err(error) => {
		// 			runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
		// 		}
		// 	}
		// });
	}
}
