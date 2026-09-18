use crate::prelude::*;

#[async_trait]
pub trait ProblemRepository: Send + Sync {
	async fn create(&self, problem: CreateProblem) -> Result<ProtoProblem>;
	async fn delete(&self, id: i64) -> Result<()>;
	async fn get_by_slug(&self, slug: &str) -> Result<ProtoProblem>;
	async fn get(&self, id: i64) -> Result<ProtoProblem>;
	async fn list(&self, query: ProblemQuery) -> Result<Page<ProtoProblem>>;
	async fn sample_problem(&self, query: ProblemQuery) -> Result<ProtoProblem>;
	async fn update(&self, id: i64, problem: UpdateProblem) -> Result<ProtoProblem>;
}

#[async_trait]
pub trait SubmissionRepository {
	async fn create(&self, submission: CreateSubmission) -> anyhow::Result<Submission>;
	async fn delete(&self, id: &str) -> anyhow::Result<()>;
	async fn get(&self, id: &str) -> anyhow::Result<Submission>;
	async fn list(&self, query: SubmissionQuery) -> anyhow::Result<Page<Submission>>;
	async fn update(&self, id: &str, submission: UpdateSubmission) -> anyhow::Result<Submission>;
}
