use crate::prelude::*;

pub enum CargoFeature {
	Native,
	None,
	Web,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalHost {
	JetBrains,
	Unknown,
	VsCode,
	Zed,
}

fn append_to_file(str: String) {}
fn terminal_host() -> TerminalHost {
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
	type Handle<C: Ctx, J> = WorkHandle<C, J>;

	// #[cfg(not(target_arch = "wasm32"))]
	// type WorkHandle<C: Ctx> = WorkHandle<C, tokio::task::JoinHandle<()>>;

	// #[cfg(target_arch = "wasm32")]
	// type Handle<C: Ctx> = WorkHandle<C, ()>;

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

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn run_background<C, J>(&self, interval: Duration, msg: String) -> WorkHandle<C, J>
	where
		C: Ctx,
		J: From<tokio::task::JoinHandle<()>>,
	{
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

		WorkHandle::new(cancel, J::from(join))
	}

	#[cfg(all(feature = "web", target_arch = "wasm32"))]
	fn run_background<C, J>(&self, interval: Duration, msg: String) -> WorkHandle<C, J>
	where
		C: Ctx,
	{
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

impl<C: Ctx> Host<C> {
	#[cfg(feature = "web")]
	pub fn new(context: Arc<C>) -> anyhow::Result<Self> {
		Ok(Self {
			context,
			worker: HostWorker::new(),
			clock: HostClock::default(),
		})
	}

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub fn new(context: Arc<C>) -> anyhow::Result<Self> {
		let runtime = tokio::runtime::Runtime::new()?;
		let handle = runtime.handle().clone();
		Ok(Self {
			context,
			worker: HostWorker::new(),
			clock: HostClock::new(handle),
			runtime,
		})
	}
}

impl<C> HostWorker<C>
where
	C: Ctx,
{
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub fn new() -> Self {
		let runtime = tokio::runtime::Runtime::new().unwrap();
		Self {
			runtime: Arc::new(runtime),
			_phantom: PhantomData,
		}
	}
	#[cfg(target_arch = "wasm32")]
	pub fn new() -> Self {
		Self {
			_phantom: PhantomData,
		}
	}
}

impl<C> HostWorker<C>
where
	C: Ctx,
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
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
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
	C: Ctx,
{
	type Clock = HostClock;
	type Worker = HostWorker<C>;

	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
	fn worker(&self) -> &HostWorker<C> {
		&self.worker
	}
}

#[derive(Debug, Clone)]
pub struct Connected {
	#[cfg(all(feature = "native"))]
	pub api: NativeApiClient,
	#[cfg(all(feature = "web"))]
	pub api: WebApiClient,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Disconnected;

pub struct Host<C>
where
	C: Ctx,
{
	pub context: Arc<C>,
	pub clock: HostClock,
	pub worker: HostWorker<C>,

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub runtime: tokio::runtime::Runtime,
}

pub struct HostRenderer;

pub struct HostWorker<C: Ctx> {
	pub _phantom: PhantomData<C>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub runtime: Arc<tokio::runtime::Runtime>,
}
