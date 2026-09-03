use crate::{
	model::problem::StoredProblem,
	proto::types::{SampleProblemRequest, *},
};

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

// ============================================================
// Shared API Trait
// ============================================================

#[async_trait::async_trait(?Send)]
pub trait Api: std::fmt::Debug + 'static {
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;

	// async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>>;
	// async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem>;
	// async fn sample_problem(&self, query: SampleProblemRequest) -> anyhow::Result<StoredProblem>;
	fn clone_box(&self) -> Box<dyn Api>;
}

impl Clone for Box<dyn Api> {
	fn clone(&self) -> Self {
		self.clone_box()
	}
}
// ============================================================
// Native
// ============================================================

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use crate::proto::{
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
	pub fn new(
		problems: ProblemServiceClient<Channel>,
		submissions: SubmissionServiceClient<Channel>,
	) -> Self {
		// let client = crate::proto::client::ProblemServiceClient::<tonic::transport::Channel>();
		Self {
			problems,
			submissions,
		}
	}
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
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}

	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		todo!("");
		// let mut client = self.problems.clone();
		// let response = client
		// 	.list_problems(ListProblemsRequest {
		// 		tags: vec![],
		// 		search: String::new(),
		// 		published_only: None,
		// 		page: Some(PageRequest {
		// 			page: 0,
		// 			page_size: 100,
		// 		}),
		// 		difficulty: None,
		// 	})
		// 	.await?;

		// response
		// 	.into_inner()
		// 	.problems
		// 	.into_iter()
		// 	.map(StoredProblem::try_from)
		// 	.collect::<Result<Vec<_>, _>>()
		// 	.map_err(Into::into)
	}

	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		todo!()
	}

	async fn sample_problem(&self, query: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
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
	fn clone_box(&self) -> Box<dyn Api> {
		Box::new(self.clone())
	}
	async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
		// fetch(...)
		// deserialize response
		// return Vec<StoredProblem>
		todo!()
	}
	async fn sample_problem(&self, request: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
		// fetch(...)
		todo!()
	}
	async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
		// fetch(...)
		todo!()
	}
	// async fn load_problems(&self) -> anyhow::Result<Vec<StoredProblem>> {
	// 	todo!()
	// }

	// async fn load_problem(&self, id: i64) -> anyhow::Result<StoredProblem> {
	// 	todo!()
	// }

	// async fn sample_problem(&self, query: SampleProblemRequest) -> anyhow::Result<StoredProblem> {
	// 	todo!()
	// }
}
