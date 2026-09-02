use crate::model::problem::StoredProblem;

#[async_trait::async_trait(?Send)]
pub trait Api: std::fmt::Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	async fn sample_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
}

// ============================================================
// Native
// ============================================================

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use crate::proto::leetcode::{
	problem_service_client::ProblemServiceClient, submission_service_client::SubmissionServiceClient,
};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use tonic::transport::Channel;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct NativeApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl NativeApiClient {
	pub async fn connect() -> anyhow::Result<Self> {
		let channel = Channel::from_static(crate::GRPC_SOCKET_CLIENT)
			.connect()
			.await?;

		Ok(Self {
			problems: ProblemServiceClient::new(channel.clone()),
			submissions: SubmissionServiceClient::new(channel),
		})
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[async_trait::async_trait(?Send)]
impl Api for NativeApiClient {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		// self.problems.get_problems(...).await
		todo!()
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!()
	}

	async fn sample_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!()
	}
}

// ============================================================
// WASM
// ============================================================

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WasmApiClient {
	base_url: String,
}

#[cfg(target_arch = "wasm32")]
impl WasmApiClient {
	pub fn new(base_url: impl Into<String>) -> Self {
		Self {
			base_url: base_url.into(),
		}
	}
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl Api for WasmApiClient {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!()
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!()
	}

	async fn sample_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!()
	}
}

// ============================================================
// Shared application state
// ============================================================

#[derive(Debug, Default, Clone)]
pub struct AppState {
	pub problems: ProblemListState,
	pub problem: ProblemState,
}

#[derive(Debug, Default, Clone)]
pub struct ProblemListState {
	pub items: Vec<StoredProblem>,
	pub loading: bool,
	pub error: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ProblemState {
	pub value: Option<StoredProblem>,
	pub loading: bool,
	pub error: Option<String>,
}
