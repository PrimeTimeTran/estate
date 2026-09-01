#![cfg(not(target_arch = "wasm32"))]
use crate::{
	app::{self, App, Runtime, model::EstateEngine},
	e,
	model::problem::StoredProblem,
	prelude::*,
	ui::{MarkdownView, View},
};

use crate::proto::leetcode::{
	problem_service_client::ProblemServiceClient, submission_service_client::SubmissionServiceClient,
};
use tonic::transport::Channel;

#[derive(Debug, Clone)]
pub struct ApiClient {
	pub problems: ProblemServiceClient<Channel>,
	pub submissions: SubmissionServiceClient<Channel>,
}
impl ApiClient {
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

#[derive(Debug, Default)]
pub struct AppState {
	pub problems: Vec<StoredProblem>,
	pub problems_loading: bool,
	pub problems_error: Option<String>,
}
