use crate::{
	doc,
	prelude::*,
	ui::{self, prelude as gui},
};

#[async_trait::async_trait(?Send)]
pub trait Api: Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}
impl<C> App<C>
where
	C: Ctx,
{
	pub fn new(host: Host<C>) -> Result<Self> {
		tracing::debug!("New App Web Context");
		let state = C::initial_state();
		{
			return Ok(Self {
				state,
				host,
				workers: vec![],
			});
		}
	}
}

impl App<Context> {
	pub fn run(&mut self) -> Result<()> {
		tracing::debug!("App run");
		self.init_services()?;
		Ok(())
	}
}

impl Ctx for Context {
	fn api(&self) -> &Self::Api {
		&self.api
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
	type GuiState = WebState;
	type EventReceiver = structs::BroadcastReceiver<e::Event>;
}

impl Host<Context> {
	pub fn init() -> Result<Self> {
		let api = ApiService::default();
		Self::new(Arc::new(Context::default()), api)
	}
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
	pub fn worker(&self) -> &HostWorker<Context> {
		&self.worker
	}
}
impl<C> Worker<C> for HostWorker<C>
where
	C: Ctx,
{
	type Handle = WorkHandle<C, tokio::task::JoinHandle<()>>;

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

pub struct App<C: Ctx> {
	pub host: Host<C>,
	pub state: C::AppState,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
}

#[derive(Default)]
pub struct Context {
	pub api: ApiService,
}

#[derive(Clone, Debug, Default)]
pub struct WebState;

/// ## WASM
///
/// Native Build needs client too.
#[derive(Debug, Clone)]
pub struct ApiClient {
	base_url: String,
}

impl ApiClient {
	pub fn new(base_url: impl Into<String>) -> Self {
		Self {
			base_url: base_url.into(),
		}
	}
}

#[async_trait::async_trait(?Send)]
impl Api for ApiClient {
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}

	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!("ApiClient load_problems")
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		todo!("ApiClient sample_problem")
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!("ApiClient load_problem")
	}
}

impl App<Context> {}
