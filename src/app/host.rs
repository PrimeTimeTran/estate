use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalHost {
	VsCode,
	Zed,
	JetBrains,
	Unknown,
}

pub fn terminal_host() -> TerminalHost {
	let env = std::env::vars().collect::<std::collections::HashMap<_, _>>();
	if env.contains_key("VSCODE_INJECTION")
		|| env.contains_key("VSCODE_PID")
		|| env.contains_key("VSCODE_IPC_HOOK_CLI")
	{
		return TerminalHost::VsCode;
	}

	// Zed sets ZED_TERM in terminals launched from Zed.
	if env.contains_key("ZED_TERM") {
		return TerminalHost::Zed;
	}

	// JetBrains terminals commonly expose TERMINAL_EMULATOR.
	if env
		.get("TERMINAL_EMULATOR")
		.is_some_and(|v| v.contains("JetBrains"))
	{
		return TerminalHost::JetBrains;
	}

	TerminalHost::Unknown
}

pub enum CargoFeature {
	Native,
	Web,
	None,
}

fn append_to_file(str: String) {}
fn sleep(str: String) {}
fn task() {
	println!(
		"The task asts the time that's used by the clock. {}",
		time_now()
	)
}
fn time_now() -> String {
	use chrono::Utc;
	let now = Utc::now();
	let format = "%B %-d, %Y at %-I:%M:%S %p UTC";
	now.format(format).to_string()
}

impl AppCtx for WebContext {
	type State = WebState;

	fn state(&self) -> &Self::State {
		&self.state()
	}
}

impl Clock for HostClock {
	type Handle<C: AppCtx> = WorkHandle<C>;

	fn now(&self) -> String {
		time_now()
	}
	fn run_once(&self) -> String {
		let now = self.now();
		tracing::info!("HostClock Tick: {}", now);
		now
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn run_foreground(&self, interval: Duration) {
		loop {
			self.run_once();
			std::thread::sleep(interval);
		}
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn run_background<C: AppCtx>(&self, interval: Duration, msg: String) -> WorkHandle<C> {
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();
		let clock = self.clone();

		let join = self.handle.spawn(async move {
			loop {
				if task_cancel.is_cancelled() {
					break;
				}
				let now = clock.run_once();
				tracing::info!("{msg}: {now}");
				tokio::select! {
					_ = task_cancel.cancelled() => break,
					_ = tokio::time::sleep(interval) => {}
				}
			}
		});

		WorkHandle::new(cancel, join)
	}
	#[cfg(target_arch = "wasm32")]
	fn run_background<C: AppCtx>(&self, interval: Duration, msg: String) -> WorkHandle<C> {
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();
		let clock = self.clone();

		wasm_bindgen_futures::spawn_local(async move {
			loop {
				if task_cancel.is_cancelled() {
					break;
				}

				let now = clock.run_once();
				tracing::info!("{msg}: {now}");

				gloo_timers::future::TimeoutFuture::new(interval.as_millis() as u32).await;
			}
		});

		WorkHandle::new(cancel)
	}
}

impl<C: AppCtx> Host<C> {
	pub fn new(context: C) -> anyhow::Result<Self> {
		#[cfg(not(target_arch = "wasm32"))]
		let runtime = tokio::runtime::Runtime::new()?;

		#[cfg(not(target_arch = "wasm32"))]
		let handle = runtime.handle().clone();

		Ok(Self {
			context,
			worker: HostWorker::new(),
			#[cfg(target_arch = "wasm32")]
			clock: HostClock::default,
			#[cfg(not(target_arch = "wasm32"))]
			clock: HostClock::new(handle),
			#[cfg(not(target_arch = "wasm32"))]
			runtime,
		})
	}
}

impl<C> Host<C>
where
	C: AppCtx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}
	pub fn context(&self) -> &C {
		&self.context
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn handle(&self) -> tokio::runtime::Handle {
		self.runtime.handle().clone()
	}
	/// Host-level execution goes here.
	/// This does NOT need to be winit.
	/// CLI, daemon, or GUI can build on top of this.
	pub fn run(&self) -> Result<()> {
		tracing::info!("Host run");
		// self.worker().spawn_ctrl_c();
		// #[cfg(not(target_arch = "wasm32"))]
		// self.worker.block_on(async {
		// 	tokio::signal::ctrl_c()
		// 		.await
		// 		.expect("failed to listen for Ctrl+C");
		// 	tracing::info!("Ctrl+C received");
		// });
		// #[cfg(all(target_arch = "wasm32"))]
		// {
		// 	// WASM has no process-level Ctrl+C signal.
		// 	// Shutdown must be triggered externally.
		// }
		Ok(())
	}
	pub fn worker(&self) -> &HostWorker<C> {
		&self.worker
	}
	pub fn shutdown(self) {
		#[cfg(all(not(target_arch = "wasm32")))]
		self.runtime.shutdown_background();
	}
}
#[cfg(feature = "native")]
impl Host<NativeContext> {
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
		let context = NativeContext::default();
		Self::new(context)
	}
}
#[cfg(feature = "web")]
impl Host<WebContext> {
	pub fn init() -> Result<Self> {
		Self::new(WebContext::default())
	}
}
#[cfg(not(target_arch = "wasm32"))]
impl HostClock {
	pub fn new(handle: tokio::runtime::Handle) -> Self {
		Self { handle }
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl<C> HostWorker<C>
where
	C: AppCtx,
{
	pub fn new() -> Self {
		let runtime = tokio::runtime::Runtime::new().unwrap();
		Self {
			runtime: Arc::new(runtime),
			_phantom: PhantomData,
		}
	}
}
#[cfg(target_arch = "wasm32")]
impl<C> HostWorker<C>
where
	C: AppCtx,
{
	pub fn new() -> Self {
		Self {
			_phantom: PhantomData,
		}
	}
}
impl<C> HostWorker<C>
where
	C: AppCtx,
{
	#[cfg(not(feature = "web"))]
	pub fn wait_for_ctrl_c(&self) {
		let (tx, rx) = std::sync::mpsc::channel();

		self.spawn_ctrl_c(tx);

		let _ = rx.recv();
	}
	#[cfg(all(not(feature = "web")))]
	pub fn spawn_ctrl_c<S>(&self, sink: S)
	where
		S: EventSink<AppEvent>,
	{
		self.runtime.block_on(async {
			#[cfg(not(target_arch = "wasm32"))]
			{
				if tokio::signal::ctrl_c().await.is_ok() {
					sink.send(AppEvent::Shutdown);
				}
			}
			#[cfg(target_arch = "wasm32")]
			{
				// WASM has no process-level Ctrl+C signal.
				// Shutdown must be triggered externally.
			}
		});
	}
	#[cfg(all(not(feature = "web")))]
	pub fn block_on<F>(&self, future: F)
	where
		F: Future,
	{
		self.runtime.block_on(future);
	}
}
impl<C> Provide<C> for Host<C>
where
	C: AppCtx,
{
	type Clock = HostClock;
	type Worker = HostWorker<C>;
	// type Renderer = HostRenderer;
	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
	fn worker(&self) -> &Self::Worker {
		&self.worker
	}
	// fn renderer(&mut self) -> &mut Self::Renderer {
	// 	&mut self.renderer
	// }
}
#[cfg(not(target_arch = "wasm32"))]
impl<C> Worker<C> for HostWorker<C>
where
	C: AppCtx,
{
	type Handle = WorkHandle<C>;

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
	}

	fn run_background<F, Fut>(&self, task: F) -> WorkHandle<C>
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

	#[cfg(not(target_arch = "wasm32"))]
	fn run_background_blocking<F>(&self, task: F) -> WorkHandle<C>
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

	#[cfg(not(target_arch = "wasm32"))]
	fn spawn<F, Fut>(&self, task: F) -> WorkHandle<C>
	where
		F: FnOnce() -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();
		let join = self.runtime.spawn(async move {
			// If you want cancellation to actually matter,
			// the future itself needs to observe task_cancel.
			task().await;
		});

		WorkHandle::new(cancel, join)
	}
}
#[cfg(target_arch = "wasm32")]
impl<C> Worker<C> for HostWorker<C>
where
	C: AppCtx,
{
	type Handle = WorkHandle<C>;

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + 'static,
		Fut: Future<Output = ()> + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		wasm_bindgen_futures::spawn_local(async move {
			task(task_cancel).await;
		});

		WorkHandle::new(cancel)
	}

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + 'static,
	{
		task();
	}
}

#[derive(Debug, Clone)]
pub struct Connected {
	#[cfg(feature = "web")]
	pub api: WebApiClient,
	#[cfg(feature = "native")]
	pub api: NativeApiClient,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Disconnected;

pub struct Host<C>
where
	C: AppCtx,
{
	pub context: C,
	clock: HostClock,
	worker: HostWorker<C>,

	#[cfg(not(target_arch = "wasm32"))]
	pub runtime: tokio::runtime::Runtime,
}

#[cfg(all(target_arch = "wasm32"))]
#[derive(Clone, Default)]
pub struct HostClock;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct HostClock {
	handle: tokio::runtime::Handle,
}
pub struct HostRenderer;

pub struct HostWorker<C: AppCtx> {
	_phantom: PhantomData<C>,
	#[cfg(not(target_arch = "wasm32"))]
	runtime: Arc<tokio::runtime::Runtime>,
}
#[derive(Debug, Default)]
pub struct WebState;
#[derive(Default)]
pub struct WebContext;
