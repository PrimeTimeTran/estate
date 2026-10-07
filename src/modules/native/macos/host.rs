use crate::{IpcServer, prelude::*};

impl<C> App<C>
where
	C: Ctx,
{
	pub fn init_daemon(&mut self) -> Result<()> {
		tracing::info!("init_daemon");
		Ok(())
	}
}

impl<C: Ctx> Host<C> {
	pub fn start(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		self.start_ipc()?;
		self.start_watchers()?;
		Ok(self.start_hid_bridge()?)
	}
	pub fn start_ipc(&self) -> Result<()> {
		let ipc = IpcServer::new(PathBuf::from(ESTATE_SOCKET), self.event_bus.clone());
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
		tracing::info!("start_hid_bridge");

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
		tracing::info!("macos host new");
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
