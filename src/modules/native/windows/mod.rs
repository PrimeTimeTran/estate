use crate::prelude::{logger, *};
use anyhow::{Context as AnyhowCtx, Result};

pub mod hdi;
pub use hdi::*;

pub mod focus;
pub use focus::*;

impl<C: Ctx> Host<C> {
	pub fn new(context: Arc<C>, tokio: tokio::runtime::Runtime) -> anyhow::Result<Self> {
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
impl Host<Context> {
	pub fn init() -> anyhow::Result<Self> {
		let parsed = cli::context::parse();

		let mut config = LogConfig::load().context("LogConfig::load failed")?;

		config.apply_cli(&parsed);

		logger::init_logging(&config).context("logger::init_logging failed")?;

		let tokio = tokio::runtime::Runtime::new().context("tokio runtime creation failed")?;

		let mut context = Context::default();

		#[cfg(not(feature = "daemon"))]
		{
			tokio
				.block_on(context.api_mut().connect())
				.context("API connect failed")?;
		}

		let context = Arc::new(context);
		Self::new(context, tokio)
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
		if self.mode == AppMode::Daemon {
			// self.init_daemon();
		}
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

		// #[cfg(not(feature = "daemon"))]
		{
			let mut renderer = structs::Renderer::<Context, <Context as Ctx>::AppState>::new(
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

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
}
