use crate::prelude::*;

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
	fn run_background(&self, interval: Duration, msg: String) -> ClockHandle {
		let cancel = CancellationToken::new();
		let clock = HostClock;
		let task_cancel = cancel.clone();

		#[cfg(not(target_arch = "wasm32"))]
		{
			std::thread::spawn(move || {
				loop {
					if task_cancel.is_cancelled() {
						break;
					}

					clock.run_once();
					std::thread::sleep(interval);
				}
			});
		}
		#[cfg(target_arch = "wasm32")]
		{
			wasm_bindgen_futures::spawn_local(async move {
				loop {
					// let time = clock.now();
					// crate::bridge::log(&format!("🔥 impl Clock for HostClock run_background {msg}"));
					// crate::bridge::log(&format!("🔥 {time}"));
					let now = clock.run_once();
					crate::bridge::log(&format!("🔥 {now}"));
					gloo_timers::future::TimeoutFuture::new(1000).await;
				}
			});
		}
		ClockHandle { cancel }
	}
}
impl ClockHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
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
	/// Host-level execution goes here.
	/// This does NOT need to be winit.
	/// CLI, daemon, or GUI can build on top of this.
	pub fn run(&self) -> Result<()> {
		tracing::info!("Host run");
		#[cfg(not(target_arch = "wasm32"))]
		self.worker.block_on(async {
			tokio::signal::ctrl_c()
				.await
				.expect("failed to listen for Ctrl+C");
			tracing::info!("Ctrl+C received");
		});
		#[cfg(all(target_arch = "wasm32"))]
		{
			// WASM has no process-level Ctrl+C signal.
			// Shutdown must be triggered externally.
		}
		Ok(())
	}
	pub fn worker(&self) -> &HostWorker {
		&self.worker
	}

	pub fn new(context: C) -> Result<Self> {
		Ok(Self {
			context,
			clock: HostClock,
			worker: HostWorker::new(),
		})
	}
}

#[cfg(feature = "native")]
impl Host<NativeContext> {
	pub fn init() -> Result<Self> {
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
impl<C> Provide for Host<C>
where
	C: AppCtx,
{
	type Clock = HostClock;
	type Worker = HostWorker;
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
impl HostWorker {
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
#[cfg(not(target_arch = "wasm32"))]
impl Worker for HostWorker {
	type Handle = WorkerHandle;

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
	}
	fn run_background<F, Fut>(&self, task: F) -> WorkerHandle
	where
		F: FnOnce(CancellationToken) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		self.runtime.spawn(async move {
			task(task_cancel).await;
		});

		WorkerHandle { cancel }
	}
	fn run_background_blocking<F>(&self, task: F) -> WorkerHandle
	where
		F: FnOnce(CancellationToken) + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		self.runtime.spawn_blocking(move || {
			task(task_cancel);
		});

		WorkerHandle { cancel }
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn spawn<F, Fut>(&self, task: F)
	where
		F: FnOnce() -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		self.runtime.spawn(task());
	}
}
#[cfg(target_arch = "wasm32")]
impl Worker for HostWorker {
	type Handle = WorkerHandle;

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + 'static,
		Fut: Future<Output = ()> + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		wasm_bindgen_futures::spawn_local(async move {
			gloo_timers::future::TimeoutFuture::new(1000).await;
		});

		WorkerHandle { cancel }
	}

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + 'static,
	{
		task();
	}
}
#[cfg(not(target_arch = "wasm32"))]
impl HostWorker {
	pub fn new() -> Self {
		let runtime = tokio::runtime::Runtime::new().unwrap();

		Self {
			runtime: Arc::new(runtime),
		}
	}
}
#[cfg(target_arch = "wasm32")]
impl HostWorker {
	pub fn new() -> Self {
		Self {
			// initialize browser worker
		}
	}
}

pub struct Host<C>
where
	C: AppCtx,
{
	// pub parsed: Cli,
	pub context: C,
	worker: HostWorker,
	clock: HostClock,
}
pub struct HostClock;
pub struct HostRenderer;

#[cfg(not(target_arch = "wasm32"))]
pub struct HostWorker {
	runtime: Arc<tokio::runtime::Runtime>,
}

#[cfg(target_arch = "wasm32")]
pub struct HostWorker;
pub struct WorkerHandle {
	pub cancel: CancellationToken,
}

#[derive(Debug, Default)]
pub struct WebState;
#[derive(Default)]
pub struct WebContext;

#[derive(Debug, Clone, Copy, Default)]
pub struct Disconnected;

#[derive(Debug, Clone)]
pub struct Connected {
	#[cfg(feature = "web")]
	pub api: WebApiClient,
	#[cfg(feature = "native")]
	pub api: NativeApiClient,
}
