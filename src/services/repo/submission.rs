use crate::{prelude::*, services::*};

#[async_trait]
pub trait SubmissionRepository {
	async fn list(&self, query: SubmissionQuery) -> anyhow::Result<Page<Submission>>;
	async fn get(&self, id: &str) -> anyhow::Result<Submission>;
	async fn create(&self, submission: CreateSubmission) -> anyhow::Result<Submission>;
	async fn update(&self, id: &str, submission: UpdateSubmission) -> anyhow::Result<Submission>;
	async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
