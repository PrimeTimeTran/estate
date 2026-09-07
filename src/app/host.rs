use crate::prelude::*;

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

impl Clock for HostClock {
	fn now(&self) -> String {
		#[cfg(not(target_arch = "wasm32"))]
		{
			time_now()
		}

		#[cfg(target_arch = "wasm32")]
		{
			"0".to_string()
		}
	}
	fn run_once(&self) {
		tracing::info!("HostClock Tick: {}", self.now());
	}
	fn run_foreground(&self, interval: Duration) {
		loop {
			self.run_once();
			std::thread::sleep(interval);
		}
	}
	fn run_background(&self, interval: Duration) -> ClockHandle {
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		#[cfg(not(target_arch = "wasm32"))]
		{
			std::thread::spawn(move || {
				let clock = HostClock;

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
			// Spawn async/browser task here later.
			let _ = (interval, task_cancel);
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
	pub fn run(&self) -> Result<()> {
		tracing::info!("Host run");
		// Host-level execution goes here.
		// This does NOT need to be winit.
		// CLI, daemon, or GUI can build on top of this.
		self.worker.block_on(async {
			tokio::signal::ctrl_c()
				.await
				.expect("failed to listen for Ctrl+C");

			tracing::info!("Ctrl+C received");
		});
		Ok(())
	}
	pub fn worker(&self) -> &HostWorker {
		&self.worker
	}

	pub fn new(context: C) -> Result<Self> {
		let parsed = cli::context::parse();
		let mut config = LogConfig::load()?;
		config.apply_cli(&parsed);
		logger::init_logging(&config)?;
		Ok(Self {
			parsed,
			context,
			clock: HostClock,
			worker: HostWorker::new(),
		})
	}
}

#[cfg(feature = "native")]
impl Host<NativeContext> {
	pub fn init() -> Result<Self> {
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
	pub fn new() -> Self {
		let runtime = tokio::runtime::Runtime::new().unwrap();
		Self {
			runtime: Arc::new(runtime),
		}
	}
	/// When there's a proxied event loop, this is how I
	/// listen for ctrl+c exit events.
	pub fn spawn_ctrl_c<S>(&self, sink: S)
	where
		S: EventSink<AppEvent>,
	{
		self.runtime.spawn(async move {
			if tokio::signal::ctrl_c().await.is_ok() {
				sink.send(AppEvent::Shutdown);
			}
		});
	}
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
			task(task_cancel).await;
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

pub struct Host<C>
where
	C: AppCtx,
{
	pub parsed: Cli,
	pub context: C,
	worker: HostWorker,
	clock: HostClock,
}
pub struct HostClock;
pub struct HostRenderer;
pub struct HostWorker {
	pub runtime: Arc<tokio::runtime::Runtime>,
}
pub struct WorkerHandle {
	pub cancel: CancellationToken,
}
