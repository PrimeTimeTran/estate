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
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash)]
pub struct StoredProblem {
	pub id: i64,
	pub number: i32,
	pub title: String,
	pub slug: String,
	pub description: String,
	pub difficulty: i32,
	pub tags: Vec<String>,
	pub examples: Vec<StoredExample>,
	pub constraints: Vec<String>,
	pub code_templates: Vec<StoredCodeTemplate>,
	pub is_published: bool,
	pub created_at: Option<String>,
	pub updated_at: Option<String>,
}
impl From<StoredProblem> for Problem {
	fn from(problem: StoredProblem) -> Self {
		Self {
			id: problem.id.to_string(),
			number: problem.number,
			title: problem.title,
			slug: problem.slug,
			description: problem.description,
			difficulty: problem.difficulty,
			tags: problem.tags,
			examples: problem.examples.into_iter().map(Into::into).collect(),
			constraints: problem.constraints,
			code_templates: problem.code_templates.into_iter().map(Into::into).collect(),
			is_published: problem.is_published,
			created_at: parse_timestamp(problem.created_at),
			updated_at: parse_timestamp(problem.updated_at),
		}
	}
}
impl From<Problem> for StoredProblem {
	fn from(problem: Problem) -> Self {
		Self {
			id: problem.id.parse().unwrap_or_default(),
			number: problem.number,
			title: problem.title,
			slug: problem.slug,
			description: problem.description,
			difficulty: problem.difficulty,
			tags: problem.tags,
			examples: problem.examples.into_iter().map(Into::into).collect(),
			constraints: problem.constraints,
			code_templates: problem.code_templates.into_iter().map(Into::into).collect(),
			is_published: problem.is_published,
			created_at: None,
			updated_at: None,
		}
	}
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StoredCodeTemplate {
	pub language: String,
	pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StoredExample {
	pub input: String,
	pub output: String,
	pub explanation: String,
}
impl From<StoredExample> for Example {
	fn from(value: StoredExample) -> Self {
		Self {
			input: value.input,
			output: value.output,
			explanation: value.explanation,
		}
	}
}
impl From<StoredCodeTemplate> for CodeTemplate {
	fn from(value: StoredCodeTemplate) -> Self {
		Self {
			language: value.language,
			source: value.source,
		}
	}
}
impl From<Example> for StoredExample {
	fn from(value: Example) -> Self {
		Self {
			input: value.input,
			output: value.output,
			explanation: value.explanation,
		}
	}
}
impl From<CodeTemplate> for StoredCodeTemplate {
	fn from(value: CodeTemplate) -> Self {
		Self {
			language: value.language,
			source: value.source,
		}
	}
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
