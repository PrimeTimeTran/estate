use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum ApiConnection {
	Disconnected,
	Connecting,
	Connected(ApiClient),
}

impl Default for ApiConnection {
	fn default() -> Self {
		Self::Disconnected
	}
}

impl ApiService {
	pub fn new(client: ApiClient) -> Self {
		Self {
			connection: ApiConnection::Connected(client),
		}
	}
	pub fn connection_state(&self) -> &'static str {
		match &self.connection {
			ApiConnection::Disconnected => "Disconnected",
			ApiConnection::Connecting => "Connecting",
			ApiConnection::Connected(_) => "Connected",
		}
	}
}
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl ApiService {
	pub async fn connect(&mut self) -> anyhow::Result<()> {
		self.connection = ApiConnection::Connecting;
		let client = ApiClient::connect().await?;
		self.connection = ApiConnection::Connected(client);
		Ok(())
	}
}
#[cfg(all(feature = "web", target_arch = "wasm32"))]
impl ApiService {
	pub fn connect(&mut self, base_url: impl Into<String>) {
		self.connection = ApiConnection::Connecting;
		let client = ApiClient::new(base_url);
		self.connection = ApiConnection::Connected(client);
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl Api for ApiService {
	async fn load_problems(&self, query: ProblemQuery) -> anyhow::Result<Vec<StoredProblem>> {
		match &self.connection {
			ApiConnection::Connected(client) => client.load_problems(query).await,
			ApiConnection::Disconnected => anyhow::bail!("API is disconnected"),
			ApiConnection::Connecting => anyhow::bail!("API is connecting"),
		}
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		match &self.connection {
			ApiConnection::Connected(client) => client.load_problem(id).await,
			ApiConnection::Disconnected => anyhow::bail!("API is disconnected"),
			ApiConnection::Connecting => anyhow::bail!("API is connecting"),
		}
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		match &self.connection {
			ApiConnection::Connected(client) => client.sample_problem(request).await,
			ApiConnection::Disconnected => anyhow::bail!("API is disconnected"),
			ApiConnection::Connecting => anyhow::bail!("API is connecting"),
		}
	}

	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[async_trait::async_trait(?Send)]
impl Api for ApiService {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		match &self.connection {
			ApiConnection::Connected(client) => client.load_problems().await,
			ApiConnection::Disconnected => {
				anyhow::bail!("API is disconnected")
			}
			ApiConnection::Connecting => {
				anyhow::bail!("API is connecting")
			}
		}
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		match &self.connection {
			ApiConnection::Connected(client) => client.load_problem(id).await,
			ApiConnection::Disconnected => {
				anyhow::bail!("API is disconnected")
			}
			ApiConnection::Connecting => {
				anyhow::bail!("API is connecting")
			}
		}
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		match &self.connection {
			ApiConnection::Connected(client) => client.sample_problem(request).await,
			ApiConnection::Disconnected => {
				anyhow::bail!("API is disconnected")
			}
			ApiConnection::Connecting => {
				anyhow::bail!("API is connecting")
			}
		}
	}

	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}
}

impl<T> Page<T> {
	pub fn page_info(&self) -> PageInfo {
		PageInfo {
			page: self.page as i32,
			page_size: self.page_size as i32,
			total: self.total as i64,
		}
	}
}

#[derive(Debug)]
pub struct JsonRepo<T> {
	#[cfg(not(feature = "web"))]
	pub path: PathBuf,
	pub _marker: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub struct Page<T> {
	pub items: Vec<T>,
	pub page: u32,
	pub page_size: u32,
	pub total: u64,
}

#[derive(Clone, Debug)]
pub struct SessionService {
	pub state_service: Arc<StateService>,
}

#[derive(Debug)]
pub struct StateService {
	pub repo: JsonRepo<EstateState>,
}

#[derive(Debug, Default, Clone)]
pub struct ApiService {
	pub connection: ApiConnection,
}
