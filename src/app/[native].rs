use crate::{
	prelude::{*},
	proto::{
		problem_service_client::ProblemServiceClient,
		submission_service_client::SubmissionServiceClient,
	},
};

#[async_trait::async_trait]
pub trait Api: Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

#[async_trait::async_trait]
impl Api for NativeApiClient {
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!("NativeApiClient load_problems")
	}
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!("NativeApiClient load_problem")
	}
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		// println!("Native API Client sample_problem");
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
impl<C> App<C>
where
	C: Ctx,
{
	pub fn run_gui(&mut self) -> Result<()>
	where
		C::AppState: Send + Sync + 'static,
	{
		let cancel = CancellationToken::new();
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.build()
			.expect("failed to build GUI event loop");
		let proxy = event_loop.create_proxy();
		let handle = self.start_app_events(proxy.clone())?;
		self.workers.push(handle);
		let mut renderer =
			structs::Renderer::<ContextNative, C::AppState>::new(self.state.clone(), cancel);
		event_loop
			.run_app(&mut renderer)
			.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"))?;
		Ok(())
	}
	pub fn start_app_events(
		&mut self,
		proxy: EventLoopProxy<AppEvent>,
	) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let handle = self.host.worker().run_background(move |cancel| async move {
			tracing::info!("🔥 APP EVENTS TASK STARTED");
			loop {
				tokio::select! {
					_ = cancel.cancelled() => {
						tracing::info!("🔥 APP EVENTS CANCELLED");
						break;
					}
					_ = tokio::time::sleep(Duration::from_secs(5)) => {
						tracing::info!("🔥 APP EVENTS SENDING");
							match proxy.send_event(AppEvent::RuntimeEvent) {
								Ok(()) => {
									tracing::info!("🔥 RuntimeEvent SENT");
								}
								Err(err) => {
									tracing::error!(?err, "🔥 RuntimeEvent FAILED");
								}
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

impl Ctx for ContextNative {
	fn initial_state() -> Self::AppState {
		structs::S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
		}
	}
	type AppState = structs::S<ContextNative>;
	type GuiState = NativeGuiState;
}

impl<ContextNative> Host<ContextNative>
where
	ContextNative: Ctx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}

	pub fn context(&self) -> Arc<ContextNative> {
		self.context.clone()
	}

	pub fn handle(&self) -> tokio::runtime::Handle {
		self.runtime.handle().clone()
	}

	pub fn shutdown(self) {
		self.runtime.shutdown_background();
	}

	pub fn wait_for_shutdown(&self) {
		self.worker.block_on(async {
			tokio::signal::ctrl_c()
				.await
				.expect("failed to listen for Ctrl+C");
			tracing::info!("Ctrl+C received");
		});
	}

	pub fn worker(&self) -> &HostWorker<ContextNative> {
		&self.worker
	}
}

impl Host<ContextNative> {
	// pub fn new(context: Arc<C>) -> anyhow::Result<Self> {
	// 	let runtime = tokio::runtime::Runtime::new()?;
	// 	let handle = runtime.handle().clone();
	// 	Ok(Self {
	// 		context,
	// 		worker: HostWorker::new(),
	// 		clock: HostClock::new(handle),
	// 		runtime,
	// 	})
	// }
	pub fn init() -> Result<Self> {
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
		let parsed = cli::context::parse();
		let mut config = LogConfig::load()?;
		config.apply_cli(&parsed);
		logger::init_logging(&config)?;
		let context = ContextNative::default();
		Self::new(Arc::new(context))
	}
}

impl HostClock {
	pub fn new(handle: tokio::runtime::Handle) -> Self {
		Self { handle }
	}
}

impl NativeApiClient {
	pub async fn connect() -> anyhow::Result<Self> {
		let chan = Channel::from_static(crate::GRPC_SOCKET_CLIENT)
			.connect()
			.await?;

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

pub struct App<C: Ctx> {
	pub cursor_events: std::sync::mpsc::Receiver<CursorEvent>,
	pub cursor_event_tx: std::sync::mpsc::Sender<CursorEvent>,
	pub host: Host<C>,
	pub state: C::AppState,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
}

#[derive(Default)]
pub struct ContextNative {
	pub state: NativeState,
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	pub tray_cursor: Option<TrayIcon>,
	pub windows: Vec<AppWindow>,
}

#[derive(Clone, Debug, Default)]
pub struct NativeState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	// pub tray_cursor: Arc<Option<TrayIcon>>,
	// pub windows: Vec<AppWindow>,
}
#[derive(Clone, Debug, Default)]
pub struct NativeGuiState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
}

#[derive(Debug, Clone)]
pub struct NativeApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
}
