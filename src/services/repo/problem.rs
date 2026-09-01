use crate::{prelude::*, services::*};

use leetcode::types::Problem;

#[async_trait]
pub trait ProblemRepository: Send + Sync {
	async fn list(&self, query: ProblemQuery) -> Result<Page<Problem>>;
	async fn create(&self, problem: CreateProblem) -> Result<Problem>;
	async fn update(&self, id: i64, problem: UpdateProblem) -> Result<Problem>;
	async fn delete(&self, id: i64) -> Result<()>;
	async fn get(&self, id: i64) -> Result<Problem>;
	async fn get_by_slug(&self, slug: &str) -> Result<Problem>;
}

pub struct ProblemQuery {
	pub page: Option<i32>,
	pub page_size: Option<i32>,
	pub difficulty: Option<i32>,
}
pub struct CreateProblem {
	pub title: String,
	pub slug: String,
}
pub struct UpdateProblem {
	pub title: Option<String>,
	pub slug: Option<String>,
}
