use crate::{model::problem::StoredProblem, prelude::*, proto::types::SampleProblemRequest};

// # Shared application state
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

// # Shared API Trait
// ============================================================
//
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait Api: Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait Api: Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

impl Clone for Box<dyn Api> {
	fn clone(&self) -> Self {
		self.clone_box()
	}
}

// # Native
// ============================================================
//
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use crate::proto::{
	problem_service_client::ProblemServiceClient, submission_service_client::SubmissionServiceClient,
};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct NativeApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl NativeApiClient {
	pub fn new(
		problems: ProblemServiceClient<Channel>,
		submissions: SubmissionServiceClient<Channel>,
	) -> Self {
		Self {
			problems,
			submissions,
		}
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl NativeApiClient {
	pub async fn connect() -> anyhow::Result<Self> {
		let chan = Channel::from_static(crate::GRPC_SOCKET_CLIENT)
			.connect()
			.await?;

		Ok(Self {
			problems: ProblemServiceClient::new(chan.clone()),
			submissions: SubmissionServiceClient::new(chan),
		})
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl Api for NativeApiClient {
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}

	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!("NativeApiClient load_problems")
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!("NativeApiClient load_problem")
	}

	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		println!("Native API Client sample_problem");
		let request: crate::proto::types::SampleProblemRequest = request.into();
		let response = self
			.problems
			.clone()
			.sample_problem(request)
			.await?
			.into_inner();

		StoredProblem::try_from(response)
	}
}

// # WASM
// ============================================================
// Native Build needs client too.
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

#[cfg(target_arch = "wasm32")]
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
