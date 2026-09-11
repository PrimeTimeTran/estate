use crate::{
	native::native_prelude::*,
	prelude::{traits::Ctx, *},
};

impl Ctx for NativeContext {
	type State = NativeState;

	fn state(&self) -> &Self::State {
		&self.state
	}
}

impl HostClock {
	pub fn new(handle: tokio::runtime::Handle) -> Self {
		Self { handle }
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

impl<NativeContext> Host<NativeContext>
where
	NativeContext: Ctx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}
	pub fn context(&self) -> Arc<NativeContext> {
		self.context.clone()
	}

	pub fn handle(&self) -> tokio::runtime::Handle {
		self.runtime.handle().clone()
	}

	pub fn shutdown(self) {
		self.runtime.shutdown_background();
	}

	pub fn wait_for_shutdown(&self) {
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			self.worker.block_on(async {
				tokio::signal::ctrl_c()
					.await
					.expect("failed to listen for Ctrl+C");

				tracing::info!("Ctrl+C received");
			});
		}
	}

	pub fn worker(&self) -> &HostWorker<NativeContext> {
		&self.worker
	}
}

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
		Self::new(Arc::new(context))
	}
}

#[derive(Clone, Default)]
pub struct NativeState {
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	// pub tray_cursor: Arc<Option<TrayIcon>>,
	// pub windows: Vec<AppWindow>,
}

#[derive(Default)]
pub struct NativeContext {
	pub state: NativeState,
	pub menu_bar: Option<MenuBar>,
	pub tray_clock: Option<MenuBar>,
	pub tray_cursor: Option<TrayIcon>,
	pub windows: Vec<AppWindow>,
}
