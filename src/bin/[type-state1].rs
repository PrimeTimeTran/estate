#![allow(warnings)]
use estate::app::*;

pub trait Ctx {
	type Executor: Executor;
	type Host: Host;
	type Runtime: Runtime;
}
pub trait Host {}

fn main() {}

impl<C> AppRuntime<C>
where
	C: Ctx,
{
	pub fn new(context: C) -> Self {
		// Works
		// context::AppContext::load_problems(&mut self);
		// context::AppContext::load_problem(&mut self);
		// self.context

		// let events = context.runtime().subscribe();

		// Self {
		// 	context,
		// 	events,
		// 	state: AppState::default(),
		// 	view: crate::START_VIEW,
		// }
		todo!("")
	}

	pub fn executor(&self) -> &C::Executor {
		// self.context.executor()
		todo!("")
	}

	pub fn host(&self) -> &C::Host {
		// self.context.host()
		todo!("")
	}
}

impl<C> AppRuntime<C>
where
	C: Ctx,
{
	pub fn start_services(&self) -> Result<()> {
		Ok(())
	}
}
impl<C> AppRuntime<C>
where
	C: Ctx + 'static,
{
	pub fn start(&self) {
		if !START_APP_CLOCK {
			return;
		}
		// let runtime = self.context.runtime().clone();
		// let task_runtime = runtime.clone();
		// self.context.runtime().spawn(async move {
		// 	let mut view_idx = 0;
		// 	let mut current_time = 5;
		// 	loop {
		// 		task_runtime.sleep(Duration::from_secs(1)).await;
		// 		if current_time == 0 {
		// 			current_time = 5;
		// 			view_idx = (view_idx + 1) % TICK_ITEMS_LENGTH;
		// 			task_runtime.emit(e::Event::app(e::Klass::Navigate(TICK_ITEMS[view_idx])));
		// 		} else {
		// 			current_time -= 1;
		// 			println!("⏰ AppRuntime CLOCK TICK: {current_time}");
		// 		}
		// 	}
		// });
		todo!("")
	}

	pub fn api(&self) -> Option<&<<<C as Ctx>::Runtime as Runtime>::Services as Services>::Client> {
		// self.context.runtime().services().api().as_ref()
		todo!("")
	}

	pub fn sample_problem(&mut self) {
		if !self.start_problems_request() {
			return;
		}
		// let runtime = self.context.runtime().clone();
		// let task_runtime = runtime.clone();
		// self.executor.spawn(async move {
		// 	let query = SampleProblemRequest {
		// 		page: None,
		// 		difficulty: None,
		// 		tags: vec![],
		// 		search: String::new(),
		// 		published_only: None,
		// 	};
		// 	let problems = runtime.services().api().sample_problem(query).await;
		// 	// let problems = runtime.services()
		// 	println!("App Runtime After problems click {:?}", problems);
		// 	match problems {
		// 		Ok(problems) => {
		// 			println!("App Runtime emitting success");
		// 			runtime.emit(e::Event::app(e::Klass::ProblemsLoaded(vec![problems])));
		// 		}
		// 		Err(error) => {
		// 			runtime.emit(e::Event::app(e::Klass::ApiError(error.to_string())));
		// 		}
		// 	}
		// });
	}
	pub fn load_problems(&mut self) {
		if !self.start_problems_request() {
			return;
		}
		// let runtime = self.context.runtime().clone();
		todo!("")
	}
}

impl<C> AppRuntime<C>
where
	C: Ctx + 'static,
{
	/// Runtime Reference
	///
	/// "Borrow a reference to the runtime."
	pub fn runtime(&self) -> &C::Runtime {
		// self.context.runtime()
		todo!("")
	}
	/// Outer App<C>
	///
	/// "Here's a thread-safe shared handle to the runtime. You can keep this."
	pub fn runtime_handle(&self) -> Arc<C> {
		// Arc::clone(&self.context.runtime())
		todo!("")
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
impl<C> AppRuntime<C>
where
	C: Ctx + 'static,
{
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, C> {
		// self.context.runtime().state().read()
		todo!("")
	}
	pub fn app_state(&self) -> &AppState {
		&self.state
	}
	pub fn jobs(&self) -> std::sync::RwLockReadGuard<'_, C> {
		self.state()
	}
}
impl<C> AppRuntime<C>
where
	C: Ctx + 'static,
{
	pub fn new_task(&mut self) {
		// self
		// 	.context
		// .runtime()
		// .emit(e::Event::app(e::Klass::TaskRequested {
		// 	request: TaskRequest::Create(TaskKind::SyncBookmarks),
		// }));
		todo!("")
	}
	pub fn clear_tasks(&mut self) {
		// self
		// 	.context
		// .runtime()
		// .emit(e::Event::app(e::Klass::CommandExecuted {
		// 	command: "task_clear".into(),
		// }));
		todo!("")
	}
	pub fn stop_session(&mut self) {
		// self.session_service.end().await.unwrap_or_else(|e| {
		// 	tracing::error!("Error occurred while ending session: {}");
		// });
	}
	pub fn show_tasks(&mut self) {
		self.show_view(ViewType::TaskManagerScreen);
		// self
		// 	.context
		// .runtime()
		// .emit(e::create::app(e::Klass::CommandExecuted {
		// 	command: "task_list".into(),
		// }));
		todo!("")
	}
	pub fn view(&self) -> ViewType {
		self.view
	}
	pub fn show_view(&mut self, view: ViewType) {
		self.view = view;
	}
}

impl<C> AppRuntime<C>
where
	C: Ctx,
{
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

impl<C> Ctx for NativeApp<C>
where
	C: Ctx,
{
	type Runtime = NativeRuntime;
	type Executor = NativeExecutor;
	type Host = NativeHost;
}

impl Ctx for Tester {
	// type Args = Cli;
	type Runtime = NativeRuntime;
	type Executor = NativeExecutor;
	type Host = NativeHost;
	// fn runtime(&self) -> &Self::Runtime {
	// 	&self.runtime
	// }
	// fn new() -> Result<Self> {
	// 	Tester::new()
	// }
}

impl<C> Debug for AppRuntime<C>
where
	C: Ctx,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("AppRuntime")
			.field("view", &self.view)
			.field("state", &self.state)
			.field("events", &"<EventReceiver>")
			.finish()
	}
}

impl<C> Drop for AppRuntime<C>
where
	C: Ctx,
{
	fn drop(&mut self) {
		tracing::info!("💀 AppRuntime Drop");
	}
}
impl Host for NativeHost {}
impl Tester {
	pub fn new() {
		todo!("")
	}
	pub fn runtime() {
		todo!("")
	}
}

impl Tester {
	// type Services = NativeServices;
	// type EventReceiver = NativeEventReceiver;
	fn event_processed(&self) {
		todo!("")
	}
	fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		todo!("")
	}
	fn state(&self) -> &RuntimeState {
		todo!("")
	}
	fn start_dispatcher(self: &Arc<Self>) {
		todo!("")
	}
	fn services(&self) -> &NativeServices {
		todo!("")
	}
	fn session_service(&self) -> &Arc<SessionService> {
		todo!("")
	}
	fn state_service(&self) -> &Arc<StateService> {
		todo!("")
	}
	fn tasks(&self) -> &Arc<RwLock<TaskManager>> {
		todo!("")
	}
	async fn sleep(&self, _time: Duration) {
		todo!("")
	}
	fn session(&self) -> Session {
		todo!("")
	}
	fn try_recv(&self) -> Option<e::Event> {
		todo!("")
	}
	fn save(&self, _state: &EstateState) -> Result<()> {
		todo!("")
	}
	fn subscribe(&self) -> NativeEventReceiver {
		todo!("")
	}
	fn emit(&self, event: estate::Event) {
		todo!("")
	}
}

/// ## [AppRuntime]
/// Wraps concrete Web & Native to expose runtime implementation to shared capabilities
/// on both platforms more easily making the architecture more robust to changes.
///
// #[derive(Default, Clone)]
pub struct AppRuntime<C>
where
	C: Ctx,
{
	pub context: C,
	pub events: <C::Runtime as Runtime>::EventReceiver,
	pub state: AppState,
	pub view: ViewType,
}
pub struct NativeApp<C>
where
	C: Ctx,
{
	pub app: AppRuntime<C>,
}

pub struct NativeHost;

#[derive(Clone)]
pub struct Tester {}
