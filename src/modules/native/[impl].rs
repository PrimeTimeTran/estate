use crate::{
	prelude::*,
	proto::{
		problem_service_client::ProblemServiceClient,
		submission_service_client::SubmissionServiceClient,
	},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
	Gui,
	Tray,
	Daemon,
}

pub fn get_app_mode() -> AppMode {
	#[cfg(feature = "native")]
	let mode = AppMode::Daemon;
	#[cfg(feature = "web")]
	let mode = AppMode::Daemon;
	#[cfg(feature = "daemon")]
	let mode = AppMode::Daemon;
	mode
}

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
		tracing::debug!(?request, "Sending ListProblemsRequest");
		let response = self
			.problems
			.clone()
			.list_problems(request)
			.await?
			.into_inner();
		tracing::debug!(
			returned = response.problems.len(),
			?response,
			"Received ListProblemsResponse"
		);
		let problems = response
			.problems
			.into_iter()
			.map(StoredProblem::try_from)
			.collect::<Result<Vec<_>, _>>()?;
		tracing::debug!(count = problems.len(), "Decoded problems");
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
		let endpoint = GRPC_SOCKET_CLIENT;

		tracing::debug!(endpoint, "Connecting to gRPC server");

		let chan = Channel::from_static(endpoint)
			.connect()
			.await
			.map_err(|error| {
				anyhow::anyhow!("failed to connect to gRPC endpoint {endpoint}: {error:#}")
			})?;

		tracing::debug!("gRPC channel connected");

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
	fn init_settings() -> Result<Settings> {
		tracing::debug!("init_settings");
		let settings = resolver::settings(resolver::source_file(file!()), "settings.json")?;
		println!("{}", serde_json::to_string_pretty(&settings)?);
		Ok(settings)
	}
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::debug!("App::new Native Context");
		let state = C::initial_state();
		let (cursor_event_tx, cursor_events) = std::sync::mpsc::channel();
		let cancel = CancellationToken::new();
		let settings = Self::init_settings()?;
		Ok(Self {
			cancel,
			cursor_event_tx,
			cursor_events,
			host,
			menu_bar: None,
			mode: get_app_mode(),
			settings,
			state,
			tray_clock: None,
			tray_cursor: None,
			workers: vec![],
		})
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
			// tracing::debug!("🔥 APP EVENTS TASK STARTED");
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

									// tracing::debug!(
										// "🔥 APP EVENTS TICK {:?}",
										// view,
									// );

							// match proxy.send_event(AppEvent::Navigate(view)) {
							// 	Ok(()) => {
							// 		tracing::debug!("🔥 Navigate SENT");
							// 	}

							// 	Err(err) => {
							// 		tracing::error!(?err, "🔥 Navigate FAILED");
							// 	}
							// }
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
	/// [CursorSink]. The returned [WorkHandle] owns a cancellation token
	/// and the Tokio join handle for the worker.
	///
	/// The cursor daemon is responsible for observing the cancellation token and
	/// returning when cancellation is requested.
	pub fn start_cursor_watcher_from_app(
		&mut self,
	) -> anyhow::Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let sink = CursorSink {
			tx: self.cursor_event_tx.clone(),
		};
		tracing::debug!("start_cursor_watcher_from_app");
		Ok(self.worker().run_background_blocking(move |cancel| {
			if let Err(error) = CursorDaemon::new(sink, cancel).run() {
				tracing::error!("Cursor daemon failed: {error}");
			}
		}))
	}
}

impl App<Context> {
	pub fn api(&self) -> &ApiService {
		self.host.api()
	}
	// pub fn run(&mut self) -> Result<()> {
	// self.init_services()?;
	// match self.mode {
	// AppMode::Gui => self.run_gui(),
	// AppMode::Tray => self.run_gui(),
	// AppMode::Daemon => self.run_daemon_foreground(),
	// }
	// }
	pub fn run(&mut self) -> Result<()> {
		tracing::info!("app native app run");
		self.init_services()?;
		match self.mode {
			AppMode::Daemon => {
				tracing::info!("Starting daemon mode");
				let hid = self.host.start()?;
				self.workers.push(hid);
				self.init_daemon();
				self.host.worker.wait_for_ctrl_c();
			}
			_ => {
				self.run_gui()?;
			}
		}

		Ok(())
	}
	fn run_daemon_foreground(&mut self) -> Result<()> {
		tracing::debug!("run_daemon");
		let hid = self.host.start()?;
		self.workers.push(hid);
		self.init_daemon();
		self
			.worker()
			.block_on(async { tokio::signal::ctrl_c().await });
		Ok(())
	}

	fn run_daemon_2(&mut self) -> Result<()> {
		tracing::debug!("starting daemon");
		self.init_services()?;
		let hid = self.host.start()?;
		self.workers.push(hid);
		self.init_daemon();
		// Stay alive until supervisor terminates us
		self.host.worker.wait_for_ctrl_c();
		Ok(())
	}

	pub fn run_daemon(&mut self) -> Result<()> {
		let hid = self.host.start()?;
		self.workers.push(hid);
		self.init_daemon();
		// self.host.worker.wait_for_shutdown();
		// macos_create_bg_daemon();
		Ok(())
	}

	async fn daemon_loop(&mut self) -> Result<()> {
		self.host.start()?;
		self.init_daemon();
		tokio::signal::ctrl_c().await?;
		Ok(())
	}
	pub fn run_gui(&mut self) -> Result<()> {
		tracing::info!("app native app run_gui");
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
		tracing::info!("app native ");
		// #[cfg(not(feature = "daemon"))]
		{
			let mut renderer = Renderer::<Context, <Context as Ctx>::AppState>::new(
				self.host.context(),
				self.state.clone(),
				Arc::new(self.settings.clone()),
				cancel,
				event_rx,
				event_tx,
			);
			event_loop
				.run_app(&mut renderer)
				.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"))?;
		}
		Ok(())
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
	type EventReceiver = s::BroadcastReceiver<e::Event>;
	type EventSender = s::BroadcastSender<e::Event>;
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
	pub fn subscribe(&self) -> structs::BroadcastReceiver<e::Event> {
		self.event_bus.subscribe_broadcast("host")
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
}

// impl Host<Context> {
// 	pub fn init() -> anyhow::Result<Self> {
// 		let parsed = cli::context::parse();
//
// 		let mut config = LogConfig::load()?;
// 		config.apply_cli(&parsed);
// 		logger::init_logging(&config)?;
//
// 		// Create the one runtime.
// 		let tokio = tokio::runtime::Runtime::new()?;
//
// 		// Context is still uniquely owned here.
// 		let mut context = Context::default();
//
// 		// Daemon may not need this
// 		// This requires server access
// #[cfg(not(feature = "daemon"))]
// 		{
// 			// Connect using the same runtime that Host will retain.
// 			tokio.block_on(context.api_mut().connect())?;
// 		}
//
// 		// Only share Context after initialization.
// 		let context = Arc::new(context);
// 		Self::new(context, tokio)
// 	}
// 	fn logging() {
// 		// let count = 1;
// 		// let host = "12";
// 		// let error = EventKind::DaemonStarted;o
// 		// let state = ViewType::DashboardScreen;
// 		// awe!("Runtime f");
// 		// awe!(Info, "Runtime initialized");
// 		// awe!(Success, "Runtime started");
// 		// awe!(Warn, "No config found");
// 		// awe!(Error, "Failed to start runtime");
// 		// awe!(Info, "Loaded {} count", count);
// 		// awe!(Success, "Connected to {}", host);
// 		// awe!(Debug, "State = {:#?}", state);
// 		// awe!(Debug, "Error = {:#?}", error);
// 		// let nums = vec![1, 2, 3];
// 		// let chars = vec!["1", "2", "3"];
// 		// awe!(Info, "Loaded {:#?} nums", nums);
// 		// awe!(Info, "Loaded {:#?} chars", chars);
// 		// panic!("hi");
// 		// awe!(Debug, "Runtime = {:?}", runtime);
// 		// crate::macros::awe!(Trace, "Dispatching event: {:?}", event);
// 		// panic!(" Hi ");
// 	}
// }

impl HostClock {
	pub fn new(handle: tokio::runtime::Handle) -> Self {
		Self { handle }
	}
}

impl<C> HostWorker<C>
where
	C: Ctx,
{
	pub fn start_grpc_server(&self) -> WorkHandle<C, tokio::task::JoinHandle<()>> {
		self.run_background(|cancel| async move {
			if let Err(error) = crate::modules::grpc::run_server(cancel).await {
				tracing::error!(
					%error,
					"❌ Estate gRPC server stopped"
				);
			}
		})
	}
}
impl<C> r#trait::Worker<C> for HostWorker<C>
where
	C: Ctx,
{
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

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
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

	type Handle = WorkHandle<C, tokio::task::JoinHandle<()>>;
}

#[derive(Debug, Clone)]
pub struct ApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
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
	pub mode: AppMode,
	pub settings: Settings,
	pub cancel: CancellationToken,
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	pub tray_cursor: Option<TrayIcon>,
}

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
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
pub struct NativeGuiState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
}
#[derive(Clone, Debug, Default)]
pub struct NativeState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
}

fn macos_create_bg_daemon() {
	// nohup /Users/future/kb/project/target/debug/daemon \
	// >/tmp/estate-daemon.out \
	// 2>/tmp/estate-daemon.err &

	//
	// future in project (main●)
	// $ DAEMON_PID=$!
	// echo "PID=$DAEMON_PID"
	// PID=16229
	//
	// future in project (main●)
	// $ ps -p "$DAEMON_PID" -o pid,ppid,state,command
	// PID  PPID STAT COMMAND
	// 16229  6760 SN   /Users/future/kb/project/target/debug/daemon
	//

	// sleep 3
	// ps -p "$DAEMON_PID" -o pid,ppid,state,command

	// kill it
	// ps aux | grep '[d]aemon'

	// - Same PID. Still alive
	// nohup "$(pwd)/target/debug/daemon" \
	// >/tmp/estate-daemon.out \
	// 2>/tmp/estate-daemon.err &
	//
	// DAEMON_PID=$!
	//
	// echo "Estate PID: $DAEMON_PID"
	// ps -p "$DAEMON_PID" -o pid,ppid,state,command

	// sleep 2
	// ps -p "$DAEMON_PID" -o pid,ppid,state,command

	// - Errors
	// cat /tmp/estate-daemon.err
	// cat /tmp/estate-daemon.out

	// ~/Library/LaunchAgents/com.estate.daemon.plist
	// - Load It
	// launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.estate.daemon.plist
	// - Load It
	// launchctl print gui/$(id -u)/com.estate.daemon
	// $ pgrep -af estate

	// $ cat /tmp/estate-daemon.err
	// $ cat /tmp/estate-daemon.out

	// $ pgrep -af estate

	// launchctl bootout gui/$(id -u)/com.estate.daemon
	// launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.estate.daemon.plist

	// <?xml version="1.0" encoding="UTF-8"?>
	// <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
	// <plist version="1.0">
	// <dict>
	// <key>Label</key>
	// <string>com.estate.daemon</string>
	//
	// <key>ProgramArguments</key>
	// <array>
	// <string>/Users/future/kb/project/target/debug/estate</string>
	// <string>--daemon</string>
	// </array>
	//
	// <key>RunAtLoad</key>
	// <true />
	//
	// <key>KeepAlive</key>
	// <true />
	//
	// <key>StandardOutPath</key>
	// <string>/tmp/estate-daemon.out</string>
	//
	// <key>StandardErrorPath</key>
	// <string>/tmp/estate-daemon.err</string>
	// </dict>
	// </plist>
	//

	// CLI Commands
	//
	// future in estate (main●)
	// $ launchctl bootout gui/$(id -u)/com.estate.daemon
	//
	//
	// future in estate (main●)
	// $ launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.estate.daemon.plist
	//
	// future in estate (main●)
	// $ launchctl print gui/$(id -u)/com.estate.daemon | grep -E 'state|active count|pid|last exit'
	// gui/501/com.estate.daemon = {
	// active count = 0
	// path = /Users/future/Library/LaunchAgents/com.estate.daemon.plist
	// state = spawn scheduled
	// stdout path = /tmp/estate-daemon.out
	// stderr path = /tmp/estate-daemon.err
	// XPC_SERVICE_NAME => com.estate.daemon
	// last exit code = 2
	// state = active
	// active count = 1
	// name = com.estate.daemon
	// state = active
	// active count = 1
	// name = com.estate.daemon
	// job state = exited
	//
	// pgrep -af daemon
	// nohup /Users/future/kb/project/target/debug/daemon \\n\t>/tmp/estate-daemon.out \\n\t2>/tmp/estate-daemon.err &
	// cat /tmp/estate-daemon.out\ncat /tmp/estate-daemon.err
	// nohup /Users/future/kb/project/target/debug/daemon \\n\t>/tmp/estate-daemon.out \\n\t2>/tmp/estate-daemon.err &
	// ps aux | grep -E '[e]state|[o]s-observer'
	// pkill -f '^/tmp/estate-os-observer$'
	// c
	// ps aux | grep -E '[e]state|[o]s-observer'
	// nohup /Users/future/kb/project/target/debug/daemon \\n\t>/tmp/estate-daemon.out \\n\t2>/tmp/estate-daemon.err &
	// ps aux | grep -E '[e]state|[o]s-observer'
	// nohup /Users/future/kb/project/target/debug/daemon \\n  >/tmp/estate-daemon.out \\n  2>/tmp/estate-daemon.err &\n\nDAEMON_PID=$!\necho "DAEMON PID=$DAEMON_PID"
	// pgrep -af '/target/debug/daemon'\npgrep -af estate-os-observer
	// ps -o pid,ppid,state,command -p "$DAEMON_PID"
	// OBSERVER_PID=$(pgrep -f '^/tmp/estate-os-observer$' | head -1)\nps -o pid,ppid,state,command -p "$OBSERVER_PID"
	// ps -o pid,ppid,state,command -p 32903
	// ps -o pid,ppid,state,command -p 32903,32995
	// ps -axo pid,ppid,state,lstart,command | grep '[e]state-os-observer'
	// ps -o pid,ppid,state,lstart,command -p 32903,34320
	// ps -o pid,ppid,state,lstart,command -p 6760,34320
	// ps -axo pid,ppid,tty,state,command | grep -E '[d]aemon|[e]state-os-observer'
}
