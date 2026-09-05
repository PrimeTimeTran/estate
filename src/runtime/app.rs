use crate::{app::app_prelude::*, app::app_state::EstateState};

/// ## AppRuntime
///
/// Wraps concrete Web & Native to expose runtime implementation to shared capabilities
/// on both platforms more easily making the architecture more robust to changes.
///
#[derive(Debug, Clone)]
pub struct AppRuntime<R: Runtime, E> {
	pub engine: EstateEngine<R>,
	pub view: ViewType,
	pub state: AppState,
	events: R::EventReceiver,
	pub executor: E,
}
impl<R: Runtime, E> AppRuntime<R, E> {
	pub fn new(engine: EstateEngine<R>, executor: E) -> Self {
		let events = engine.runtime.subscribe();
		// engine.
		Self {
			engine,
			events,
			executor,
			state: AppState::default(),
			view: crate::START_VIEW,
		}
	}
	pub fn start_services(&self) {
		println!("AppRuntime start_services");
		// self
		// 	.engine
		// 	.runtime
		// 	.emit(e::Event::app(e::Klass::SessionStart {}));
		// tracing::info!("AppRuntime start_services");
	}
}
impl<R: Runtime + 'static, E: Executor> AppRuntime<R, E> {
	pub fn start(&self) {
		if !START_APP_CLOCK {
			return;
		}
		let runtime = self.engine.runtime.clone();
		let task_runtime = runtime.clone();
		self.executor.spawn(async move {
			let mut view_idx = 0;
			let mut current_time = 5;
			loop {
				task_runtime.sleep(Duration::from_secs(1)).await;
				if current_time == 0 {
					current_time = 5;
					view_idx = (view_idx + 1) % TICK_ITEMS_LENGTH;
					task_runtime.emit(e::Event::app(e::Klass::Navigate(TICK_ITEMS[view_idx])));
				} else {
					current_time -= 1;
					println!("⏰ AppRuntime CLOCK TICK: {current_time}");
				}
			}
		});
	}
	pub fn sample_problem(&mut self) {
		if !self.start_problems_request() {
			return;
		}
		let runtime = self.engine.runtime.clone();
		// let task_runtime = runtime.clone();
		self.executor.spawn(async move {
			let query = SampleProblemRequest {
				page: None,
				difficulty: None,
				tags: vec![],
				search: String::new(),
				published_only: None,
			};
			let problems = runtime.services().api().sample_problem(query).await;
			println!("App Runtime After problems click {:?}", problems);
			match problems {
				Ok(problems) => {
					println!("App Runtime emiting success");
					runtime.emit(e::Event::app(e::Klass::ProblemsLoaded(vec![problems])));
				}
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
		let runtime = self.engine.runtime.clone();
		self.executor.spawn(async move {
			let problems = runtime.services().api().load_problems().await;
			match problems {
				Ok(problems) => {
					runtime.emit(e::Event::app(e::Klass::ProblemsLoaded(problems)));
				}
				Err(error) => {
					runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
				}
			}
		});
	}
}
impl<R: Runtime, E> AppRuntime<R, E> {
	/// Runtime Reference
	///
	/// "Borrow a reference to the runtime."
	pub fn runtime(&self) -> &R {
		&self.engine.runtime
	}
	/// Outer App<C>
	///
	/// "Here's a thread-safe shared handle to the runtime. You can keep this."
	pub fn runtime_handle(&self) -> Arc<R> {
		Arc::clone(&self.engine.runtime)
	}
	// "Here's my runtime while I'm borrowing this context."
	//
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
					self.state.problem.value = problems.into_iter().next();
					self.state.problem.loading = false;
					self.state.problem.error = None;
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
impl<R: Runtime, E> AppRuntime<R, E> {
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
impl<R: Runtime, E> AppRuntime<R, E> {
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
			.emit(e::create::app(e::Klass::CommandExecuted {
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
impl<R: Runtime + 'static, E: Executor> AppRuntime<R, E> {
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

impl<R: Runtime, E> Drop for AppRuntime<R, E> {
	fn drop(&mut self) {
		tracing::info!("💀 AppRuntime Drop");
	}
}
