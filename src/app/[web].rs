use crate::{
	doc,
	prelude::{traits::Ctx, *},
	ui, ui_prelude as gui,
};

#[async_trait::async_trait(?Send)]
pub trait Api: Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

impl Ctx for ContextWeb {
	type AppState = structs::S<ContextWeb>;
	type GuiState = WebState;

	fn initial_state() -> Self::AppState {
		structs::S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
		}
	}
}

impl Host<ContextWeb> {
	// pub fn new(context: Arc<C>) -> anyhow::Result<Self> {
	// 	let clock = HostClock {};
	// 	Ok(Self {
	// 		clock,
	// 		context,
	// 		worker: HostWorker::new(),
	// 	})
	// }
	pub fn init() -> Result<Self> {
		Self::new(Arc::new(ContextWeb::default()))
	}
}

impl<ContextWeb> Host<ContextWeb>
where
	ContextWeb: Ctx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}
	pub fn context(&self) -> Arc<ContextWeb> {
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
	pub fn worker(&self) -> &HostWorker<ContextWeb> {
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

#[derive(Default)]
pub struct ContextWeb;

#[derive(Clone, Debug, Default)]
pub struct WebState;

/// ## WASM
///
/// Native Build needs client too.
#[derive(Debug, Clone)]
pub struct WebApiClient {
	base_url: String,
}

impl WebApiClient {
	pub fn new(base_url: impl Into<String>) -> Self {
		Self {
			base_url: base_url.into(),
		}
	}
}

#[async_trait::async_trait(?Send)]
impl Api for WebApiClient {
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}

	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!("WebApiClient load_problems")
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		todo!("WebApiClient sample_problem")
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!("WebApiClient load_problem")
	}
}

impl App<ContextWeb> {}
