use crate::prelude::*;

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
	type GuiState = WebState;
}

pub struct App<C: Ctx> {
	pub host: Host<C>,
	pub state: C::AppState,
	pub workers: Vec<WorkHandle<C, tokio::task::JoinHandle<()>>>,
}

#[derive(Clone, Default)]
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

	async fn sample_problem(&self, _request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		todo!("ApiClient sample_problem")
	}

	async fn load_problem(&self, _id: i64) -> anyhow::Result<StoredProblem> {
		todo!("ApiClient load_problem")
	}
}

impl App<Context> {}
