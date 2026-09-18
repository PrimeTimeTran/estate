use crate::{
	prelude::*,
	proto::{
		problem_service_client::ProblemServiceClient,
		submission_service_client::SubmissionServiceClient,
	},
};


#[async_trait::async_trait]
pub trait Api: Debug + 'static {
	async fn load_problems(&self, query: ProblemQuery) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

#[async_trait::async_trait]
impl Api for ApiClient {
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}
	async fn load_problems(&self, query: ProblemQuery) -> anyhow::Result<Vec<StoredProblem>> {
		let request: crate::proto::types::ListProblemsRequest = query.try_into()?;

		tracing::info!(?request, "Sending ListProblemsRequest");

		let response = self
			.problems
			.clone()
			.list_problems(request)
			.await?
			.into_inner();

		tracing::info!(
			returned = response.problems.len(),
			?response,
			"Received ListProblemsResponse"
		);

		let problems = response
			.problems
			.into_iter()
			.map(StoredProblem::try_from)
			.collect::<Result<Vec<_>, _>>()?;

		tracing::info!(count = problems.len(), "Decoded problems");

		Ok(problems)
	}

	async fn load_problem(&self, _id: i64) -> anyhow::Result<StoredProblem> {
		todo!("load_problem");
		// StoredProblem::try_from(response)
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		let request: crate::proto::types::SampleProblemRequest = request.into();
		let response = self
			.problems
			.clone()
			.sample_problem(request)
			.await?
			.into_inner();
		StoredProblem::try_from(response)
	}
}

impl ApiClient {
	pub async fn connect() -> anyhow::Result<Self> {
		let endpoint = crate::GRPC_SOCKET_CLIENT;

		tracing::info!(endpoint, "Connecting to gRPC server");

		let chan = Channel::from_static(endpoint)
			.connect()
			.await
			.map_err(|error| {
				anyhow::anyhow!("failed to connect to gRPC endpoint {endpoint}: {error:#}")
			})?;

		tracing::info!("gRPC channel connected");

		Ok(Self {
			problems: ProblemServiceClient::new(chan.clone()),
			submissions: SubmissionServiceClient::new(chan),
		})
	}
	pub fn new(
		problems: ProblemServiceClient<Channel>,
		submissions: SubmissionServiceClient<Channel>,
	) -> Self {
		Self {
			problems,
			submissions,
		}
	}
}

impl<C> App<C>
where
	C: Ctx,
{
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::debug!("New App Native Context");
		let state = C::initial_state();
		let (cursor_event_tx, cursor_events) = std::sync::mpsc::channel();
		return Ok(Self {
			cursor_event_tx,
			cursor_events,
			host,
			state,
			workers: vec![],
		});
	}
}

impl App<Context> {
	pub fn api(&self) -> &ApiService {
		self.host.api()
	}
	pub fn run(&mut self) -> Result<()> {
		tracing::debug!("App run");
		self.init_services()?;
		self.run_gui()?;
		Ok(())
	}
	pub fn run_gui(&mut self) -> Result<()> {
		let cancel = CancellationToken::new();

		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.build()
			.expect("failed to build GUI event loop");
		let proxy = event_loop.create_proxy();
		let handle = self.start_app_events(proxy.clone())?;
		self.workers.push(handle);

		let event_rx = self.host.event_bus.subscribe_broadcast("app");
		let event_tx = self.host.event_bus.sender();
		self.host.runtime.attach_event_proxy(proxy);
		let mut renderer = Renderer::<Context, <Context as Ctx>::AppState>::new(
			self.host.context(),
			self.state.clone(),
			cancel,
			event_rx,
			event_tx,
		);

		event_loop
			.run_app(&mut renderer)
			.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"))?;

		Ok(())
	}
}

impl<C> App<C>
where
	C: Ctx,
{
	pub fn start_app_events(
		&mut self,
		proxy: EventLoopProxy<AppEvent>,
	) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let handle = self.host.worker().run_background(move |cancel| async move {
			tracing::debug!("🔥 APP EVENTS TASK STARTED");

			let mut view_idx = 0;
			let mut current_time = 3;

			loop {
				tokio::select! {
					_ = cancel.cancelled() => {
						tracing::debug!("🔥 APP EVENTS CANCELLED");
						break;
					}

					_ = tokio::time::sleep(Duration::from_secs(1)) => {
						if current_time == 0 {
							current_time = 3;
							view_idx = (view_idx + 1) % TICK_ITEMS_LENGTH;

							let view = TICK_ITEMS[view_idx];

							tracing::debug!(
								"🔥 APP EVENTS NAVIGATING TO {:?}",
								view,
							);

							match proxy.send_event(AppEvent::Navigate(view)) {
								Ok(()) => {
									tracing::debug!("🔥 Navigate SENT");
								}

								Err(err) => {
									tracing::error!(?err, "🔥 Navigate FAILED");
								}
							}
						} else {
							current_time -= 1;

							tracing::debug!(
								"⏰ APP EVENTS CLOCK TICK: {current_time}",
							);
						}
					}
				}
			}
		});

		Ok(handle)
	}
	pub fn start_cargo_watcher(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		tracing::debug!("cargo: entered");
		let watcher = CargoWatcher::new().map_err(|error| {
			tracing::error!("Failed to create Cargo watcher: {error}");
			error
		})?;
		tracing::debug!("cargo: watcher created");
		tracing::debug!("Watching Cargo.toml: {}", watcher.path().display());
		let worker = self.worker();
		tracing::debug!("cargo: watcher started");
		Ok(worker.run_background_blocking(move |cancel| {
			let (tx, rx) = std::sync::mpsc::channel();
			let mut fs_watcher = match RecommendedWatcher::new(tx, Config::default()) {
				Ok(watcher) => watcher,
				Err(error) => {
					tracing::error!("Failed to create Cargo watcher: {error}");
					return;
				}
			};
			if let Err(error) = fs_watcher.watch(watcher.path(), RecursiveMode::NonRecursive) {
				tracing::error!("Failed to watch Cargo.toml: {error}");
				return;
			}
			loop {
				if cancel.is_cancelled() {
					break;
				}

				match rx.recv_timeout(std::time::Duration::from_millis(100)) {
					Ok(Ok(event)) => {
						if matches!(
							event.kind,
							notify::EventKind::Modify(_) | notify::EventKind::Create(_)
						) {
							tracing::debug!("Cargo.toml changed");

							if let Err(error) = watcher.run_once_sync() {
								tracing::error!("Failed to process Cargo.toml: {error}");
							}
						}
					}

					Ok(Err(error)) => {
						tracing::error!("Cargo watcher error: {error}");
					}

					Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
						// Allows us to check cancellation.
					}

					Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
						break;
					}
				}
			}
			tracing::debug!("Cargo watcher stopped");
		}))
	}

	/// ## `App::start_cursor_watcher_from_app`
	///
	/// Starts a blocking background worker that runs the native cursor daemon.
	///
	/// Cursor events are forwarded to the app's cursor-event channel through
	/// [AppCursorSink]. The returned [WorkHandle] owns a cancellation token
	/// and the Tokio join handle for the worker.
	///
	/// The cursor daemon is responsible for observing the cancellation token and
	/// returning when cancellation is requested.
	pub fn start_cursor_watcher_from_app(
		&mut self,
	) -> anyhow::Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let sink = AppCursorSink {
			tx: self.cursor_event_tx.clone(),
		};

		Ok(self.worker().run_background_blocking(move |cancel| {
			if let Err(error) = CursorDaemon::new(sink, cancel).run() {
				tracing::error!("Cursor daemon failed: {error}");
			}
		}))
	}
}
impl<NativeCtx, S> ApplicationHandler<AppEvent> for structs::Renderer<NativeCtx, S>
where
	NativeCtx: Ctx + 'static,
	S: Send + Sync + 'static,
{
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		tracing::debug!("about_to_wait");
		// self.app.update();
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			tracing::info!("MenuEvent::receiver");
			println!("MenuEvent::receiver");
			self.handle_event(event, event_loop);
		}
	}
	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: winit::event::DeviceId,
		_event: winit::event::DeviceEvent,
	) {
		tracing::debug!("device_event");
	}
	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::debug!("exiting")
	}
	fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::debug!("memory_warning")
	}
	fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
		tracing::debug!("new_events")
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		tracing::debug!("🔥 RESUMED");
		if self.windows.is_empty() {
			self.open_window(event_loop, crate::START_WINDOW);
		}
	}
	fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::info!("suspended")
	}
	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
		tracing::debug!("user_event");
		match event {
			AppEvent::Navigate(view) => {
				self.navigate_to(view);
			}

			AppEvent::RuntimeEvent => {
				tracing::info!("user_event RuntimeEvent");
				let _ctx = self.app_context();
				Self::process_events(self);
				self.sync_views();
				// self.process_runtime_events();
				// self.window.request_redraw();
			}
			AppEvent::Shutdown => {
				tracing::debug!(">>> shutdown event received");
				tracing::debug!(">>> event_loop.exit() called");
			}
			AppEvent::ModifiersChanged {
				alt: _,
				command: _,
				ctrl: _,
				shift: _,
			} => {
				tracing::info!("Modifiers Changed")
			}
			_ => {}
		}
	}

	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		tracing::debug!("window_event: {:?}", event);

		let Some(window) = self
			.windows
			.iter_mut()
			.find(|window| window.window.instance.id() == window_id)
		else {
			return;
		};

		let response = window
			.window
			.gui_state
			.on_window_event(&window.window.instance, &event);

		if response.repaint {
			window.window.instance.request_redraw();
		}

		match event {
			WindowEvent::Resized(size) => {
				window.window.resize(size);
				window.window.instance.request_redraw();
			}

			WindowEvent::RedrawRequested => {
				if window.window.occluded {
					return;
				}

				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					input: IOState::default(),
					last_revision: 0,
				};

				if let Err(e) = window.window.draw(&mut ctx) {
					tracing::error!("DEV >>> draw failed: {e:#}");
				}
			}

			_ => {}
		}
	}
}

impl Context {
	fn new(state: NativeState, api: ApiService) -> Self {
		Self { state, api }
	}
}
impl Default for Context {
	fn default() -> Self {
		Self::new(NativeState::default(), ApiService::default())
	}
}

impl Ctx for Context {
	fn api(&self) -> &Self::Api {
		&self.api
	}
	fn api_mut(&mut self) -> &mut Self::Api {
		&mut self.api
	}
	fn initial_state() -> Self::AppState {
		structs::S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
		}
	}
	type Api = ApiService;
	type AppState = structs::S<Context>;
	type EventReceiver = structs::BroadcastReceiver<e::Event>;
	type EventSender = structs::BroadcastSender<e::Event>;
	type GuiState = NativeGuiState;
}

impl<Context> Host<Context>
where
	Context: Ctx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}

	pub fn context(&self) -> Arc<Context> {
		self.context.clone()
	}

	pub fn handle(&self) -> tokio::runtime::Handle {
		self.tokio.handle().clone()
	}

	pub fn shutdown(self) {
		self.tokio.shutdown_background();
	}

	pub fn wait_for_shutdown(&self) {
		self.worker.block_on(async {
			tokio::signal::ctrl_c()
				.await
				.expect("failed to listen for Ctrl+C");
			tracing::info!("Ctrl+C received");
		});
	}

	pub fn worker(&self) -> &HostWorker<Context> {
		&self.worker
	}
	pub fn subscribe(&self) -> structs::BroadcastReceiver<e::Event> {
		self.event_bus.subscribe_broadcast("host")
	}
}

impl Host<Context> {
	pub fn init() -> anyhow::Result<Self> {
		let parsed = cli::context::parse();

		let mut config = LogConfig::load()?;
		config.apply_cli(&parsed);
		logger::init_logging(&config)?;

		// Create the one runtime.
		let tokio = tokio::runtime::Runtime::new()?;

		// Context is still uniquely owned here.
		let mut context = Context::default();

		// Connect using the same runtime that Host will retain.
		tokio.block_on(context.api_mut().connect())?;

		// Only share Context after initialization.
		let context = Arc::new(context);

		Self::new(context, tokio)
	}
	fn logging() {
		// let count = 1;
		// let host = "12";
		// let error = EventKind::DaemonStarted;o
		// let state = ViewType::DashboardScreen;
		// awe!("Runtime f");
		// awe!(Info, "Runtime initialized");
		// awe!(Success, "Runtime started");
		// awe!(Warn, "No config found");
		// awe!(Error, "Failed to start runtime");
		// awe!(Info, "Loaded {} count", count);
		// awe!(Success, "Connected to {}", host);
		// awe!(Debug, "State = {:#?}", state);
		// awe!(Debug, "Error = {:#?}", error);
		// let nums = vec![1, 2, 3];
		// let chars = vec!["1", "2", "3"];
		// awe!(Info, "Loaded {:#?} nums", nums);
		// awe!(Info, "Loaded {:#?} chars", chars);
		// panic!("hi");
		// awe!(Debug, "Runtime = {:?}", runtime);
		// crate::app_macros::awe!(Trace, "Dispatching event: {:?}", event);
		// panic!(" Hi ");
	}
}

impl HostClock {
	pub fn new(handle: tokio::runtime::Handle) -> Self {
		Self { handle }
	}
}

impl<NativeCtx, S> structs::Renderer<NativeCtx, S>
where
	NativeCtx: Ctx + 'static,
	S: 'static,
{
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow<NativeCtx, S>> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		tracing::info!(" open window start");
		if self.window_by_type(kind).is_some() {
			return;
		}
		match Window::new(event_loop, self.view) {
			Ok(window) => {
				tracing::info!(" open window end, new window");
				window.instance.set_title(self.view.name().into());
				self.windows.push(AppWindow {
					// runtime: self.runtime.clone(),
					kind,
					view: self.view,
					window,
				});
			}
			Err(error) => {
				tracing::error!("failed to create window: {error}");
			}
		}
	}
	fn handle_event(&mut self, _event: MenuEvent, _event_loop: &ActiveEventLoop) {
		tracing::info!("handle_event");

		// match event {
		// 	MenuEvent::Navigate(view) => {
		// 		// self.navigate_to(view);
		// 	} // Other menu events...
		// 	  // MenuEvent::OpenWindow(kind) => {
		// 	  //   self.open_window(event_loop, kind);
		// 	  // }
		// }
	}
}
impl<NativeCtx, S> structs::Renderer<NativeCtx, S>
where
	NativeCtx: Ctx + 'static,
	S: 'static,
{
	// fn sync_views(&mut self) {
	// 	for window in &mut self.windows {
	// 		window.view = self.view;
	// 		window.window.sync_view(window.view);
	// 		window.window.instance.set_title(self.view.name());
	// 		window.window.instance.request_redraw();
	// 	}
	// }
	// fn navigate_to(&mut self, view: ViewType) {
	// 	tracing::debug!("navigating from {:?} to {:?}", self.view, view,);
	// 	self.view = view;
	// 	self.sync_views();
	// }
}

impl<C> Worker<C> for HostWorker<C>
where
	C: Ctx,
{
	type Handle = WorkHandle<C, tokio::task::JoinHandle<()>>;

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
	}

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		let join = self.runtime.spawn(async move {
			task(task_cancel).await;
		});

		WorkHandle::new(cancel, join)
	}

	/// Runs a blocking task on Tokio's blocking thread pool.
	///
	/// The task receives a [CancellationToken] which is also retained by the
	/// returned [WorkHandle]. Cancelling the handle signals the task to stop;
	/// it does not forcibly terminate the running task.
	///
	/// The returned handle owns the Tokio join handle, allowing the caller to
	/// await the worker's completion during shutdown.
	///
	/// The task must be `'static` because Tokio may outlive the current stack
	/// frame while executing it.
	///
	fn run_background_blocking<F>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		let join = self.runtime.spawn_blocking(move || {
			task(task_cancel);
		});

		WorkHandle::new(cancel, join)
	}

	fn spawn<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce() -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();

		let join = self.runtime.spawn(async move {
			task().await;
		});

		WorkHandle::new(cancel, join)
	}
}

/// ## [App]
///
/// App's root in the native context
///
/// ### Properties
///
/// - [cursor_events][App::cursor_events]: Detect cursor position for teleporting cursor on shift+scroll.
/// - [cursor_event_tx][App::cursor_event_tx]: Send events through this channel.
///
pub struct App<C: Ctx> {
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,
	pub host: Host<C>,
	pub state: C::AppState,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
	// pub menu_bar: Option<MenuBar>,
	// pub tray_clock: Option<MenuBar>,
	// pub tray_cursor: Option<TrayIcon>,
}

#[derive(Debug, Clone)]
pub struct CursorDaemon<S> {
	pub sink: S,
	pub cancel: CancellationToken,
}
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
	pub x: f64,
	pub y: f64,
}

#[derive(Clone, Debug, Default)]
pub struct NativeState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
}
#[derive(Clone, Debug, Default)]
pub struct NativeGuiState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
}
