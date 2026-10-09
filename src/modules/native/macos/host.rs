use super::*;
use crate::{IpcServer, prelude::*};

impl Host<Context> {
	pub fn init() -> anyhow::Result<Self> {
		tracing::info!("app_macos Host init");
		let parsed = cli::context::parse();

		let mut config = LogConfig::load()?;
		config.apply_cli(&parsed);
		logger::init_logging(&config)?;

		// Create the one runtime.
		let tokio = tokio::runtime::Runtime::new()?;

		// Context is still uniquely owned here.
		let mut context = Context::default();

		// Daemon may not need this
		// This requires server access
		#[cfg(not(feature = "daemon"))]
		{
			// Connect using the same runtime that Host will retain.
			tokio.block_on(context.api_mut().connect())?;
		}

		// Only share Context after initialization.
		let context = Arc::new(context);
		Self::new(context, tokio)
	}
	fn logging() {}
}
impl<C: Ctx> Host<C> {
	pub fn start(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		self.start_ipc()?;
		self.start_watchers()?;
		Ok(self.start_hid_bridge()?)
	}
	pub fn start_ipc(&self) -> Result<()> {
		let ipc = IpcServer::new(PathBuf::from(ESTATE_IPC_SOCKET), self.event_bus.clone());
		self.worker.run_background(|_cancel| async move {
			if let Err(error) = ipc.start().await {
				tracing::error!(%error, "Estate IPC server stopped");
			}
		});
		Ok(())
	}
	pub fn start_watchers(&mut self) -> Result<()> {
		// cargo.toml, settings files, caches, index,
		Ok(())
	}
	pub fn start_hid_bridge(&self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		tracing::info!("🕋 MacOS HOST start_hid_bridge");

		let mut hid = MacosHid::new()?;

		hid.start()?;

		let events = self.event_bus.clone();

		let handle = self.worker.run_background(move |cancel| async move {
			if let Err(error) = hid.run(events, cancel).await {
				tracing::error!(%error, "macOS HID stopped");
			}
		});

		Ok(handle)
	}

	pub fn new(context: Arc<C>, tokio: tokio::runtime::Runtime) -> anyhow::Result<Self> {
		tracing::info!("🕋 MacOS Host new");
		// ## TODO:
		//
		// - [ ] Read settings.json using settings_resolver
		// - [ ] Drive behavior using settings

		let handle = tokio.handle().clone();
		let event_bus = EventBus::new();
		let runtime = NativeRuntime::new(Arc::clone(&context), handle.clone(), event_bus.clone())?;
		runtime.start_dispatcher();
		Ok(Self {
			context,
			runtime,
			event_bus,
			worker: HostWorker::new(),
			clock: HostClock::new(handle),
			tokio,
		})
	}
}
